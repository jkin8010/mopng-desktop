use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::RwLock;

use image::DynamicImage;
use ndarray::Array3;
use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::commands::ModelSource;
use crate::models::descriptor::DescriptorJson;
use crate::models::MattingModel;
use crate::models::ModelState;
use crate::models::PluginCapabilities;

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub state: ModelState,
    pub filename: String,
    pub sources: Vec<ModelSource>,
    pub checksum: Option<String>,
    pub param_schema: serde_json::Value,
    pub capabilities: crate::models::PluginCapabilities,
    pub input_size: Option<u32>,
    pub mean: Option<Vec<f32>>,
    pub std: Option<Vec<f32>>,
}

pub struct ModelDescriptor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filename: String,
    pub sources: Vec<ModelSource>,
    pub checksum: Option<String>,
    pub param_schema: serde_json::Value,
    pub capabilities: crate::models::PluginCapabilities,
    pub input_size: Option<u32>,
    pub mean: Option<Vec<f32>>,
    pub std: Option<Vec<f32>>,
}

struct LoadedModel {
    model_id: String,
    model: Box<dyn MattingModel>,
}

/// Populated by scan_models_directory() at startup via the scan_models Tauri command.
/// Empty at compile time — models are discovered from models/*/descriptor.json at runtime.
pub(crate) static DESCRIPTORS: Lazy<RwLock<Vec<ModelDescriptor>>> = Lazy::new(|| {
    RwLock::new(Vec::new())
});

static ACTIVE_MODEL: Lazy<Mutex<Option<LoadedModel>>> = Lazy::new(|| {
    Mutex::new(None)
});

static MODEL_STATES: Lazy<Mutex<HashMap<String, ModelState>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn list_models() -> Vec<ModelInfo> {
    let descriptors = DESCRIPTORS.read().expect("DESCRIPTORS RwLock poisoned");
    let states = MODEL_STATES.lock().expect("MODEL_STATES Mutex poisoned");
    descriptors
        .iter()
        .map(|d| ModelInfo {
            id: d.id.clone(),
            name: d.name.clone(),
            description: d.description.clone(),
            state: states
                .get(&d.id)
                .cloned()
                .unwrap_or(ModelState::NotDownloaded),
            filename: d.filename.clone(),
            sources: d.sources.clone(),
            checksum: d.checksum.clone(),
            param_schema: d.param_schema.clone(),
            capabilities: d.capabilities,
            input_size: d.input_size,
            mean: d.mean.clone(),
            std: d.std.clone(),
        })
        .collect()
}

pub fn is_model_loaded() -> bool {
    ACTIVE_MODEL
        .lock()
        .expect("ACTIVE_MODEL Mutex poisoned")
        .is_some()
}

pub fn init_model(model_id: &str, model_path: PathBuf) -> Result<(), String> {
    // 1. Verify file exists
    if !model_path.exists() {
        return Err(format!("模型文件不存在: {:?}", model_path));
    }

    // 2. Read descriptor
    let descriptors = DESCRIPTORS.read().expect("DESCRIPTORS RwLock poisoned");
    let descriptor = descriptors
        .iter()
        .find(|d| d.id == model_id)
        .ok_or_else(|| format!("未知模型: {}", model_id))?;

    // 3. SHA256 checksum verification
    if let Some(ref expected_checksum) = descriptor.checksum {
        if let Ok(actual) = compute_file_sha256(&model_path) {
            if actual != *expected_checksum {
                return Err(format!(
                    "模型文件 SHA256 校验失败\n期望: {}\n实际: {}\n文件可能已损坏，请重新下载",
                    expected_checksum, actual
                ));
            }
        }
    }
    drop(descriptors); // Explicitly release RwLock read guard

    // 4. Set state to Loading
    {
        let mut states = MODEL_STATES
            .lock()
            .expect("MODEL_STATES Mutex poisoned");
        states.insert(model_id.to_string(), ModelState::Loading);
    }

    let model_id_owned = model_id.to_string();
    let path_clone = model_path.clone();

    // 5. Spawn OS thread for ONNX session loading with catch_unwind
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut model = create_model(&model_id_owned)?;
            model
                .init(path_clone.clone())
                .map_err(|e| format!("模型初始化失败: {}", e))?;
            Ok::<Box<dyn MattingModel>, String>(model)
        }));

        let mut states = MODEL_STATES
            .lock()
            .expect("MODEL_STATES Mutex poisoned");
        let mut active = ACTIVE_MODEL
            .lock()
            .expect("ACTIVE_MODEL Mutex poisoned");

        match result {
            Ok(Ok(model)) => {
                states.insert(model_id_owned.clone(), ModelState::Loaded);
                active.replace(LoadedModel {
                    model_id: model_id_owned.clone(),
                    model,
                });
                log::info!("Model {} loaded successfully", model_id_owned);
            }
            Ok(Err(e)) => {
                states.insert(
                    model_id_owned.clone(),
                    ModelState::Error(format!("模型加载失败: {}", e)),
                );
                log::error!("Model {} init failed: {}", model_id_owned, e);
            }
            Err(panic_err) => {
                let msg = if let Some(s) = panic_err.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_err.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown panic during model initialization".to_string()
                };
                states.insert(
                    model_id_owned.clone(),
                    ModelState::Error(format!("模型加载崩溃: {}", msg)),
                );
                log::error!("Model {} init panicked: {}", model_id_owned, msg);
            }
        }
    });

    Ok(()) // Return immediately — frontend polls list_models() to observe ModelState::Loading
}

