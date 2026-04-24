pub mod birefnet;
pub mod session;

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
pub fn init_model(model_path: String, provider: Option<String>) -> Result<(), String> {
    let path = PathBuf::from(model_path);
    if !path.exists() {
        return Err(format!("模型文件不存在: {:?}", path));
    }

    let provider = provider.unwrap_or_else(|| "coreml".to_string());

    birefnet::BirefnetSession::init(path, &provider)
        .map_err(|e| format!("模型初始化失败: {}", e))?;

    log::info!("BiRefNet 模型初始化成功，Provider: {}", provider);
    Ok(())
}

/// 检查模型是否已加载到内存
#[tauri::command]
pub fn is_model_loaded() -> bool {
    birefnet::BirefnetSession::get().is_some()
}
