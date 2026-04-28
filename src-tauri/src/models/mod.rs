use std::path::PathBuf;

use image::DynamicImage;
use ndarray::Array3;
use serde::{Deserialize, Serialize};

use crate::commands::ModelSource;

pub mod birefnet;
pub mod registry;

/// Plugin protocol for matting models.
pub trait MattingModel: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn init(&mut self, model_path: PathBuf) -> Result<(), Box<dyn std::error::Error>>;
    fn is_loaded(&self) -> bool;
    fn infer(&mut self, image: DynamicImage) -> Result<Array3<u8>, Box<dyn std::error::Error>>;
    fn filename(&self) -> &str;
    fn sources(&self) -> Vec<ModelSource> {
        vec![]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MattingSettings {
    pub mode: String,
    pub output_format: String,
    pub quality: u32,
    pub bg_type: String,
    pub bg_color: Option<String>,
    pub bg_image_url: Option<String>,
    pub bg_opacity: Option<u32>,
    pub target_width: Option<u32>,
    pub target_height: Option<u32>,
    pub maintain_aspect_ratio: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessParams {
    pub file_path: String,
    pub settings: MattingSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessResult {
    pub output_path: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub file_size: u64,
    pub preview_path: String,
    pub mask_data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailParams {
    pub path: String,
    pub max_size: u32,
}

#[tauri::command]
pub fn init_model(model_id: String, model_path: String) -> Result<(), String> {
    let path = PathBuf::from(&model_path);
    if !path.exists() {
        return Err(format!("模型文件不存在: {}", model_path));
    }
    log::info!("开始加载模型到内存...");
    log::info!("模型路径: {:?}", path);
    log::info!(
        "模型大小: {} MB",
        path.metadata()
            .ok()
            .map(|m| m.len() / 1_048_576)
            .unwrap_or(0)
    );
    registry::init_model(&model_id, path)
}

#[tauri::command]
pub fn is_model_loaded() -> bool {
    registry::is_model_loaded()
}

#[tauri::command]
pub fn list_models() -> Vec<registry::ModelInfo> {
    registry::list_models()
}
