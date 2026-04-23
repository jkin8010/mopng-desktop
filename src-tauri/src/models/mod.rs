use serde::{Deserialize, Serialize};

// pub mod birefnet;  // 取消注释以启用 BiRefNet ONNX 推理
// pub mod session;   // 取消注释以启用 BiRefNet ONNX 推理

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
