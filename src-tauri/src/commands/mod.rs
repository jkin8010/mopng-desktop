use std::fs;
use std::path::Path;

use image::{imageops, ImageReader, ImageFormat};
use image::GenericImageView;
use tauri::{command, Manager};
use tauri_plugin_dialog::FilePath;
pub use self::download::*;
pub use self::file::*;

use crate::models::{MattingSettings, ProcessParams, ProcessResult, ThumbnailParams};

pub mod download;
pub mod file;

/// Process an image using BiRefNet ONNX model
#[command]
pub async fn process_image(
    params: ProcessParams,
    app: tauri::AppHandle,
) -> Result<ProcessResult, String> {
    let start_time = std::time::Instant::now();

    // Use Downloads folder for output
    let output_dir = app
        .path()
        .download_dir()
        .map_err(|e| format!("Failed to get downloads dir: {}", e))?
        .join("mopng_output");
    fs::create_dir_all(&output_dir).map_err(|e| format!("Failed to create output dir: {}", e))?;

    // Load and preprocess the image
    let img_path = Path::new(&params.file_path);
    let img = ImageReader::open(img_path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    // Run inference on a blocking thread so the async IPC stays responsive
    let inference_img = img.clone();
    let mask_u8 = tokio::task::spawn_blocking(move || {
        crate::models::registry::infer(inference_img)
    })
    .await
    .map_err(|e| format!("Inference thread failed: {}", e))??;

    // Convert Array3<u8> (H, W, 1) to Array2<f32> for downstream compositing
    let (h, w, _) = mask_u8.dim();
    let mask = mask_u8
        .remove_axis(ndarray::Axis(2))
        .mapv(|v| v as f32 / 255.0);

    println!("[Inference] Inference complete, mask shape: {}x{}", h, w);

    // Generate mask data URL for frontend real-time compositing
    let mask_data_url = Some(generate_mask_data_url(&mask)?);

    // Apply mask to create the output image
    let output = apply_mask(&img, &mask, &params.settings)?;

    // Resize to target dimensions if specified by size template
    let output = resize_to_target(output, &params.settings);

    // Determine output path and format
    let ext = match params.settings.output_format.as_str() {
        "jpg" | "jpeg" => "jpg",
        "webp" => "webp",
        _ => "png",
    };

    let output_filename = format!(
        "{}_matting.{}",
        img_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output"),
        ext
    );

    let output_path = output_dir.join(&output_filename);

    println!(
        "[process_image] bg_type={}, bg_color={:?}, output_format={}, quality={}",
        params.settings.bg_type, params.settings.bg_color, params.settings.output_format, params.settings.quality,
    );

    // Save with format-specific encoder for quality-controlled compression
    let file = std::fs::File::create(&output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    match ext {
        "jpg" => {
            let jpg_output = apply_bg_type(&output, &params.settings, "jpg")?;
            let quality = params.settings.quality.clamp(10, 100) as u8;
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(file, quality);
            jpg_output
                .write_with_encoder(encoder)
                .map_err(|e| format!("Failed to save JPEG: {}", e))?;
        }
        "webp" => {
            // WebP lossless (smaller than PNG, ~25-35% better compression)
            let encoder = image::codecs::webp::WebPEncoder::new_lossless(file);
            output
                .write_with_encoder(encoder)
                .map_err(|e| format!("Failed to save WebP: {}", e))?;
        }
        _ => {
            // PNG: use maximum compression to reduce file size
            let encoder = image::codecs::png::PngEncoder::new_with_quality(
                file,
                image::codecs::png::CompressionType::Best,
                image::codecs::png::FilterType::Adaptive,
            );
            output
                .write_with_encoder(encoder)
                .map_err(|e| format!("Failed to save PNG: {}", e))?;
        }
    }

    // Generate transparent preview for canvas display (no background baked in)
    // Canvas background div handles visual feedback for different bg types
    let preview_filename = format!(
        "{}_preview.png",
        img_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output"),
    );
    let preview_path = output_dir.join(&preview_filename);

    let preview = create_preview(&output)?;
    preview
        .save_with_format(&preview_path, ImageFormat::Png)
        .map_err(|e| format!("Failed to save preview: {}", e))?;

    let file_size = fs::metadata(&output_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let elapsed = start_time.elapsed();
    println!("Processed {} in {:?}", params.file_path, elapsed);

    Ok(ProcessResult {
        output_path: output_path.to_string_lossy().to_string(),
        width: output.width(),
        height: output.height(),
        format: ext.to_string(),
        file_size,
        preview_path: preview_path.to_string_lossy().to_string(),
        mask_data_url,
    })
}

/// Generate a thumbnail for the given image
#[command]
pub fn generate_thumbnail(params: ThumbnailParams) -> Result<String, String> {
    let img = ImageReader::open(&params.path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let (width, height) = (img.width(), img.height());
    let max_size = params.max_size;

    let (new_width, new_height) = if width > height {
        let ratio = max_size as f32 / width as f32;
        (max_size, (height as f32 * ratio) as u32)
    } else {
        let ratio = max_size as f32 / height as f32;
        ((width as f32 * ratio) as u32, max_size)
    };

    let thumbnail = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);

    // Convert to base64 for display
    let mut bytes: Vec<u8> = Vec::new();
    thumbnail
        .write_to(&mut std::io::Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|e| format!("Failed to encode thumbnail: {}", e))?;

    let base64 = base64::engine::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &bytes,
    );
    Ok(format!("data:image/png;base64,{}", base64))
}

/// Open the file location in the system file manager
#[command]
pub fn open_in_folder(path: String) -> Result<(), String> {
    let path = Path::new(&path);

    // If the file doesn't exist, open its parent directory
    let target = if path.exists() {
        path.to_path_buf()
    } else if let Some(parent) = path.parent() {
        parent.to_path_buf()
    } else {
        path.to_path_buf()
    };

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .args(["/select,", target.to_string_lossy().as_ref()])
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        let mut cmd = std::process::Command::new("open");
        if path.exists() {
            cmd.arg("-R");
        }
        cmd.arg(&target)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        let parent = target.parent().unwrap_or(&target);
        std::process::Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
}

/// Export image to a user-selected location (dialog-based)
#[command]
pub async fn export_image_dialog(
    source_path: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    let source = Path::new(&source_path);
    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("output.png");

    let (tx, rx) = tokio::sync::oneshot::channel();

    println!("[export_image_dialog] called, source_path={}", source_path);
    println!("[export_image_dialog] opening save dialog...");

    app.dialog()
        .file()
        .set_file_name(file_name)
        .add_filter("PNG", &["png"])
        .add_filter("JPEG", &["jpg", "jpeg"])
        .add_filter("WebP", &["webp"])
        .save_file(move |path| {
            println!("[export_image_dialog] dialog callback received path: {:?}", path);
            let _ = tx.send(path);
        });

    println!("[export_image_dialog] awaiting dialog result...");
    let save_path = rx.await.map_err(|_| "Dialog interrupted".to_string())?;
    println!("[export_image_dialog] dialog result: {:?}", save_path);

    match save_path {
        Some(FilePath::Path(path)) => {
            fs::copy(&source_path, &path).map_err(|e| format!("Failed to copy file: {}", e))?;
            Ok(path.to_string_lossy().to_string())
        }
        Some(FilePath::Url(uri)) => {
            Ok(uri.to_string())
        }
        None => Err("No path selected".to_string()),
    }
}

/// Save a data URL (base64-encoded image) to a user-selected location
#[command]
pub async fn save_data_url(
    data_url: String,
    suggested_name: String,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    println!("[save_data_url] called, data_url len={}, suggested_name={}", data_url.len(), suggested_name);
    println!("[save_data_url] data_url prefix: {}", &data_url[..std::cmp::min(80, data_url.len())]);

    // Parse data URL: data:image/<type>;base64,<data>
    let (mime_type, b64) = data_url
        .strip_prefix("data:")
        .and_then(|rest| rest.split_once(";base64,"))
        .ok_or_else(|| {
            let preview: String = data_url.chars().take(100).collect();
            format!("Invalid data URL format. Starts with: {}", preview)
        })?;

    println!("[save_data_url] mime_type={}, b64_len={}", mime_type, b64.len());

    let bytes = base64::engine::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        b64,
    )
    .map_err(|e| format!("Failed to decode base64: {}", e))?;

    println!("[save_data_url] decoded {} bytes", bytes.len());

    let (tx, rx) = tokio::sync::oneshot::channel();

    let ext_filter = if mime_type.contains("jpeg") || mime_type.contains("jpg") {
        ("JPEG", vec!["jpg", "jpeg"])
    } else if mime_type.contains("webp") {
        ("WebP", vec!["webp"])
    } else {
        ("PNG", vec!["png"])
    };

    println!("[save_data_url] opening save dialog with filter {:?}", ext_filter.0);

    app.dialog()
        .file()
        .set_file_name(suggested_name.clone())
        .add_filter(ext_filter.0, &ext_filter.1)
        .save_file(move |path| {
            println!("[save_data_url] dialog callback received path: {:?}", path);
            let _ = tx.send(path);
        });

    println!("[save_data_url] awaiting dialog result...");
    let save_path = rx.await.map_err(|_| "Dialog interrupted".to_string())?;
    println!("[save_data_url] dialog result: {:?}", save_path);

    match save_path {
        Some(tauri_plugin_dialog::FilePath::Path(path)) => {
            fs::write(&path, &bytes)
                .map_err(|e| format!("Failed to write file: {}", e))?;
            println!("[save_data_url] written to {}", path.display());
            Ok(path.to_string_lossy().to_string())
        }
        Some(tauri_plugin_dialog::FilePath::Url(uri)) => Ok(uri.to_string()),
        None => Err("No path selected".to_string()),
    }
}

fn apply_mask(
    img: &image::DynamicImage,
    mask: &ndarray::Array2<f32>,
    settings: &MattingSettings,
) -> Result<image::DynamicImage, String> {
    let (width, height) = (img.width(), img.height());
    let mut output = image::RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let mask_value = mask[[y as usize, x as usize]];

            let alpha = if settings.mode == "background" {
                (255.0 * (1.0 - mask_value)) as u8
            } else {
                (255.0 * mask_value) as u8
            };

            output.put_pixel(x, y, image::Rgba([pixel[0], pixel[1], pixel[2], alpha]));
        }
    }

    Ok(image::DynamicImage::ImageRgba8(output))
}

