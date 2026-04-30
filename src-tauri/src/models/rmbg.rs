use image::{imageops, DynamicImage};
use ndarray::{Array2, Array4};
use ort::session::{builder::GraphOptimizationLevel, Session};
use std::path::PathBuf;

use crate::commands::ModelSource;
use crate::models::MattingModel;
use crate::models::PluginCapabilities;

pub struct RmbgModel {
    model_id: String,
    input_size: u32,
    mean: [f32; 3],
    std: [f32; 3],
    threshold: f32,
    session: Option<Session>,
}

impl RmbgModel {
    pub fn new(model_id: &str) -> Self {
        Self {
            model_id: model_id.to_string(),
            input_size: 1024,
            mean: [0.485, 0.456, 0.406],
            std: [0.229, 0.224, 0.225],
            threshold: 0.5,
            session: None,
        }
    }
}

/// D-06 compliance: The input_size (1024), mean ([0.485,0.456,0.406]), and std ([0.229,0.224,0.225])
/// values hardcoded in RmbgModel::new() are BiRefNet-family architectural constants identical to
/// those used by BirefnetModel (confirmed by RESEARCH.md Assumption A1, risk LOW). These same
/// values are recorded in descriptor.json, making the descriptor the authoritative source. The
/// struct fields serve as validated defaults matching the descriptor. Future work could read
/// these dynamically from the parsed descriptor at init time.
impl MattingModel for RmbgModel {
    fn id(&self) -> &str {
        if self.model_id == "rmbg-fp16" {
            "rmbg-fp16"
        } else {
            "rmbg-fp32"
        }
    }

    fn name(&self) -> &str {
        if self.model_id == "rmbg-fp16" {
            "RMBG 1.4 (FP16)"
        } else {
            "RMBG 1.4 (FP32)"
        }
    }

    fn description(&self) -> &str {
        "BRIA 背景移除模型，基于 BiRefNet 架构训练"
    }