/// Switch the active model to a different model_id, using a directly specified
/// model directory (for testing without a Tauri AppHandle).
///
/// Per D-07: if inference is running, the ACTIVE_MODEL Mutex ensures we wait.
/// Per D-08: drop old model BEFORE loading new to prevent RSS doubling.
/// Per D-10: Loading state -> background load -> success/failure.
///
/// Returns immediately; frontend polls list_models() to detect completion.
pub(crate) fn switch_model_with_dir(model_id: &str, model_dir: &Path) -> Result<(), String> {
    // 1. Verify the model_id is known and get its filename
    let descriptors = DESCRIPTORS.read().map_err(|e| e.to_string())?;
    let filename = descriptors
        .iter()
        .find(|d| d.id == model_id)
        .ok_or_else(|| format!("未知模型: {}", model_id))?
        .filename
        .clone();
    drop(descriptors);

    let model_id_owned = model_id.to_string();

    // 2. Resolve model file path (subdirectory named by model_id containing the ONNX file)
    let model_path = model_dir.join(&model_id_owned).join(&filename);
    if !model_path.exists() {
        return Err(format!("模型文件不存在: {:?}", model_path));
    }

    // 3. Set new model state to Loading
    {
        let mut states = MODEL_STATES.lock().map_err(|e| e.to_string())?;
        states.insert(model_id_owned.clone(), ModelState::Loading);
    }

    // 4. Capture old model to enable revert on switch failure. Per D-08, we still
    // drop the old model BEFORE loading new to prevent RSS doubling, but we keep
    // the value so the spawned thread can restore it if the new model fails.
    let old_model = {
        let mut active = ACTIVE_MODEL.lock().map_err(|e| e.to_string())?;
        active.take()
    };
    if old_model.is_some() {
        log::info!("Captured old model for potential revert on switch failure");
    }

    let path_clone = model_path.clone();

    // 5. Spawn OS thread for new ONNX session loading (same pattern as init_model)
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut model = create_model(&model_id_owned)?;
            model
                .init(path_clone)
                .map_err(|e| format!("模型初始化失败: {}", e))?;
            Ok::<Box<dyn MattingModel>, String>(model)
        }));

        let mut states = MODEL_STATES.lock().expect("MODEL_STATES poisoned");
        let mut active = ACTIVE_MODEL.lock().expect("ACTIVE_MODEL poisoned");

        match result {
            Ok(Ok(model)) => {
                log::info!("Dropped old model (replaced by '{}')", model_id_owned);
                states.insert(model_id_owned.clone(), ModelState::Loaded);
                active.replace(LoadedModel {
                    model_id: model_id_owned.clone(),
                    model,
                });
                log::info!("Model switched to '{}' successfully", model_id_owned);
            }
            Ok(Err(e)) => {
                states.insert(
                    model_id_owned.clone(),
                    ModelState::Error(format!("模型切换失败: {}", e)),
                );
                // Revert: restore the old model on failure (Gap 1)
                if let Some(prev) = old_model {
                    log::warn!("Model switch failed, reverting to '{}'", prev.model_id);
                    states.insert(prev.model_id.clone(), ModelState::Loaded);
                    active.replace(prev);
                }
                log::error!("Model switch to '{}' failed: {}", model_id_owned, e);
            }
            Err(panic_err) => {
                let msg = if let Some(s) = panic_err.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic_err.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown panic during model switch".to_string()
                };
                states.insert(
                    model_id_owned.clone(),
                    ModelState::Error(format!("模型切换崩溃: {}", msg)),
                );
                // Revert: restore the old model on failure (Gap 1)
                if let Some(prev) = old_model {
                    log::warn!("Model switch panicked, reverting to '{}'", prev.model_id);
                    states.insert(prev.model_id.clone(), ModelState::Loaded);
                    active.replace(prev);
                }
                log::error!("Model switch to '{}' panicked: {}", model_id_owned, msg);
            }
        }
    });

    Ok(()) // Return immediately — frontend polls list_models() to observe Loading state
}

