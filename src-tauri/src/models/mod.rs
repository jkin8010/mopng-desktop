use std::path::PathBuf;

use image::DynamicImage;
use ndarray::Array3;
// ort::value::Tensor<f32> used in preprocess/postprocess trait methods
use serde::{Deserialize, Serialize};

use crate::commands::ModelSource;

pub mod birefnet;
pub mod descriptor;
pub mod registry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelState {
    NotDownloaded,
    Loading,
    Loaded,
    Error(String),
}

/// Per-model capability flags per D-02.
/// Declares what the model can do beyond basic inference.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginCapabilities {
    /// 输出透明 PNG 的抠图能力
    #[serde(default)]
    pub matting: bool,
    /// 背景替换能力
    #[serde(default, rename = "backgroundReplace")]
    pub background_replace: bool,
    /// 边缘羽化/平滑能力
    #[serde(default, rename = "edgeRefinement")]
    pub edge_refinement: bool,
    /// 置信度 mask 输出能力
    #[serde(default, rename = "uncertaintyMask")]
    pub uncertainty_mask: bool,
}

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

    /// Return a JSON Schema describing model-specific inference parameters.
    /// Default: no parameters (empty schema). Per D-01.
    fn param_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    /// Return capability flags for this model. Per D-02.
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::default()
    }

    /// Preprocess a DynamicImage into an ONNX input tensor (NCHW format).
    /// Each model defines its own resize/normalization pipeline. Per D-03.
    fn preprocess(&self, _image: DynamicImage) -> Result<ort::value::Tensor<f32>, Box<dyn std::error::Error>>;

    /// Postprocess ONNX output tensor into (H, W, 1) u8 alpha mask.
    /// Each model defines its own output extraction logic. Per D-03.
    fn postprocess(
        &self,
        _tensor: ort::value::Tensor<f32>,
        _original_dims: (u32, u32),
    ) -> Result<Array3<u8>, Box<dyn std::error::Error>>;
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test struct that implements MattingModel minimally for testing defaults.
    struct TestModel;

    impl MattingModel for TestModel {
        fn id(&self) -> &str { "test" }
        fn name(&self) -> &str { "Test" }
        fn description(&self) -> &str { "Test model" }
        fn init(&mut self, _: PathBuf) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
        fn is_loaded(&self) -> bool { true }
        fn infer(&mut self, _: DynamicImage) -> Result<Array3<u8>, Box<dyn std::error::Error>> {
            Ok(Array3::zeros((1, 1, 1)))
        }
        fn filename(&self) -> &str { "test.onnx" }
        fn preprocess(&self, _: DynamicImage) -> Result<ort::value::Tensor<f32>, Box<dyn std::error::Error>> {
            Err("not implemented".into())
        }
        fn postprocess(&self, _: ort::value::Tensor<f32>, _: (u32, u32)) -> Result<Array3<u8>, Box<dyn std::error::Error>> {
            Err("not implemented".into())
        }
    }

    #[test]
    fn test_plugin_capabilities_default() {
        let caps = PluginCapabilities::default();
        assert!(!caps.matting);
        assert!(!caps.background_replace);
        assert!(!caps.edge_refinement);
        assert!(!caps.uncertainty_mask);
    }

    #[test]
    fn test_param_schema_default_returns_empty_json_schema() {
        let model = TestModel;
        let schema = model.param_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_capabilities_default_returns_all_false() {
        let model = TestModel;
        let caps = model.capabilities();
        assert!(!caps.matting);
        assert!(!caps.background_replace);
        assert!(!caps.edge_refinement);
        assert!(!caps.uncertainty_mask);
    }
}