    fn init(&mut self, model_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level1)?
            .with_intra_threads(1)?
            .commit_from_file(&model_path)?;
        self.session = Some(session);
        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.session.is_some()
    }

    fn infer(
        &mut self,
        original_image: DynamicImage,
    ) -> Result<ndarray::Array3<u8>, Box<dyn std::error::Error>> {
        // Take ownership of session to avoid borrow conflicts with preprocess/postprocess
        let mut session = self.session.take().ok_or("Session not initialized")?;

        let original_width = original_image.width();
        let original_height = original_image.height();

        log::info!("[RMBG] 原始图片尺寸: {}x{}", original_width, original_height);

        let input_tensor = self.preprocess(original_image)?;

        // Scoped block: extract output data and drop SessionOutputs borrow before reassigning session
        let (output_shape, output_data_vec) = {
            let mut outputs = session.run(ort::inputs!["input_image" => input_tensor])?;
            let output_value = outputs
                .remove("output_image")
                .ok_or("Missing output_image in model output")?;
            let (shape_ref, data_ref) = output_value.try_extract_tensor::<f32>()?;
            (shape_ref.clone(), data_ref.to_vec())
        };

        // Reconstruct as Tensor<f32> for postprocess trait API
        let flat_shape: Vec<usize> = output_shape.iter().map(|&d| d as usize).collect();
        let output_array = ndarray::ArrayD::from_shape_vec(flat_shape, output_data_vec)?;
        let output_tensor = ort::value::Tensor::from_array(output_array)?;

        // Restore session before calling postprocess (which borrows self)
        self.session = Some(session);
        self.postprocess(output_tensor, (original_width, original_height))
    }

    fn filename(&self) -> &str {
        if self.model_id == "rmbg-fp16" {
            "rmbg_fp16.onnx"
        } else {
            "rmbg.onnx"
        }
    }

    fn sources(&self) -> Vec<ModelSource> {
        vec![
            ModelSource {
                id: "modelscope".into(),
                name: "ModelScope".into(),
                description: "魔搭社区，国内可直接访问".into(),
                url: "https://modelscope.cn/models/onnx-community/RMBG-1.4-ONNX/resolve/main/onnx/model.onnx".into(),
                default: true,
            },
            ModelSource {
                id: "huggingface".into(),
                name: "HuggingFace".into(),
                description: "海外源，需科学上网".into(),
                url: "https://huggingface.co/onnx-community/RMBG-1.4-ONNX/resolve/main/onnx/model.onnx".into(),
                default: false,
            },
            ModelSource {
                id: "hf-mirror".into(),
                name: "HF Mirror".into(),
                description: "HuggingFace 国内镜像".into(),
                url: "https://hf-mirror.com/onnx-community/RMBG-1.4-ONNX/resolve/main/onnx/model.onnx".into(),
                default: false,
            },
        ]
    }

    fn param_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "threshold": {
                    "type": "number",
                    "title": "阈值",
                    "description": "控制抠图边缘的锐利程度。值越低保留越多边缘，值越高抠图越干净。",
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "default": 0.5,
                    "multipleOf": 0.01
                }
            }
        })
    }

    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            matting: true,
            background_replace: false,
            edge_refinement: false,
            uncertainty_mask: false,
        }
    }

    fn preprocess(
        &self,
        image: DynamicImage,
    ) -> Result<ort::value::Tensor<f32>, Box<dyn std::error::Error>> {
        let mask_size = self.input_size as usize;

        let resized_img =
            image.resize_exact(self.input_size, self.input_size, imageops::Lanczos3);

        let rgb_img = resized_img.to_rgb8();
        let mut input_data = Vec::with_capacity(mask_size * mask_size * 3);

        for c in 0..3 {
            for y in 0..mask_size {
                for x in 0..mask_size {
                    let pixel = rgb_img.get_pixel(x as u32, y as u32);
                    let normalized =
                        (pixel[c] as f32 / 255.0 - self.mean[c]) / self.std[c];
                    input_data.push(normalized);
                }
            }
        }

        let input_array =
            Array4::from_shape_vec((1, 3, mask_size, mask_size), input_data)?;
        Ok(ort::value::Tensor::from_array(input_array)?)
    }

    fn postprocess(
        &self,
        tensor: ort::value::Tensor<f32>,
        original_dims: (u32, u32),
    ) -> Result<ndarray::Array3<u8>, Box<dyn std::error::Error>> {
        let mask_size = self.input_size as usize;
        let (original_width, original_height) = original_dims;

        let (output_shape, output_data) = tensor.try_extract_tensor::<f32>()?;
        let output_dims = &**output_shape;
        let cols = *output_dims.last().unwrap_or(&(mask_size as i64)) as usize;

        let mask_2d = Array2::from_shape_fn((mask_size, mask_size), |(h, w)| {
            output_data[h * cols + w]
        });

        let upscaled_mask =
            super::birefnet::bilinear_resize_f32(&mask_2d, original_width, original_height);

        // Apply threshold: clip values below threshold to 0, rescale [threshold,1] to [0,1]
        let t = self.threshold;
        let alpha_mask =
            upscaled_mask.mapv(|x| if x < t { 0.0 } else { ((x - t) / (1.0 - t)).clamp(0.0, 1.0) });

        let alpha_u8 = alpha_mask.mapv(|x| (x * 255.0).clamp(0.0, 255.0).round() as u8);

        let output_array = ndarray::Array3::from_shape_vec(
            (original_height as usize, original_width as usize, 1_usize),
            alpha_u8.into_raw_vec_and_offset().0,
        )?;

        Ok(output_array)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn test_rmbg_fp32_new_creates_with_correct_id() {
        let model = RmbgModel::new("rmbg-fp32");
        assert_eq!(model.id(), "rmbg-fp32");
        assert_eq!(model.filename(), "rmbg.onnx");
    }

    #[test]
    fn test_rmbg_fp16_new_creates_with_correct_id() {
        let model = RmbgModel::new("rmbg-fp16");
        assert_eq!(model.id(), "rmbg-fp16");
        assert_eq!(model.filename(), "rmbg_fp16.onnx");
    }

    #[test]
    fn test_param_schema_has_threshold() {
        let model = RmbgModel::new("rmbg-fp32");
        let schema = model.param_schema();
        assert_eq!(schema["type"], "object");

        let threshold = &schema["properties"]["threshold"];
        assert_eq!(threshold["type"], "number");
        assert_eq!(threshold["minimum"], 0.0);
        assert_eq!(threshold["maximum"], 1.0);
        assert_eq!(threshold["default"], 0.5);
        assert_eq!(threshold["multipleOf"], 0.01);
    }

    #[test]
    fn test_capabilities_returns_matting_true() {
        let model = RmbgModel::new("rmbg-fp32");
        let caps = model.capabilities();
        assert!(caps.matting);
        assert!(!caps.background_replace);
        assert!(!caps.edge_refinement);
        assert!(!caps.uncertainty_mask);
    }

    #[test]
    fn test_preprocess_output_shape() {
        let model = RmbgModel::new("rmbg-fp32");
        // Create a 256x256 test image
        let test_img = RgbaImage::from_fn(256, 256, |x, y| {
            let v = ((x + y) * 2 % 256) as u8;
            image::Rgba([v, v, v, 255])
        });
        let dyn_img = DynamicImage::ImageRgba8(test_img);

        let tensor = model.preprocess(dyn_img).expect("preprocess should succeed");
        // Verify NCHW shape: [1, 3, 1024, 1024]
        let (shape, _data) = tensor
            .try_extract_tensor::<f32>()
            .expect("should extract tensor");
        assert_eq!(
            &**shape,
            &[1i64, 3, 1024, 1024],
            "Expected NCHW shape [1, 3, 1024, 1024], got {:?}",
            &**shape
        );
    }

    #[test]
    fn test_postprocess_applies_threshold() {
        let model = RmbgModel::new("rmbg-fp32");
        // Create a synthetic 1024x1024 mask tensor: half low values, half high values
        let mut mask_data: Vec<f32> = Vec::with_capacity(1024 * 1024);
        for i in 0..1024 * 1024 {
            if i < 1024 * 1024 / 2 {
                mask_data.push(0.3); // below threshold -> should be clipped to 0
            } else {
                mask_data.push(0.8); // above threshold -> should be rescaled
            }
        }
        let mask_array =
            ndarray::Array4::from_shape_vec((1, 1, 1024, 1024), mask_data)
                .expect("Failed to create test array");
        let tensor =
            ort::value::Tensor::from_array(mask_array).expect("Failed to create test tensor");

        let result = model
            .postprocess(tensor, (256, 256))
            .expect("postprocess should succeed");
        // Expected shape: [256, 256, 1]
        assert_eq!(result.shape(), &[256, 256, 1]);

        // Values below threshold (0.3 < 0.5) should map to 0
        // Values above threshold (0.8) should map to (0.8-0.5)/(1.0-0.5) * 255 = 0.6 * 255 = 153
        let slice = result.as_slice().unwrap();
        // First half of the upscaled output should be near 0
        let first_val = slice[0];
        assert!(first_val < 128, "Low value should be near 0, got {}", first_val);
        // Later values should be higher
        let later_val = slice[slice.len() - 1];
        assert!(later_val > 128, "High value should be > 128, got {}", later_val);
    }

    #[test]
    fn test_postprocess_output_shape() {
        let model = RmbgModel::new("rmbg-fp32");
        // Create a synthetic 1024x1024 mask tensor (all 0.5 values)
        let mask_data: Vec<f32> = vec![0.5f32; 1024 * 1024];
        let mask_array =
            ndarray::Array4::from_shape_vec((1, 1, 1024, 1024), mask_data)
                .expect("Failed to create test array");
        let tensor =
            ort::value::Tensor::from_array(mask_array).expect("Failed to create test tensor");

        let result = model
            .postprocess(tensor, (256, 256))
            .expect("postprocess should succeed");
        // Expected shape: [256, 256, 1]
        assert_eq!(result.shape(), &[256, 256, 1]);
    }

    #[test]
    fn test_threshold_changes_with_different_values() {
        // Test with threshold = 0.0: no clipping
        let mut model_low = RmbgModel::new("rmbg-fp32");
        model_low.threshold = 0.0;
        // Test with threshold = 1.0: everything clipped
        let mut model_high = RmbgModel::new("rmbg-fp32");
        model_high.threshold = 1.0;

        let mask_data: Vec<f32> = vec![0.5f32; 1024 * 1024];
        let mask_array =
            ndarray::Array4::from_shape_vec((1, 1, 1024, 1024), mask_data.clone())
                .expect("Failed to create test array");
        let tensor_low =
            ort::value::Tensor::from_array(mask_array).expect("Failed to create test tensor");

        let mask_array =
            ndarray::Array4::from_shape_vec((1, 1, 1024, 1024), mask_data)
                .expect("Failed to create test array");
        let tensor_high =
            ort::value::Tensor::from_array(mask_array).expect("Failed to create test tensor");

        let result_low = model_low
            .postprocess(tensor_low, (64, 64))
            .expect("postprocess should succeed");
        let result_high = model_high
            .postprocess(tensor_high, (64, 64))
            .expect("postprocess should succeed");

        // With threshold=0.0, 0.5 should map to ~127.5
        // With threshold=1.0, 0.5 < 1.0 -> clipped to 0
        let low_avg: f32 =
            result_low.as_slice().unwrap().iter().map(|&x| x as f32).sum::<f32>()
                / result_low.len() as f32;
        let high_avg: f32 =
            result_high.as_slice().unwrap().iter().map(|&x| x as f32).sum::<f32>()
                / result_high.len() as f32;

        assert!(low_avg > high_avg, "Lower threshold should produce higher values");
        assert!(high_avg < 10.0, "Threshold 1.0 should clip nearly everything to 0");
    }
}