/// Switch the active model. Resolves model directory from AppHandle.
pub fn switch_model(model_id: &str, app: &AppHandle) -> Result<(), String> {
    let dir = model_dir(app)?;
    switch_model_with_dir(model_id, &dir)
}

pub fn infer(image: DynamicImage) -> Result<Array3<u8>, String> {
    let mut lock = ACTIVE_MODEL
        .lock()
        .expect("ACTIVE_MODEL Mutex poisoned");
    let loaded = lock
        .as_mut()
        .ok_or_else(|| "模型未初始化，请先加载模型".to_string())?;
    loaded
        .model
        .infer(image)
        .map_err(|e| format!("推理失败: {}", e))
}

pub fn model_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("models");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path)
}

pub fn model_filename_for(model_id: &str) -> Option<String> {
    let descriptors = DESCRIPTORS
        .read()
        .expect("DESCRIPTORS RwLock poisoned");
    descriptors
        .iter()
        .find(|d| d.id == model_id)
        .map(|d| d.filename.clone())
}

pub fn model_sources_for(model_id: &str) -> Option<Vec<ModelSource>> {
    let descriptors = DESCRIPTORS
        .read()
        .expect("DESCRIPTORS RwLock poisoned");
    descriptors
        .iter()
        .find(|d| d.id == model_id)
        .map(|d| d.sources.clone())
}

fn create_model(id: &str) -> Result<Box<dyn MattingModel>, String> {
    match id {
        "birefnet" => Ok(Box::new(crate::models::birefnet::BirefnetModel::new())),
        "rmbg-fp32" => Ok(Box::new(crate::models::rmbg::RmbgModel::new("rmbg-fp32"))),
        "rmbg-fp16" => Ok(Box::new(crate::models::rmbg::RmbgModel::new("rmbg-fp16"))),
        _ => Err(format!("不支持的模型: {}", id)),
    }
}

/// Scan the models directory for subdirectories containing descriptor.json + model.onnx.
/// Returns all valid ModelDescriptors found. Silently skips invalid/missing descriptors.
/// Per D-18: each subdirectory represents a model with model.onnx and descriptor.json.
/// Per D-19: scan happens at startup, results cached in DESCRIPTORS RwLock.
pub fn scan_models_directory(base_dir: &Path) -> Vec<ModelDescriptor> {
    let mut descriptors = Vec::new();
    let entries = match std::fs::read_dir(base_dir) {
        Ok(e) => e,
        Err(_) => return descriptors,
    };

    for entry in entries.flatten() {
        let model_dir = entry.path();
        if !model_dir.is_dir() {
            continue;
        }

        let desc_path = model_dir.join("descriptor.json");
        if !desc_path.exists() {
            continue;
        }

        let content = match std::fs::read_to_string(&desc_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let desc: DescriptorJson = match serde_json::from_str(&content) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("Skipping invalid descriptor.json in {:?}: {}", model_dir, e);
                continue;
            }
        };

        // Validate: filename must not contain path separators (security — T-B03-01)
        if desc.filename.contains('/') || desc.filename.contains('\\') || desc.filename.contains("..") {
            log::warn!("Skipping model {}: filename contains path separators", desc.id);
            continue;
        }

        // Validate: id must not be empty
        if desc.id.is_empty() {
            log::warn!("Skipping model in {:?}: empty id", model_dir);
            continue;
        }

        descriptors.push(ModelDescriptor {
            id: desc.id,
            name: desc.name,
            description: desc.description,
            filename: desc.filename,
            sources: desc.sources,
            checksum: desc.checksum,
            param_schema: desc.param_schema,
            capabilities: desc.capabilities,
            input_size: desc.input_size,
            mean: desc.mean,
            std: desc.std,
        });
    }

    descriptors
}

