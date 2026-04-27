use image::{imageops, DynamicImage};
use ndarray::{Array2, Array3, Array4};
use once_cell::sync::Lazy;
use ort::session::{builder::GraphOptimizationLevel, Session};
use std::path::PathBuf;
use std::sync::Mutex;

static BIREFNET_SESSION: Lazy<Mutex<Option<BirefnetSession>>> =
    Lazy::new(|| Mutex::new(None));

pub struct BirefnetSession {
    input_size: u32,
    mean: [f32; 3],
    std: [f32; 3],
    session: Session,
}

impl BirefnetSession {
    pub fn new(model_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Disable)?
            .with_intra_threads(1)?
            .commit_from_file(&model_path)?;

        Ok(Self {
            input_size: 1024,
            mean: [0.485, 0.456, 0.406],
            std: [0.229, 0.224, 0.225],
            session,
        })
    }

    pub fn init(model_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let session = Self::new(model_path)?;
        let mut lock = BIREFNET_SESSION.lock().map_err(|_| "Mutex poisoned")?;
        *lock = Some(session);
        Ok(())
    }

    pub fn get() -> Option<BirefnetSessionGuard> {
        let lock = BIREFNET_SESSION.lock().ok()?;
        if lock.is_some() {
            Some(BirefnetSessionGuard)
        } else {
            None
        }
    }
}

pub struct BirefnetSessionGuard;

impl BirefnetSessionGuard {
    pub fn run(
        &self,
        original_image: DynamicImage,
    ) -> Result<Array3<u8>, Box<dyn std::error::Error>> {
        let mut lock = BIREFNET_SESSION.lock().map_err(|_| "Mutex poisoned")?;
        let session = lock.as_mut().ok_or("Session not initialized")?;

        let mask_size = session.input_size as usize;
        let original_width = original_image.width();
        let original_height = original_image.height();

        log::info!("原始图片尺寸: {}x{}", original_width, original_height);

        // 1. 调整图片尺寸
        let resized_img = original_image.resize_exact(
            mask_size as u32,
            mask_size as u32,
            imageops::Lanczos3,
        );

        // 2. 转换为 RGB 数组并归一化
        let rgb_img = resized_img.to_rgb8();
        let mut input_data = Vec::with_capacity(mask_size * mask_size * 3);

        // NCHW 格式: 先填充所有 R 通道, 再 G, 再 B
        for c in 0..3 {
            for y in 0..mask_size {
                for x in 0..mask_size {
                    let pixel = rgb_img.get_pixel(x as u32, y as u32);
                    let normalized = (pixel[c] as f32 / 255.0 - session.mean[c]) / session.std[c];
                    input_data.push(normalized);
                }
            }
        }

        // 3. 构建输入 Tensor [1, 3, 1024, 1024] (NCHW)
        let input_array = Array4::from_shape_vec(
            (1, 3, mask_size, mask_size),
            input_data,
        )?;

        // 4. 运行推理 (ort)
        let input_tensor = ort::value::Tensor::from_array(input_array)?;
        let outputs = session.session.run(
            ort::inputs!["input_image" => input_tensor]
        )?;

        // 5. 提取输出
        let (output_shape, output_data) = outputs["output_image"].try_extract_tensor::<f32>()?;
        let output_dims = &**output_shape;
        log::info!("输出形状: {:?}", output_dims);

        // NCHW 格式，最后一个维度是宽度
        let cols = *output_dims.last().unwrap_or(&(mask_size as i64)) as usize;

        let mask_2d = Array2::from_shape_fn((mask_size, mask_size), |(h, w)| {
            output_data[h * cols + w]
        });

        // Upscale the f32 mask to original dimensions using bilinear interpolation.
        // Keeping values as f32 during upscaling produces smooth anti-aliased edges;
        // quantizing to u8 first (like Lanczos3 on u8) creates stair-step artifacts.
        let upscaled_mask = bilinear_resize_f32(&mask_2d, original_width, original_height);

        // Convert to u8 only at final resolution
        let alpha_mask = upscaled_mask.mapv(|x| (x * 255.0).clamp(0.0, 255.0).round() as u8);

        let output_array = Array3::from_shape_vec(
            (original_height as usize, original_width as usize, 1_usize),
            alpha_mask.into_raw_vec_and_offset().0,
        )?;

        Ok(output_array)
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

    #[test]
    fn test_model_loading() {
        // Model path: ~/Library/Application Support/cn.mopng.desktop/models/model_fp16.onnx
        let home = std::env::var("HOME").unwrap();
        let model_path = PathBuf::from(format!(
            "{}/Library/Application Support/cn.mopng.desktop/models/model_fp16.onnx",
            home
        ));

        assert!(model_path.exists(), "模型文件不存在: {:?}", model_path);
        log::info!("模型文件大小: {} MB", model_path.metadata().unwrap().len() / 1_048_576);

        // 1. 初始化模型
        BirefnetSession::init(model_path.clone())
            .expect("模型初始化失败");

        // 2. 获取 guard 运行推理
        let guard = BirefnetSession::get().expect("无法获取模型 session");

        // 3. 创建一张测试图片 (256x256 灰色渐变图)
        let test_img = RgbaImage::from_fn(256, 256, |x, y| {
            let v = ((x + y) * 2 % 256) as u8;
            image::Rgba([v, v, v, 255])
        });
        let dyn_img = DynamicImage::ImageRgba8(test_img);

        // 4. 运行推理
        let output = guard.run(dyn_img)
            .expect("推理失败");
        log::info!("推理输出形状: {:?}", output.dim());

        // 4. 验证输出维度
        assert_eq!(output.shape(), &[256, 256, 1], "输出形状不匹配");

        // 5. 验证输出值范围 (mask 应该在 0-255 之间)
        for &v in output.iter() {
            assert!(v <= 255, "输出值超出范围: {}", v);
        }

        log::info!("模型加载和推理测试通过!");
    }
}