/// Apply the selected background type to the RGBA output image.
fn apply_bg_type(
    img: &image::DynamicImage,
    settings: &MattingSettings,
    ext: &str,
) -> Result<image::DynamicImage, String> {
    let supports_alpha = ext != "jpg";

    match settings.bg_type.as_str() {
        "transparent" if supports_alpha => Ok(img.clone()),
        "white" => composite_on_solid(img, 255, 255, 255),
        "color" => {
            let (r, g, b) = settings
                .bg_color
                .as_ref()
                .and_then(|c| parse_color(c))
                .unwrap_or((255, 255, 255));
            composite_on_solid(img, r, g, b)
        }
        "checkerboard" => composite_on_checkerboard(img),
        // Fallback: composite on white (e.g. transparent + JPG)
        _ => composite_on_solid(img, 255, 255, 255),
    }
}

fn composite_on_solid(
    img: &image::DynamicImage,
    r: u8,
    g: u8,
    b: u8,
) -> Result<image::DynamicImage, String> {
    let (width, height) = (img.width(), img.height());
    let mut output = image::RgbImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let alpha = pixel[3] as f32 / 255.0;

            let pr = (pixel[0] as f32 * alpha + r as f32 * (1.0 - alpha)) as u8;
            let pg = (pixel[1] as f32 * alpha + g as f32 * (1.0 - alpha)) as u8;
            let pb = (pixel[2] as f32 * alpha + b as f32 * (1.0 - alpha)) as u8;

            output.put_pixel(x, y, image::Rgb([pr, pg, pb]));
        }
    }

    Ok(image::DynamicImage::ImageRgb8(output))
}

