use image::{imageops, DynamicImage};
use ndarray::{Array2, Array3, Array4};
use ort::session::{builder::GraphOptimizationLevel, Session};
use std::path::PathBuf;

use crate::commands::ModelSource;
use crate::models::PluginCapabilities;
use crate::models::MattingModel;

pub struct BirefnetModel {
    input_size: u32,
    mean: [f32; 3],
    std: [f32; 3],
    session: Option<Session>,
}

impl BirefnetModel {
    pub fn new() -> Self {
        Self {
            input_size: 1024,
            mean: [0.485, 0.456, 0.406],
            std: [0.229, 0.224, 0.225],
            session: None,
        }
    }
}

impl MattingModel for BirefnetModel {
    fn id(&self) -> &str {
        "birefnet"
    }

    fn name(&self) -> &str {
        "BiRefNet"
    }

    fn description(&self) -> &str {
        "通用高精度抠图模型，支持各类主体（人物、物体、动物等）"
    }

    fn init(&mut self, model_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level1)? // ORT_ENABLE_BASIC: 常量折叠、节点消除等安全优化，不改变模型行为
            .with_intra_threads(1)?
            .commit_from_file(&model_path)?;
        self.session = Some(session);
        Ok(())
    }

    fn is_loaded(&self) -> bool {
        self.session.is_some()
    }

    /// Refactored to delegate to preprocess() and postprocess(). Per D-03.
    fn infer(&mut self, original_image: DynamicImage, _params: serde_json::Value) -> Result<Array3<u8>, Box<dyn std::error::Error>> {
        // Take ownership of session to avoid borrow conflicts with preprocess/postprocess
        let mut session = self.session.take().ok_or("Session not initialized")?;

        let original_width = original_image.width();
        let original_height = original_image.height();

        log::info!("原始图片尺寸: {}x{}", original_width, original_height);

        let input_tensor = self.preprocess(original_image)?;

        // Scoped block: extract output data and drop SessionOutputs borrow before reassigning session
        let (output_shape, output_data_vec) = {
            let mut outputs = session.run(ort::inputs!["input_image" => input_tensor])?;
            let output_value = outputs.remove("output_image").ok_or("Missing output_image in model output")?;
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
        "birefnet.onnx"
    }

    fn sources(&self) -> Vec<ModelSource> {
        vec![
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
        ]
    }

    /// BiRefNet has no tunable inference parameters. Per D-01.
    fn param_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    /// BiRefNet supports core matting (outputs transparent PNG) only. Per D-02.
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities {
            matting: true,
            background_replace: false,
            edge_refinement: false,
            uncertainty_mask: false,
        }
    }

    /// Preprocess: resize to input_size, RGB conversion, ImageNet normalization, NCHW tensor.
    /// Per D-03.
    fn preprocess(&self, image: DynamicImage) -> Result<ort::value::Tensor<f32>, Box<dyn std::error::Error>> {
        let mask_size = self.input_size as usize;

        let resized_img = image.resize_exact(
            self.input_size,
            self.input_size,
            imageops::Lanczos3,
        );

        let rgb_img = resized_img.to_rgb8();
        let mut input_data = Vec::with_capacity(mask_size * mask_size * 3);

        for c in 0..3 {
            for y in 0..mask_size {
                for x in 0..mask_size {
                    let pixel = rgb_img.get_pixel(x as u32, y as u32);
                    let normalized = (pixel[c] as f32 / 255.0 - self.mean[c]) / self.std[c];
                    input_data.push(normalized);
                }
            }
        }

        let input_array = Array4::from_shape_vec((1, 3, mask_size, mask_size), input_data)?;
        Ok(ort::value::Tensor::from_array(input_array)?)
    }

    /// Postprocess: extract f32 tensor, reshape to 2D mask, bilinear upscale, convert to u8.
    /// Per D-03.
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

        let upscaled_mask = bilinear_resize_f32(&mask_2d, original_width, original_height);

        let alpha_mask = upscaled_mask.mapv(|x| (x * 255.0).clamp(0.0, 255.0).round() as u8);

        let output_array = ndarray::Array3::from_shape_vec(
            (original_height as usize, original_width as usize, 1_usize),
            alpha_mask.into_raw_vec_and_offset().0,
        )?;

        Ok(output_array)
    }
}

