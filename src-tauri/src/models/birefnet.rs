use image::{imageops, DynamicImage, ImageBuffer, ImageEncoder};
use ndarray::{Array3, Axis};
use once_cell::sync::Lazy;
use ort::value::Tensor;
use std::path::PathBuf;
use std::sync::Mutex;

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
    pub fn new(model_path: PathBuf, provider: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let session_options = super::session::SessionOptions::new()
            .with_providers(vec![provider.to_string()])
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
    pub fn run(&self, original_image: DynamicImage) -> Result<Array3<u8>, Box<dyn std::error::Error>> {
        let mut lock = BIREFNET_SESSION.lock().map_err(|_| "Mutex poisoned")?;
        let session = lock.as_mut().ok_or("Session not initialized")?;

        let mask_size = session.input_size;
        let original_width = original_image.width();
        let original_height = original_image.height();

        log::info!("原始图片尺寸: {}x{}", original_width, original_height);

        let resized_img = original_image.resize_exact(mask_size, mask_size, imageops::Lanczos3);
        let image_buffer_array = Array3::<u8>::from_shape_vec(
            (mask_size as usize, mask_size as usize, 3_usize),
            resized_img.to_rgb8().to_vec(),
        ).map_err(|_| SessionError::ImageProcessingError)?;

        let mut input_array = image_buffer_array.mapv(|x| x as f32 / 255.0);

        // 归一化
        for c in 0..3 {
            input_array.index_axis_mut(Axis(2), c).map_mut(|pixel| {
                *pixel = (*pixel - session.mean[c]) / session.std[c];
            });
        }

        let input_tensor = input_array.permuted_axes([2, 0, 1]).insert_axis(Axis(0));

        let model = &mut session.base_session.inner_session;

        let ort_input = Tensor::from_array(input_tensor)?;
        let ort_inputs = ort::inputs![ort_input];
        let ort_outputs = model.run(ort_inputs)?;

        let output_array_view = ort_outputs[0].try_extract_array::<f32>()?;
        let output_arr = output_array_view.to_owned();
        let alpha_mask_raw = output_arr.into_shape((1, 1, mask_size as usize, mask_size as usize))?
            .remove_axis(Axis(0));

        let alpha_mask = alpha_mask_raw.permuted_axes([1, 2, 0]);
        let alpha_mask = alpha_mask.mapv(|x| (x * 255.0).round() as u8);
        let (mask_width, mask_height, _) = alpha_mask.dim();

        let alpha_image = DynamicImage::ImageLuma8(
            ImageBuffer::from_vec(
                mask_width as u32,
                mask_height as u32,
                alpha_mask.into_raw_vec(),
            )
            .unwrap_or_default(),
        );

        let output_image = alpha_image.resize_exact(original_width, original_height, imageops::Lanczos3);
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
