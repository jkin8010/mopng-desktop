use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use image::DynamicImage;
use ndarray::Array3;
use once_cell::sync::Lazy;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::commands::ModelSource;
use crate::models::MattingModel;

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub loaded: bool,
    pub filename: String,
    pub sources: Vec<ModelSource>,
}

pub struct ModelDescriptor {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub filename: &'static str,
    pub sources: Vec<ModelSource>,
}

struct RegistryInner {
    descriptors: Vec<ModelDescriptor>,
    loaded: Option<(String, Box<dyn MattingModel>)>,
}

static REGISTRY: Lazy<Mutex<RegistryInner>> = Lazy::new(|| {
    Mutex::new(RegistryInner {
        descriptors: vec![crate::models::birefnet::descriptor()],
        loaded: None,
    })
});

pub fn list_models() -> Vec<ModelInfo> {
    let lock = REGISTRY.lock().expect("Registry mutex poisoned");
    let loaded_id = lock.loaded.as_ref().map(|(id, _)| id.clone());
    lock.descriptors
        .iter()
        .map(|d| ModelInfo {
            id: d.id.to_string(),
            name: d.name.to_string(),
            description: d.description.to_string(),
            loaded: loaded_id.as_deref() == Some(d.id),
            filename: d.filename.to_string(),
            sources: d.sources.clone(),
        })
        .collect()
}

pub fn init_model(model_id: &str, model_path: PathBuf) -> Result<(), String> {
    let mut lock = REGISTRY.lock().expect("Registry mutex poisoned");

    let _descriptor = lock
        .descriptors
        .iter()
        .find(|d| d.id == model_id)
        .ok_or_else(|| format!("未知模型: {}", model_id))?;

    let mut model = create_model(model_id)?;
    model
        .init(model_path.clone())
        .map_err(|e| format!("模型初始化失败: {}", e))?;

    log::info!("模型 {} 已加载: {:?}", model_id, model_path);
    lock.loaded = Some((model_id.to_string(), model));

    Ok(())
}

pub fn is_model_loaded() -> bool {
    let lock = REGISTRY.lock().expect("Registry mutex poisoned");
    lock.loaded.is_some()
}

pub fn infer(image: DynamicImage) -> Result<Array3<u8>, String> {
    let mut lock = REGISTRY.lock().expect("Registry mutex poisoned");
    let (_id, model) = lock
        .loaded
        .as_mut()
        .ok_or("模型未初始化，请先加载模型")?;
    model
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
    let lock = REGISTRY.lock().expect("Registry mutex poisoned");
    lock.descriptors
        .iter()
        .find(|d| d.id == model_id)
        .map(|d| d.filename.to_string())
}

pub fn model_sources_for(model_id: &str) -> Option<Vec<ModelSource>> {
    let lock = REGISTRY.lock().expect("Registry mutex poisoned");
    lock.descriptors
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
