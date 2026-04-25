pub mod birefnet;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MattingSettings {
    pub mode: String,
    pub output_format: String,
    pub quality: u32,
    pub bg_type: String,
    pub bg_color: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailParams {
    pub path: String,
    pub max_size: u32,
}

/// 初始化模型（由前端在确认模型存在后调用）
#[tauri::command]
pub fn init_model(model_path: String, _provider: Option<String>) -> Result<(), String> {
    let path = PathBuf::from(model_path);
    if !path.exists() {
        return Err(format!("模型文件不存在: {:?}", path));
    }

    log::info!("开始加载模型到内存...");
    log::info!("模型路径: {:?}", path);
    log::info!("模型大小: {} MB", path.metadata().ok().map(|m| m.len() / 1_048_576).unwrap_or(0));

    let result = birefnet::BirefnetSession::init(path)
        .map_err(|e| format!("模型初始化失败: {}", e));

    log::info!("BiRefNet 模型初始化{:?}", result.as_ref().map(|_| "成功").unwrap_or(&"失败"));
    result
}

/// 检查模型是否已加载到内存
#[tauri::command]
pub fn is_model_loaded() -> bool {
    birefnet::BirefnetSession::get().is_some()
}