fn composite_on_checkerboard(
    img: &image::DynamicImage,
) -> Result<image::DynamicImage, String> {
    let (width, height) = (img.width(), img.height());
    let mut output = image::RgbImage::new(width, height);
    let tile_size = 16;

    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let alpha = pixel[3] as f32 / 255.0;

            let tx = x / tile_size;
            let ty = y / tile_size;
            let is_light = (tx + ty) % 2 == 0;
            let (bg_r, bg_g, bg_b) = if is_light {
                (204u8, 204u8, 204u8)
            } else {
                (153u8, 153u8, 153u8)
            };

            let pr = (pixel[0] as f32 * alpha + bg_r as f32 * (1.0 - alpha)) as u8;
            let pg = (pixel[1] as f32 * alpha + bg_g as f32 * (1.0 - alpha)) as u8;
            let pb = (pixel[2] as f32 * alpha + bg_b as f32 * (1.0 - alpha)) as u8;

            output.put_pixel(x, y, image::Rgb([pr, pg, pb]));
        }
    }

    Ok(image::DynamicImage::ImageRgb8(output))
}

fn generate_mask_data_url(
    mask: &ndarray::Array2<f32>,
) -> Result<String, String> {
    let (height, width) = (mask.nrows() as u32, mask.ncols() as u32);
    let mut img_buf = image::GrayImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let v = (mask[[y as usize, x as usize]] * 255.0).clamp(0.0, 255.0) as u8;
            img_buf.put_pixel(x, y, image::Luma([v]));
        }
    }
    let mut bytes: Vec<u8> = Vec::new();
    let dyn_img = image::DynamicImage::ImageLuma8(img_buf);
    dyn_img
        .write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode mask PNG: {}", e))?;
    let b64 = base64::engine::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &bytes,
    );
    Ok(format!("data:image/png;base64,{}", b64))
}