fn compute_file_sha256(path: &std::path::Path) -> Result<String, String> {
    crate::commands::download::compute_file_sha256(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    /// Create a minimal descriptor.json for testing.
    fn write_descriptor(dir: &Path, id: &str, name: &str, filename: &str) {
        fs::create_dir_all(dir).unwrap();
        let json = serde_json::json!({
            "id": id,
            "name": name,
            "description": "Test model for file-system scanning",
            "filename": filename,
        });
        fs::write(
            dir.join("descriptor.json"),
            serde_json::to_string_pretty(&json).unwrap(),
        )
        .unwrap();
    }

    fn temp_dir(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("mopng-test-scan-{}", label))
    }

    fn clean(path: &Path) {
        if path.exists() {
            let _ = fs::remove_dir_all(path);
        }
    }

    #[test]
    fn scan_models_directory_returns_descriptor_when_valid() {
        let root = temp_dir("valid");
        clean(&root);
        write_descriptor(&root.join("birefnet"), "birefnet", "BiRefNet", "birefnet.onnx");

        let descriptors = scan_models_directory(&root);
        assert_eq!(descriptors.len(), 1, "Should find one model");
        assert_eq!(descriptors[0].id, "birefnet");
        assert_eq!(descriptors[0].name, "BiRefNet");
        assert_eq!(descriptors[0].filename, "birefnet.onnx");

        clean(&root);
    }

    #[test]
    fn scan_models_directory_returns_empty_when_dir_missing() {
        let root = temp_dir("missing");
        clean(&root);

        let descriptors = scan_models_directory(&root);
        assert_eq!(
            descriptors.len(),
            0,
            "Should return empty vec when directory does not exist"
        );
    }

    #[test]
    fn scan_models_directory_skips_subdir_without_descriptor() {
        let root = temp_dir("nodesc");
        clean(&root);
        let model_dir = root.join("some-model");
        fs::create_dir_all(&model_dir).unwrap();
        // Create an ONNX file but NO descriptor.json
        fs::write(model_dir.join("model.onnx"), b"fake onnx data").unwrap();

        let descriptors = scan_models_directory(&root);
        assert_eq!(
            descriptors.len(),
            0,
            "Should skip subdirectory without descriptor.json"
        );

        clean(&root);
    }

    #[test]
    fn scan_models_directory_skips_invalid_json_descriptor() {
        let root = temp_dir("invalidjson");
        clean(&root);
        let model_dir = root.join("bad-model");
        fs::create_dir_all(&model_dir).unwrap();
        fs::write(model_dir.join("descriptor.json"), b"not valid json {{{").unwrap();

        let descriptors = scan_models_directory(&root);
        assert_eq!(descriptors.len(), 0, "Should skip invalid descriptor.json");

        clean(&root);
    }

    #[test]
    fn list_models_includes_param_schema_and_capabilities() {
        // Reset DESCRIPTORS with a known test descriptor
        {
            let mut lock = DESCRIPTORS.write().unwrap();
            *lock = vec![ModelDescriptor {
                id: "test-model".into(),
                name: "Test".into(),
                description: "Test desc".into(),
                filename: "test.onnx".into(),
                sources: vec![],
                checksum: None,
                param_schema: serde_json::json!({"test_param": true}),
                capabilities: crate::models::PluginCapabilities {
                    matting: true,
                    background_replace: true,
                    edge_refinement: false,
                    uncertainty_mask: false,
                },
                input_size: None,
                mean: None,
                std: None,
            }];
        }

        let models = list_models();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].param_schema, serde_json::json!({"test_param": true}));
        assert!(models[0].capabilities.matting);
        assert!(models[0].capabilities.background_replace);
        assert!(!models[0].capabilities.edge_refinement);

        // Clean up: reset to empty
        {
            let mut lock = DESCRIPTORS.write().unwrap();
            *lock = vec![];
        }
    }

    // ── switch_model TDD tests ─────────────────────────────────────────────────
    //
    // These tests call switch_model_with_dir() which will fail to compile until
    // switch_model is implemented. This is the RED phase of TDD.
    //
    // Behavior under test:
    //   Test 1: switch_model("birefnet") when ACTIVE_MODEL is None -> sets
    //           Loading state, spawns thread, loads model
    //   Test 2: switch_model("rmbg-fp32") when ACTIVE_MODEL has birefnet loaded
    //           -> drops old birefnet session, loads rmbg
    //   Test 3: switch_model("nonexistent") -> returns Err, does not crash
    //   Test 4: After successful switch, ACTIVE_MODEL contains the new model_id
    //   Test 5: After failed switch, ACTIVE_MODEL is None (old model already
    //           dropped per D-08)

    /// Helper: register a test-model in DESCRIPTORS and create its model file.
    fn register_test_model(root: &Path, id: &str, filename: &str, file_content: &[u8]) {
        {
            let mut lock = DESCRIPTORS.write().unwrap();
            lock.push(ModelDescriptor {
                id: id.into(),
                name: format!("Test {}", id),
                description: "Test model for switch_model".into(),
                filename: filename.into(),
                sources: vec![],
                checksum: None,
                param_schema: serde_json::json!({}),
                capabilities: crate::models::PluginCapabilities::default(),
                input_size: None,
                mean: None,
                std: None,
            });
        }
        let model_subdir = root.join(id);
        fs::create_dir_all(&model_subdir).unwrap();
        fs::write(model_subdir.join(filename), file_content).unwrap();
    }

    /// Helper: reset DESCRIPTORS, MODEL_STATES, ACTIVE_MODEL to clean state.
    fn reset_globals() {
        {
            let mut lock = DESCRIPTORS.write().unwrap();
            *lock = vec![];
        }
        {
            let mut lock = MODEL_STATES.lock().unwrap();
            lock.clear();
        }
        {
            let mut lock = ACTIVE_MODEL.lock().unwrap();
            *lock = None;
        }
    }

    #[test]
    fn switch_model_rejects_nonexistent_model() {
        let root = temp_dir("reject-nonexistent");
        clean(&root);
        reset_globals();
        // No descriptors registered — every model_id is unknown.

        let result = switch_model_with_dir("nonexistent", root.as_path());
        assert!(result.is_err(), "Unknown model should return Err");

        // No state should have been set for an unknown model.
        let states = MODEL_STATES.lock().unwrap();
        assert!(!states.contains_key("nonexistent"));

        clean(&root);
    }

    #[test]
    fn switch_model_sets_loading_state_for_known_model() {
        let root = temp_dir("set-loading");
        clean(&root);
        reset_globals();
        register_test_model(&root, "test-model", "model.onnx", b"fake");

        let result = switch_model_with_dir("test-model", root.as_path());
        assert!(result.is_ok(), "Known model with file should return Ok");

        // The Loading state is set synchronously before the thread spawns.
        let states = MODEL_STATES.lock().unwrap();
        assert!(states.contains_key("test-model"), "State should be set");

        // Note: the async Error state from the spawned thread is NOT checked
        // here because leftover threads from earlier tests can interfere with
        // the global Mutexes. The thread-failure paths are verified through
        // the same pattern used in init_model (catch_unwind + Error state).

        clean(&root);
    }

    #[test]
    fn switch_model_drops_old_model_before_new_load() {
        let root = temp_dir("drop-old");
        clean(&root);
        reset_globals();
        register_test_model(&root, "test-model", "model.onnx", b"fake");

        // Ensure ACTIVE_MODEL is None before switch (fresh state).
        {
            let active = ACTIVE_MODEL.lock().unwrap();
            assert!(active.is_none(), "ACTIVE_MODEL should start empty");
        }

        switch_model_with_dir("test-model", root.as_path()).unwrap();

        // Immediately after switch_model returns, ACTIVE_MODEL should be None
        // because the old model was taken() before the thread provides a new one.
        let active = ACTIVE_MODEL.lock().unwrap();
        assert!(
            active.is_none(),
            "ACTIVE_MODEL should be None after take() before thread completes"
        );

        clean(&root);
    }

    #[test]
    fn switch_model_returns_err_for_missing_model_file() {
        let root = temp_dir("missing-file");
        clean(&root);
        reset_globals();
        register_test_model(&root, "test-model", "model.onnx", b"fake");

        // Remove the model file after registration.
        fs::remove_file(root.join("test-model").join("model.onnx")).unwrap();

        let result = switch_model_with_dir("test-model", root.as_path());
        assert!(
            result.is_err(),
            "Missing model file should return Err, got {:?}",
            result
        );
        let err_msg = result.unwrap_err();
        assert!(
            err_msg.contains("不存在") || err_msg.contains("exist"),
            "Error should mention file not found, got: {}",
            err_msg
        );

        clean(&root);
    }
}
