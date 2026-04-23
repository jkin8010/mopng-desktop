use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use image::io::Reader as ImageReader;

use crate::models::birefnet::BirefnetSession;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessRequest {
    pub image_path: String,
    pub model_path: Option<String>,
    pub provider: String,
    pub max_width: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessResult {
    pub output_path: String,
    pub width: u32,
    pub height: u32,
}

/// 初始化 BiRefNet 会话
#[tauri::command]
pub fn init_model(model_path: String, provider: String) -> Result<(), String> {
    let path = PathBuf::from(model_path);
    if !path.exists() {
        return Err(format!("模型文件不存在: {:?}", path));
    }

    BirefnetSession::init(path, &provider)
        .map_err(|e| format!("模型初始化失败: {}", e))?;

    log::info!("BiRefNet 模型初始化成功");
    Ok(())
}

/// 执行抠图
#[tauri::command]
pub fn matte_image(request: ProcessRequest) -> Result<String, String> {
    // 检查会话是否已初始化
    let session = BirefnetSession::get()
        .ok_or("模型未初始化，请先加载模型")?;

    let image_path = PathBuf::from(&request.image_path);
    if !image_path.exists() {
        return Err(format!("图片文件不存在: {}", request.image_path));
    }

    log::info!("开始处理图片: {}", request.image_path);

    // 读取图片
    let original_image = ImageReader::open(&image_path)
        .map_err(|e| format!("打开图片失败: {}", e))?
        .with_guessed_format()
        .map_err(|e| format!("识别图片格式失败: {}", e))?
        .decode()
        .map_err(|e| format!("解码图片失败: {}", e))?;

    // 限制最大宽度
    let image = if let Some(max_w) = request.max_width {
        let (w, h) = (original_image.width(), original_image.height());
        if w > max_w {
            let ratio = max_w as f32 / w as f32;
            let new_h = (h as f32 * ratio) as u32;
            original_image.resize(max_w, new_h, image::imageops::Lanczos3)
        } else {
            original_image
        }
    } else {
        original_image
    };

    // 运行推理
    let alpha_mask = session.run(image.clone())
        .map_err(|e| format!("推理失败: {}", e))?;

    // 后处理
    let png_bytes = session.post_process(alpha_mask, image)
        .map_err(|e| format!("后处理失败: {}", e))?;

    // 保存结果
    let output_dir = image_path.parent()
        .unwrap_or(PathBuf::from(".").as_path())
        .join("output");
    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let stem = image_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("result");
    let output_path = output_dir.join(format!("{}_matte.png", stem));

    std::fs::write(&output_path, png_bytes)
        .map_err(|e| format!("保存结果失败: {}", e))?;

    log::info!("处理完成: {:?}", output_path);

    Ok(output_path.to_string_lossy().to_string())
}

/// 获取图片尺寸信息
#[tauri::command]
pub fn get_image_info(path: String) -> Result<(u32, u32), String> {
    let image = ImageReader::open(&path)
        .map_err(|e| e.to_string())?
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?;

    Ok((image.width(), image.height()))
}
