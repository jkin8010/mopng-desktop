use image::{imageops, DynamicImage};
use ndarray::{Array2, Array3, Array4};
use ort::session::{builder::GraphOptimizationLevel, Session};
use std::path::PathBuf;

use crate::commands::ModelSource;
use crate::models::PluginCapabilities;
use crate::models::registry::ModelDescriptor;
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

    fn infer(&mut self, original_image: DynamicImage) -> Result<Array3<u8>, Box<dyn std::error::Error>> {
        let session = self.session.as_mut().ok_or("Session not initialized")?;

        let mask_size = self.input_size as usize;
        let original_width = original_image.width();
        let original_height = original_image.height();

        log::info!("原始图片尺寸: {}x{}", original_width, original_height);

        // 1. Resize image
        let resized_img = original_image.resize_exact(
            mask_size as u32,
            mask_size as u32,
            imageops::Lanczos3,
        );

        // 2. Convert to RGB and normalize (NCHW)
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

        // 3. Build input tensor [1, 3, 1024, 1024] (NCHW)
        let input_array = Array4::from_shape_vec((1, 3, mask_size, mask_size), input_data)?;

        // 4. Run inference (ort)
        let input_tensor = ort::value::Tensor::from_array(input_array)?;
        let outputs = session.run(ort::inputs!["input_image" => input_tensor])?;

        // 5. Extract output
        let (output_shape, output_data) = outputs["output_image"].try_extract_tensor::<f32>()?;
        let output_dims = &**output_shape;
        log::info!("输出形状: {:?}", output_dims);

        let cols = *output_dims.last().unwrap_or(&(mask_size as i64)) as usize;

        let mask_2d = Array2::from_shape_fn((mask_size, mask_size), |(h, w)| {
            output_data[h * cols + w]
        });

        // Upscale the f32 mask to original dimensions using bilinear interpolation
        let upscaled_mask = bilinear_resize_f32(&mask_2d, original_width, original_height);

        // Convert to u8 at final resolution
        let alpha_mask = upscaled_mask.mapv(|x| (x * 255.0).clamp(0.0, 255.0).round() as u8);

        let output_array = Array3::from_shape_vec(
            (original_height as usize, original_width as usize, 1_usize),
            alpha_mask.into_raw_vec_and_offset().0,
        )?;

        Ok(output_array)
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

    // Task 1 stubs — real implementations in Task 2
    fn preprocess(&self, _image: DynamicImage) -> Result<ort::value::Tensor<f32>, Box<dyn std::error::Error>> {
        Err("not implemented".into())
    }

    fn postprocess(
        &self,
        _tensor: ort::value::Tensor<f32>,
        _original_dims: (u32, u32),
    ) -> Result<ndarray::Array3<u8>, Box<dyn std::error::Error>> {
        Err("not implemented".into())
    }
}

pub fn descriptor() -> ModelDescriptor {
    ModelDescriptor {
        id: "birefnet".to_string(),
        name: "BiRefNet".to_string(),
        description: "通用高精度抠图模型，支持各类主体（人物、物体、动物等）".to_string(),
        filename: "birefnet.onnx".to_string(),
        checksum: Some("58f621f00f5d756097615970a88a791584600dcf7c45b18a0a6267535a1ebd3c".to_string()),
        param_schema: serde_json::json!({}),
        capabilities: PluginCapabilities {
            matting: true,
            background_replace: false,
            edge_refinement: false,
            uncertainty_mask: false,
        },
        input_size: None,
        mean: None,
        std: None,
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
    }
}

/// Bilinear interpolation upscale of a 2D f32 mask.
fn bilinear_resize_f32(mask: &Array2<f32>, new_width: u32, new_height: u32) -> Array2<f32> {
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
        let output = model.infer(dyn_img).expect("推理失败");
        log::info!("推理输出形状: {:?}", output.dim());

        // 4. Verify output dimensions
        assert_eq!(output.shape(), &[256, 256, 1], "输出形状不匹配");

        // 5. Verify output is not empty (u8 values are always in range)
        assert!(!output.is_empty(), "输出不应为空");

        log::info!("模型加载和推理测试通过!");
    }
}
