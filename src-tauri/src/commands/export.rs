use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use base64::prelude::*;
use image::ImageEncoder;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportSettings {
    pub format: String,
    pub quality: u8,
    pub include_original: bool,
    pub output_folder: Option<String>,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WatermarkSettings {
    pub enabled: bool,
    pub text: String,
    pub position: String,
    pub font_size: u32,
    pub color: String,
    pub opacity: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BgSettings {
    pub r#type: String,
    pub color: String,
    pub image_path: Option<String>,
}

/// 导出图片
#[tauri::command]
pub fn export_image(
    image_path: String,
    original_path: String,
    settings: ExportSettings,
) -> Result<String, String> {
    let input_path = PathBuf::from(&image_path);
    if !input_path.exists() {
        return Err(format!("结果文件不存在: {}", image_path));
    }

    // 确定输出目录
    let output_dir = if let Some(folder) = settings.output_folder {
        PathBuf::from(folder)
    } else {
        PathBuf::from(&original_path)
            .parent()
            .unwrap_or(PathBuf::from(".").as_path())
            .join("output")
    };
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    // 确定输出文件名
    let stem_buf = PathBuf::from(&settings.file_name);
    let stem = stem_buf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("result");

    let output_path = match settings.format.as_str() {
        "jpg" | "jpeg" => {
            let path = output_dir.join(format!("{}_matte.jpg", stem));
            // 读取 PNG 并转换为 JPG（白色背景）
            let img = image::io::Reader::open(&input_path)
                .map_err(|e| e.to_string())?
                .decode()
                .map_err(|e| e.to_string())?;

            let rgb_img = img.to_rgb8();
            let mut output_file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
            let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(
                &mut output_file,
                settings.quality,
            );
            encoder.encode(&rgb_img, rgb_img.width(), rgb_img.height(), image::ColorType::Rgb8)
                .map_err(|e| e.to_string())?;

            path
        }
        "webp" => {
            let path = output_dir.join(format!("{}_matte.webp", stem));
            std::fs::copy(&input_path, &path).map_err(|e| e.to_string())?;
            path
        }
        _ => {
            // 默认 PNG
            let path = output_dir.join(format!("{}_matte.png", stem));
            std::fs::copy(&input_path, &path).map_err(|e| e.to_string())?;
            path
        }
    };

    log::info!("导出完成: {:?}", output_path);
    Ok(output_path.to_string_lossy().to_string())
}

/// 预览合成图（带背景和水印）
#[tauri::command]
pub fn preview_composite(
    result_path: String,
    original_width: u32,
    original_height: u32,
    bg_settings: BgSettings,
    watermark_settings: WatermarkSettings,
) -> Result<String, String> {
    let result = image::io::Reader::open(&result_path)
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;

    let rgba = result.to_rgba8();
    let (width, height) = rgba.dimensions();

    // 创建背景
    let mut composite = match bg_settings.r#type.as_str() {
        "color" => {
            let mut img = image::RgbaImage::new(width, height);
            let color = parse_color(&bg_settings.color);
            for pixel in img.pixels_mut() {
                *pixel = color;
            }
            img
        }
        "image" => {
            if let Some(bg_path) = bg_settings.image_path {
                if let Ok(bg) = image::io::Reader::open(&bg_path)
                    .and_then(|r| Ok(r.decode().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?)) {
                    bg.resize_exact(width, height, image::imageops::Lanczos3).to_rgba8()
                } else {
                    image::RgbaImage::new(width, height)
                }
            } else {
                image::RgbaImage::new(width, height)
            }
        }
        _ => {
            // transparent - 棋盘格背景
            create_checkerboard(width, height)
        }
    };

    // 合成抠图结果
    for (x, y, pixel) in rgba.enumerate_pixels() {
        let alpha = pixel[3] as f32 / 255.0;
        if alpha > 0.0 {
            let bg_pixel = composite.get_pixel(x, y);
            let r = (pixel[0] as f32 * alpha + bg_pixel[0] as f32 * (1.0 - alpha)) as u8;
            let g = (pixel[1] as f32 * alpha + bg_pixel[1] as f32 * (1.0 - alpha)) as u8;
            let b = (pixel[2] as f32 * alpha + bg_pixel[2] as f32 * (1.0 - alpha)) as u8;
            composite.put_pixel(x, y, image::Rgba([r, g, b, 255]));
        }
    }

    // TODO: 添加水印（需要字体渲染库）

    // 编码为 base64
    let mut png_bytes = Vec::new();
    {
        let cursor = std::io::Cursor::new(&mut png_bytes);
        let encoder = image::codecs::png::PngEncoder::new(cursor);
        encoder.write_image(
            &composite.into_raw(),
            width,
            height,
            image::ColorType::Rgba8,
        ).map_err(|e| e.to_string())?;
    }

    Ok(BASE64_STANDARD.encode(&png_bytes))
}

/// 创建棋盘格背景
fn create_checkerboard(width: u32, height: u32) -> image::RgbaImage {
    let mut img = image::RgbaImage::new(width, height);
    let square_size = 16;

    for y in 0..height {
        for x in 0..width {
            let cx = x / square_size;
            let cy = y / square_size;
            let color = if (cx + cy) % 2 == 0 {
                image::Rgba([255, 255, 255, 255])
            } else {
                image::Rgba([220, 220, 220, 255])
            };
            img.put_pixel(x, y, color);
        }
    }

    img
}

/// 解析颜色字符串
fn parse_color(color: &str) -> image::Rgba<u8> {
    let hex = color.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    image::Rgba([r, g, b, 255])
}
