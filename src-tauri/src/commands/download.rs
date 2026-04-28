use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

/// 编译时可覆盖的默认模型 URL（优先从编译环境变量读取，否则用 CDN 默认值）
const DEFAULT_MODEL_URL: &str = match option_env!("MODEL_URL") {
    Some(url) => url,
    None => "https://modelscope.cn/models/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model.onnx",
};

/// 编译时可覆盖的默认模型文件名
const DEFAULT_MODEL_FILENAME: &str = match option_env!("MODEL_FILENAME") {
    Some(name) => name,
    None => "birefnet.onnx",
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
    } else if url.ends_with(".onnx") || url.contains("huggingface") {
        url
    } else {
        format!("{}/{}", url, filename)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSource {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub default: bool,
}

const HF_RAW_URL: &str = "https://huggingface.co/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model.onnx";
const HF_MIRROR_URL: &str = "https://hf-mirror.com/onnx-community/BiRefNet-ONNX/resolve/main/onnx/model.onnx";

static CANCEL_DOWNLOAD: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn get_model_sources(model_id: Option<String>) -> Vec<ModelSource> {
    let id = model_id.unwrap_or_else(|| "birefnet".to_string());
    if let Some(sources) = crate::models::registry::model_sources_for(&id) {
        return sources;
    }
    // Fallback: legacy hardcoded sources
    let default_url = model_download_url();
    vec![
        ModelSource {
            id: "modelscope".into(),
            name: "ModelScope".into(),
            description: "魔搭社区，国内可直接访问".into(),
            url: default_url.clone(),
            default: true,
        },
        ModelSource {
            id: "huggingface".into(),
            name: "HuggingFace".into(),
            description: "海外源，需科学上网".into(),
            url: HF_RAW_URL.to_string(),
            default: false,
        },
        ModelSource {
            id: "hf-mirror".into(),
            name: "HF Mirror".into(),
            description: "HuggingFace 国内镜像".into(),
            url: HF_MIRROR_URL.to_string(),
            default: false,
        },
    ]
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
    if url.contains("modelscope.cn") {
        "ModelScope".to_string()
    } else if url.contains("huggingface") {
        "HuggingFace".to_string()
    } else {
        "自动配置".to_string()
    }
}

#[tauri::command]
pub fn check_model(app: AppHandle, model_id: Option<String>) -> Result<ModelInfo, String> {
    let id = model_id.unwrap_or_else(|| "birefnet".to_string());
    let model_dir = crate::models::registry::model_dir(&app)?;
    let filename = crate::models::registry::model_filename_for(&id)
        .unwrap_or_else(|| format!("{}.onnx", id));
    let path = model_dir.join(&filename);
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

/// 下载模型
#[tauri::command]
pub async fn download_model(app: AppHandle, source_url: Option<String>, model_id: Option<String>) -> Result<String, String> {
    download_model_inner(app, source_url, model_id).await
}

async fn download_model_inner(app: AppHandle, source_url: Option<String>, model_id: Option<String>) -> Result<String, String> {
    CANCEL_DOWNLOAD.store(false, Ordering::SeqCst);
    let id = model_id.unwrap_or_else(|| "birefnet".to_string());
    let url = source_url.unwrap_or_else(model_download_url);
    let model_dir = crate::models::registry::model_dir(&app)?;
    let filename = crate::models::registry::model_filename_for(&id)
        .unwrap_or_else(|| format!("{}.onnx", id));
    let model_path = model_dir.join(&filename);
    let temp_path = model_path.with_extension("tmp");

    let mut client_builder = reqwest::Client::builder();
    if url.contains("modelscope.cn") {
        client_builder = client_builder
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::REFERER,
                    reqwest::header::HeaderValue::from_static("https://modelscope.cn/"),
                );
                h.insert(
                    reqwest::header::ACCEPT,
                    reqwest::header::HeaderValue::from_static("*/*"),
                );
                h
            });
    }
    let client = client_builder
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let downloaded = if temp_path.exists() {
        tokio::fs::metadata(&temp_path).await.map(|m| m.len()).unwrap_or(0)
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
        tokio::fs::rename(&temp_path, &model_path).await.map_err(|e| format!("重命名失败: {}", e))?;
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

    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&temp_path)
        .await
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
        if CANCEL_DOWNLOAD.load(Ordering::SeqCst) {
            return Err("下载已取消".to_string());
        }

        file.write_all(&chunk).await.map_err(|e| format!("写入失败: {}", e))?;

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

    tokio::fs::rename(&temp_path, &model_path).await.map_err(|e| format!("重命名失败: {}", e))?;

    let final_size = tokio::fs::metadata(&model_path).await.map(|m| m.len()).unwrap_or(0);
    let _ = app.emit("model-download-complete", ModelInfo {
        exists: true,
        path: model_path.to_string_lossy().to_string(),
        size_bytes: final_size,
    });

    Ok(model_path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn cancel_download(app: AppHandle, model_id: Option<String>) -> Result<(), String> {
    CANCEL_DOWNLOAD.store(true, Ordering::SeqCst);
    let id = model_id.unwrap_or_else(|| "birefnet".to_string());
    let model_dir = crate::models::registry::model_dir(&app)?;
    let filename = crate::models::registry::model_filename_for(&id)
        .unwrap_or_else(|| format!("{}.onnx", id));
    let model_path = model_dir.join(&filename);
    let temp_path = model_path.with_extension("tmp");
    if temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }
    Ok(())
}

#[tauri::command]
pub fn get_model_dir(app: AppHandle) -> Result<String, String> {
    let dir = crate::models::registry::model_dir(&app)?;
    Ok(dir.to_string_lossy().to_string())
}

/// 使用流式读取（8KB 缓冲区）计算文件 SHA256，返回十六进制字符串
pub(crate) fn compute_file_sha256(path: &std::path::Path) -> Result<String, String> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("无法打开文件进行 SHA256 校验: {}", e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let n = file.read(&mut buffer)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}