fn create_preview(
    img: &image::DynamicImage,
) -> Result<image::DynamicImage, String> {
    // Create a smaller preview for display
    let max_preview_size = 800;
    let (width, height) = (img.width(), img.height());

    if width <= max_preview_size && height <= max_preview_size {
        return Ok(img.clone());
    }

    let (new_width, new_height) = if width > height {
        let ratio = max_preview_size as f32 / width as f32;
        (max_preview_size, (height as f32 * ratio) as u32)
    } else {
        let ratio = max_preview_size as f32 / height as f32;
        ((width as f32 * ratio) as u32, max_preview_size)
    };

    Ok(img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3))
}

/// Resize output to target dimensions when a size template is specified.
/// When maintain_aspect_ratio is true: scales to fit within target and
/// centers with transparent padding (letterbox/pillarbox).
/// When false: directly resizes (may distort).
/// When no target is set: returns the image unchanged.
fn resize_to_target(
    img: image::DynamicImage,
    settings: &MattingSettings,
) -> image::DynamicImage {
    let tw = match settings.target_width {
        Some(w) if w > 0 => w,
        _ => return img,
    };
    let th = match settings.target_height {
        Some(h) if h > 0 => h,
        _ => return img,
    };

    let (ow, oh) = (img.width(), img.height());

    if !settings.maintain_aspect_ratio {
        return img.resize_exact(tw, th, image::imageops::FilterType::Lanczos3);
    }

    // Letterbox/pillarbox: fit within target, pad with transparency
    let scale = (tw as f64 / ow as f64).min(th as f64 / oh as f64);
    let scaled_w = (ow as f64 * scale).round() as u32;
    let scaled_h = (oh as f64 * scale).round() as u32;

    let resized = img.resize_exact(scaled_w, scaled_h, image::imageops::FilterType::Lanczos3);

    let mut output = image::RgbaImage::new(tw, th);
    let offset_x = ((tw as i64) - (scaled_w as i64)) / 2;
    let offset_y = ((th as i64) - (scaled_h as i64)) / 2;

    imageops::overlay(&mut output, &resized.to_rgba8(), offset_x, offset_y);

    image::DynamicImage::ImageRgba8(output)
}

fn parse_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some((r, g, b))
}