/// Bilinear interpolation upscale of a 2D f32 mask.
pub(crate) fn bilinear_resize_f32(mask: &Array2<f32>, new_width: u32, new_height: u32) -> Array2<f32> {
    let old_h = mask.nrows();
    let old_w = mask.ncols();
    let nw = new_width as usize;
    let nh = new_height as usize;
    let mut result = Array2::zeros((nh, nw));

    let scale_x = old_w as f64 / nw as f64;
    let scale_y = old_h as f64 / nh as f64;

    for y in 0..nh {
        let src_y = y as f64 * scale_y;
        let y0 = (src_y.floor() as usize).min(old_h.saturating_sub(1));
        let y1 = (y0 + 1).min(old_h.saturating_sub(1));
        let dy = src_y - y0 as f64;

        for x in 0..nw {
            let src_x = x as f64 * scale_x;
            let x0 = (src_x.floor() as usize).min(old_w.saturating_sub(1));
            let x1 = (x0 + 1).min(old_w.saturating_sub(1));
            let dx = src_x - x0 as f64;

            let v = mask[[y0, x0]] as f64 * (1.0 - dx) * (1.0 - dy)
                + mask[[y0, x1]] as f64 * dx * (1.0 - dy)
                + mask[[y1, x0]] as f64 * (1.0 - dx) * dy
                + mask[[y1, x1]] as f64 * dx * dy;

            result[[y, x]] = v as f32;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;
    use std::path::PathBuf;

    #[test]
    fn test_param_schema_returns_empty_json_schema() {
        let model = BirefnetModel::new();
        let schema = model.param_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].as_object().unwrap().is_empty());
    }

    #[test]
    fn test_capabilities_returns_matting_true_only() {
        let model = BirefnetModel::new();
        let caps = model.capabilities();
        assert!(caps.matting);
        assert!(!caps.background_replace);
        assert!(!caps.edge_refinement);
        assert!(!caps.uncertainty_mask);
    }

    #[test]
    fn test_preprocess_output_shape() {
        let model = BirefnetModel::new();
        // Create a 256x256 test image
        let test_img = RgbaImage::from_fn(256, 256, |x, y| {
            let v = ((x + y) * 2 % 256) as u8;
            image::Rgba([v, v, v, 255])
        });
        let dyn_img = DynamicImage::ImageRgba8(test_img);

        let tensor = model.preprocess(dyn_img).expect("preprocess should succeed");
        // Verify NCHW shape: [1, 3, 1024, 1024]
        let (shape, _data) = tensor.try_extract_tensor::<f32>().expect("should extract tensor");
        assert_eq!(&**shape, &[1i64, 3, 1024, 1024],
            "Expected NCHW shape [1, 3, 1024, 1024], got {:?}", &**shape);
    }

    #[test]
    fn test_postprocess_output_shape() {
        let model = BirefnetModel::new();
        // Create a synthetic 1024x1024 mask tensor (all 0.5 values)
        let mask_data: Vec<f32> = vec![0.5f32; 1024 * 1024];
        let mask_array = ndarray::Array4::from_shape_vec((1, 1, 1024, 1024), mask_data)
            .expect("Failed to create test array");
        let tensor = ort::value::Tensor::from_array(mask_array)
            .expect("Failed to create test tensor");

        let result = model.postprocess(tensor, (256, 256))
            .expect("postprocess should succeed");
        // Expected shape: [256, 256, 1]
        assert_eq!(result.shape(), &[256, 256, 1],
            "Expected shape [256, 256, 1], got {:?}", result.shape());
    }

    #[test]
    fn test_preprocess_values_in_normalized_range() {
        let model = BirefnetModel::new();
        // Create a solid gray 128x128 image (R=128, G=128, B=128)
        let test_img = RgbaImage::from_fn(128, 128, |_, _| {
            image::Rgba([128, 128, 128, 255])
        });
        let dyn_img = DynamicImage::ImageRgba8(test_img);

        let tensor = model.preprocess(dyn_img).expect("preprocess should succeed");

        // Extract first pixel's normalized values
        let data = tensor.try_extract_tensor::<f32>().expect("should extract f32 data");
        let flat = data.1;
        // R channel: (128/255 - 0.485) / 0.229 ≈ (0.502 - 0.485) / 0.229 ≈ 0.074
        // G channel: (128/255 - 0.456) / 0.224 ≈ (0.502 - 0.456) / 0.224 ≈ 0.205
        // B channel: (128/255 - 0.406) / 0.225 ≈ (0.502 - 0.406) / 0.225 ≈ 0.427
        let r = flat[0]; // NCHW: first channel, first pixel
        let g = flat[1024 * 1024]; // second channel, first pixel
        let b = flat[2 * 1024 * 1024]; // third channel, first pixel

        // All values should be in normalized range (roughly -2.0 to 2.0 for ImageNet)
        assert!((-3.0..3.0).contains(&r), "R channel value {} out of expected range", r);
        assert!((-3.0..3.0).contains(&g), "G channel value {} out of expected range", g);
        assert!((-3.0..3.0).contains(&b), "B channel value {} out of expected range", b);
    }

    #[test]
    fn test_model_loading() {
        let home = std::env::var("HOME").unwrap();
        let model_path = PathBuf::from(format!(
            "{}/Library/Application Support/cn.mopng.desktop/models/birefnet.onnx",
            home
        ));

        if !model_path.exists() {
            eprintln!("跳过测试: 模型文件不存在 ({:?})，请先下载模型", model_path);
            return;
        }
        log::info!(
            "模型文件大小: {} MB",
            model_path.metadata().unwrap().len() / 1_048_576
        );

        // 1. Initialize model
        let mut model = BirefnetModel::new();
        model.init(model_path.clone()).expect("模型初始化失败");

        // 2. Create test image (256x256 gradient)
        let test_img = RgbaImage::from_fn(256, 256, |x, y| {
            let v = ((x + y) * 2 % 256) as u8;
            image::Rgba([v, v, v, 255])
        });
        let dyn_img = DynamicImage::ImageRgba8(test_img);

        // 3. Run inference
        let output = model.infer(dyn_img, serde_json::json!({})).expect("推理失败");
        log::info!("推理输出形状: {:?}", output.dim());

        // 4. Verify output dimensions
        assert_eq!(output.shape(), &[256, 256, 1], "输出形状不匹配");

        // 5. Verify output is not empty (u8 values are always in range)
        assert!(!output.is_empty(), "输出不应为空");

        log::info!("模型加载和推理测试通过!");
    }
}
