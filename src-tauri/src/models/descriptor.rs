use serde::{Deserialize, Serialize};

use crate::commands::ModelSource;

/// File-system descriptor.json representation per D-21.
/// Loaded at startup via scan_models_directory().
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DescriptorJson {
    pub id: String,
    pub name: String,
    pub description: String,
    pub filename: String,
    #[serde(default)]
    pub checksum: Option<String>,
    #[serde(default)]
    pub sources: Vec<ModelSource>,
    #[serde(default)]
    pub param_schema: serde_json::Value,
    #[serde(default)]
    pub capabilities: super::PluginCapabilities,
    #[serde(default)]
    pub input_size: Option<u32>,
    #[serde(default)]
    pub mean: Option<Vec<f32>>,
    #[serde(default)]
    pub std: Option<Vec<f32>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptor_json_deserialize_minimal() {
        let json = r#"{
            "id": "test-model",
            "name": "Test Model",
            "description": "A test model",
            "filename": "test.onnx"
        }"#;
        let desc: DescriptorJson = serde_json::from_str(json).expect("Failed to deserialize minimal descriptor");
        assert_eq!(desc.id, "test-model");
        assert_eq!(desc.name, "Test Model");
        assert_eq!(desc.description, "A test model");
        assert_eq!(desc.filename, "test.onnx");
        assert!(desc.checksum.is_none());
        assert!(desc.sources.is_empty());
        assert_eq!(desc.input_size, None);
    }

    #[test]
    fn test_descriptor_json_deserialize_full() {
        let json = r#"{
            "id": "birefnet",
            "name": "BiRefNet",
            "description": "通用高精度抠图模型",
            "filename": "birefnet.onnx",
            "checksum": "58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c",
            "inputSize": 1024,
            "mean": [0.485, 0.456, 0.406],
            "std": [0.229, 0.224, 0.225],
            "param_schema": {
                "type": "object",
                "properties": {}
            },
            "capabilities": {
                "matting": true,
                "backgroundReplace": false,
                "edgeRefinement": false,
                "uncertaintyMask": false
            },
            "sources": [
                {
                    "id": "modelscope",
                    "name": "ModelScope",
                    "description": "魔搭社区",
                    "url": "https://example.com/model.onnx",
                    "default": true
                }
            ]
        }"#;
        let desc: DescriptorJson = serde_json::from_str(json).expect("Failed to deserialize full descriptor");
        assert_eq!(desc.id, "birefnet");
        assert_eq!(desc.input_size, Some(1024));
        assert_eq!(desc.mean, Some(vec![0.485, 0.456, 0.406]));
        assert_eq!(desc.std, Some(vec![0.229, 0.224, 0.225]));
        assert!(desc.capabilities.matting);
        assert!(!desc.capabilities.background_replace);
        assert_eq!(desc.sources.len(), 1);
        assert_eq!(desc.sources[0].id, "modelscope");
    }

    #[test]
    fn test_descriptor_json_empty_param_schema_default() {
        let json = r#"{
            "id": "no-params",
            "name": "No Params",
            "description": "Model with no parameters",
            "filename": "noparams.onnx"
        }"#;
        let desc: DescriptorJson = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(
            desc.param_schema,
            serde_json::Value::Null
        );
        assert!(!desc.capabilities.matting);
    }
}
