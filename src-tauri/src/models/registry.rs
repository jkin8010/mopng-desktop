use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::RwLock;

use image::DynamicImage;
use ndarray::Array3;
use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::commands::ModelSource;
use crate::models::MattingModel;
use crate::models::ModelState;

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

static DESCRIPTORS: Lazy<RwLock<Vec<ModelDescriptor>>> = Lazy::new(|| {
    // Temporary hardcoded descriptor — will be replaced by file-system scanning (D-18/D-19)
    RwLock::new(vec![ModelDescriptor {
        id: "birefnet".to_string(),
        name: "BiRefNet".to_string(),
        description: "通用高精度抠图模型，支持各类主体（人物、物体、动物等）".to_string(),
        filename: "birefnet.onnx".to_string(),
        checksum: Some("58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c".to_string()),
        param_schema: serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        capabilities: crate::models::PluginCapabilities {
            matting: true,
            background_replace: false,
            edge_refinement: false,
            uncertainty_mask: false,
        },
        input_size: Some(1024),
        mean: Some(vec![0.485, 0.456, 0.406]),
        std: Some(vec![0.229, 0.224, 0.225]),
        sources: vec![
            ModelSource {
                id: "modelscope".into(),
                name: "ModelScope".into(),
                description: "魔搭社区，国内可直接访问".into(),
                url: "https://modelscope.cn/models/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model.onnx".into(),
                default: true,
            },
            ModelSource {
                id: "huggingface".into(),
                name: "HuggingFace".into(),
                description: "海外源，需科学上网".into(),
                url: "https://huggingface.co/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model.onnx".into(),
                default: false,
            },
            ModelSource {
                id: "hf-mirror".into(),
                name: "HF Mirror".into(),
                description: "HuggingFace 国内镜像".into(),
                url: "https://hf-mirror.com/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model.onnx".into(),
                default: false,
            },
        ],
    }])
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
        .map(|d| d.filename.to_string())
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
        _ => Err(format!("不支持的模型: {}", id)),
    }
}

fn compute_file_sha256(path: &std::path::Path) -> Result<String, String> {
    crate::commands::download::compute_file_sha256(path)
}
