use image::{imageops, DynamicImage, ImageBuffer};
use ndarray::{Array2, Array3, Array4, Axis};
use once_cell::sync::Lazy;
use std::path::PathBuf;
use std::sync::Mutex;
use tract_onnx::prelude::*;

use super::session::{BaseSession, SessionError};

static BIREFNET_SESSION: Lazy<Mutex<Option<BirefnetSession>>> =
    Lazy::new(|| Mutex::new(None));

pub struct BirefnetSession {
    input_size: u32,
    mean: [f32; 3],
    std: [f32; 3],
    base_session: BaseSession,
}

impl BirefnetSession {
    pub fn new(model_path: PathBuf, _provider: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let session_options = super::session::SessionOptions::new()
            .with_providers(vec!["cpu".to_string()])
            .build()?;

        let base_session = BaseSession::new(false, session_options, model_path)?;

        Ok(Self {
            input_size: 1024,
            mean: [0.485, 0.456, 0.406],
            std: [0.229, 0.224, 0.225],
            base_session,
        })
    }

    pub fn init(model_path: PathBuf, provider: &str) -> Result<(), Box<dyn std::error::Error>> {
        let session = Self::new(model_path, provider)?;
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
    pub fn run(&self,
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

        for y in 0..mask_size {
            for x in 0..mask_size {
                let pixel = rgb_img.get_pixel(x as u32, y as u32);
                for c in 0..3 {
                    let normalized = (pixel[c] as f32 / 255.0 - session.mean[c]) / session.std[c];
                    input_data.push(normalized);
                }
            }
        }

        // 3. 构建输入 Tensor [1, 3, 1024, 1024] (NCHW)
        let input_array = Array4::from_shape_vec((1, 3, mask_size, mask_size), input_data)?;
        let input_tensor: Tensor = input_array.into();

        // 4. 运行推理
        let plan = &session.base_session.inner_session;
        let outputs = plan.run(tvec!(input_tensor.into()))?;

        // 5. 提取输出 (sigmoid 后已经是 [0,1] 范围)
        let output = outputs[0].to_array_view::<f32>()?;
        let output_shape = output.shape();
        log::info!("输出形状: {:?}", output_shape);

        // 处理不同输出格式: [1,1,1024,1024] 或 [1,1024,1024]
        let mask_2d = if output_shape.len() == 4 {
            // [1, 1, H, W] -> [H, W]
            Array2::from_shape_fn((mask_size, mask_size), |(h, w)| {
                output[[0, 0, h, w]]
            })
        } else if output_shape.len() == 3 {
            // [1, H, W] -> [H, W]
            Array2::from_shape_fn((mask_size, mask_size), |(h, w)| {
                output[[0, h, w]]
            })
        } else {
            Array2::from_shape_fn((mask_size, mask_size), |(h, w)| {
                output[[h, w]]
            })
        };

        // 6. 转换为 u8
        let alpha_mask = mask_2d.mapv(|x| (x * 255.0).clamp(0.0, 255.0).round() as u8);

        // 7. 调整回原始尺寸
        let alpha_image = DynamicImage::ImageLuma8(
            ImageBuffer::from_vec(
                mask_size as u32,
                mask_size as u32,
                alpha_mask.into_raw_vec(),
            )
            .unwrap_or_default(),
        );

        let output_image = alpha_image.resize_exact(
            original_width,
            original_height,
            imageops::Lanczos3,
        );

        let output_array = Array3::from_shape_vec(
            (original_height as usize, original_width as usize, 1_usize),
            output_image.to_luma8().into_raw(),
        )?;

        Ok(output_array)
    }

    pub fn post_process(
        &self,
        output: Array3<u8>,
        original_image: DynamicImage,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let shape = [
            original_image.height() as usize,
            original_image.width() as usize,
            4,
        ];
        let mut output_img = Array3::from_shape_vec(
            shape,
            original_image.to_rgba8().to_vec(),
        )?;

        output_img
            .index_axis_mut(Axis(2), 3)
            .assign(&output.remove_axis(Axis(2)));

        // 转换为 PNG bytes
        let (height, width, _) = output_img.dim();
        let img_buffer = output_img.into_raw_vec();

        let mut png_bytes = Vec::new();
        {
            let cursor = std::io::Cursor::new(&mut png_bytes);
            let encoder = image::codecs::png::PngEncoder::new(cursor);
            encoder.write_image(
                &img_buffer,
                width as u32,
                height as u32,
                image::ExtendedColorType::Rgba8,
            )?;
        }

        Ok(png_bytes)
    }
}
