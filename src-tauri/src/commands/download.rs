use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};

/// 编译时可覆盖的默认模型 URL（优先从编译环境变量读取，否则用 CDN 默认值）
const DEFAULT_MODEL_URL: &str = match option_env!("MODEL_URL") {
    Some(url) => url,
    None => "https://mocdn.mopng.cn/models/",
};

/// 编译时可覆盖的默认模型文件名
const DEFAULT_MODEL_FILENAME: &str = match option_env!("MODEL_FILENAME") {
    Some(name) => name,
    None => "birefnet_fp16.onnx",
};

/// 运行时环境变量可覆盖模型 URL（用户级自定义）
fn model_url() -> String {
    std::env::var("MODEL_URL").unwrap_or_else(|_| DEFAULT_MODEL_URL.to_string())
}

/// 运行时环境变量可覆盖模型文件名（用户级自定义）
fn model_filename() -> String {
    std::env::var("MODEL_FILENAME").unwrap_or_else(|_| DEFAULT_MODEL_FILENAME.to_string())
}

/// 构建完整下载链接
fn model_download_url() -> String {
    let url = model_url();
    let filename = model_filename();
    if url.ends_with('/') {
        format!("{}{}", url, filename)
    } else if url.contains("huggingface") || url.contains("model.onnx") {
        url
    } else {
        format!("{}/{}", url, filename)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub percentage: f64,
    pub speed_mbps: f64,
    pub eta_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub exists: bool,
    pub path: String,
    pub size_bytes: u64,
}

#[tauri::command]
pub fn get_model_download_url() -> String {
    let url = model_url();
    if url.contains("mocdn.mopng.cn") {
        "MoCDN".to_string()
    } else if url.contains("huggingface") {
        "HuggingFace".to_string()
    } else {
        "自动配置".to_string()
    }
}

#[tauri::command]
pub fn get_model_path(app: AppHandle) -> Result<String, String> {
    let path = model_file_path(&app)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn check_model(app: AppHandle) -> Result<ModelInfo, String> {
    let path = model_file_path(&app)?;
    let exists = path.exists();
    let size = if exists {
        fs::metadata(&path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    Ok(ModelInfo {
        exists,
        path: path.to_string_lossy().to_string(),
        size_bytes: size,
    })
}

/// 下载模型（同步命令，内部使用 async_runtime）
#[tauri::command]
pub fn download_model(app: AppHandle) -> Result<String, String> {
    tauri::async_runtime::block_on(async { download_model_inner(app).await })
}

async fn download_model_inner(app: AppHandle) -> Result<String, String> {
    let url = model_download_url();
    let model_path = model_file_path(&app)?;
    let temp_path = model_path.with_extension("tmp");

    let client = reqwest::Client::new();

    let downloaded = if temp_path.exists() {
        fs::metadata(&temp_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let total_size = match client.head(&url).send().await {
        Ok(resp) => resp
            .headers()
            .get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0),
        Err(e) => return Err(format!("无法获取文件信息: {}", e)),
    };

    if total_size > 0 && downloaded >= total_size {
        fs::rename(&temp_path, &model_path).map_err(|e| format!("重命名失败: {}", e))?;
        let _ = app.emit("model-download-complete", ModelInfo {
            exists: true,
            path: model_path.to_string_lossy().to_string(),
            size_bytes: total_size,
        });
        return Ok(model_path.to_string_lossy().to_string());
    }

    let mut request = client.get(&url);
    if downloaded > 0 {
        request = request.header("Range", format!("bytes={}-", downloaded));
    }

    let mut response = request
        .send()
        .await
        .map_err(|e| format!("下载失败: {}", e))?;

    if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(format!("服务器返回错误: {}", response.status()));
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&temp_path)
        .map_err(|e| format!("无法创建临时文件: {}", e))?;

    let total_size = if response.headers().contains_key("content-range") {
        total_size
    } else {
        response.content_length().unwrap_or(total_size)
    };

    let downloaded_ref = Arc::new(AtomicU64::new(downloaded));
    let start_time = std::time::Instant::now();
    let last_emit = Arc::new(AtomicU64::new(downloaded));

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("下载中断: {}", e))?
    {
        file.write_all(&chunk).map_err(|e| format!("写入失败: {}", e))?;

        let new_downloaded = downloaded_ref.fetch_add(chunk.len() as u64, Ordering::SeqCst) + chunk.len() as u64;
        let last = last_emit.load(Ordering::SeqCst);
        let elapsed = start_time.elapsed().as_secs_f64();

        if new_downloaded.saturating_sub(last) >= 524_288 || elapsed >= 0.2 {
            last_emit.store(new_downloaded, Ordering::SeqCst);

            let speed = if elapsed > 0.0 {
                (new_downloaded as f64 / elapsed) / 1_048_576.0
            } else {
                0.0
            };

            let eta = if speed > 0.0 && total_size > 0 {
                let remaining = total_size.saturating_sub(new_downloaded) as f64;
                (remaining / (speed * 1_048_576.0)) as u64
            } else {
                0
            };

            let progress = DownloadProgress {
                bytes_downloaded: new_downloaded,
                total_bytes: total_size,
                percentage: if total_size > 0 {
                    (new_downloaded as f64 / total_size as f64) * 100.0
                } else {
                    0.0
                },
                speed_mbps: speed,
                eta_seconds: eta,
            };

            let _ = app.emit("model-download-progress", progress);
        }
    }

    fs::rename(&temp_path, &model_path).map_err(|e| format!("重命名失败: {}", e))?;

    let final_size = fs::metadata(&model_path).map(|m| m.len()).unwrap_or(0);
    let _ = app.emit("model-download-complete", ModelInfo {
        exists: true,
        path: model_path.to_string_lossy().to_string(),
        size_bytes: final_size,
    });

    Ok(model_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn cancel_download(app: AppHandle) -> Result<(), String> {
    let model_path = model_file_path(&app)?;
    let temp_path = model_path.with_extension("tmp");
    if temp_path.exists() {
        fs::remove_file(&temp_path).map_err(|e| format!("删除临时文件失败: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub fn get_model_dir(app: AppHandle) -> Result<String, String> {
    let dir = model_dir(&app)?;
    Ok(dir.to_string_lossy().to_string())
}

fn model_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("models");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path)
}

fn model_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = model_dir(app)?;
    Ok(dir.join(model_filename()))
}
