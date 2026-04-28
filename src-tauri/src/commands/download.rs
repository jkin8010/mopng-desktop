use std::fs;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSource {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub default: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceError {
    pub source_id: String,
    pub source_name: String,
    pub error_type: String, // "network", "http_404", "checksum_mismatch", "timeout", "http_5xx"
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DownloadErrorResponse {
    pub message: String,
    pub source_errors: Vec<SourceError>,
    pub model_filename: String,
}

static CANCEL_DOWNLOAD: AtomicBool = AtomicBool::new(false);

#[tauri::command]
pub fn get_model_sources(model_id: Option<String>) -> Vec<ModelSource> {
    let id = model_id.unwrap_or_else(|| "birefnet".to_string());
    crate::models::registry::model_sources_for(&id).unwrap_or_default()
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

/// 下载模型（仅接受 model_id，不再接受 source_url）
#[tauri::command]
pub async fn download_model(app: AppHandle, model_id: Option<String>) -> Result<String, String> {
    let id = model_id.unwrap_or_else(|| "birefnet".to_string());
    download_model_with_fallback(app, &id).await
}

/// 核心回退逻辑：遍历所有下载源直到成功
async fn download_model_with_fallback(app: AppHandle, model_id: &str) -> Result<String, String> {
    CANCEL_DOWNLOAD.store(false, Ordering::SeqCst);

    let model_dir = crate::models::registry::model_dir(&app)?;
    let filename = crate::models::registry::model_filename_for(model_id)
        .unwrap_or_else(|| format!("{}.onnx", model_id));
    let model_path = model_dir.join(&filename);

    let sources = crate::models::registry::model_sources_for(model_id).unwrap_or_default();

    // .env 覆盖（per D-05）：MODEL_URL 设为优先源
    let sources = if let Ok(override_url) = std::env::var("MODEL_URL") {
        let mut s = vec![ModelSource {
            id: "env-override".into(),
            name: "环境变量覆盖".into(),
            description: "来自 .env MODEL_URL".into(),
            url: override_url,
            default: true,
        }];
        s.extend(sources);
        s
    } else {
        sources
    };

    if sources.is_empty() {
        return Err(format!("模型 {} 没有配置下载源", model_id));
    }

    let mut source_errors: Vec<SourceError> = Vec::new();

    for source in &sources {
        log::info!("尝试下载源: {} ({})", source.name, source.url);
        match download_from_source(&app, &model_path, source, model_id).await {
            Ok(path) => {
                let final_size = tokio::fs::metadata(&model_path).await.map(|m| m.len()).unwrap_or(0);
                let _ = app.emit("model-download-complete", ModelInfo {
                    exists: true,
                    path: path.clone(),
                    size_bytes: final_size,
                });
                return Ok(path);
            }
            Err(e) => {
                log::warn!("源 {} 下载失败: {}", source.name, e);
                source_errors.push(SourceError {
                    source_id: source.id.clone(),
                    source_name: source.name.clone(),
                    error_type: classify_download_error(&e),
                    detail: e.clone(),
                });
            }
        }
    }

    let error_response = DownloadErrorResponse {
        message: format!("模型下载失败：所有 {} 个下载源均不可用", sources.len()),
        source_errors,
        model_filename: filename.clone(),
    };
    Err(serde_json::to_string(&error_response)
        .unwrap_or_else(|_| "下载失败，所有源均不可用".to_string()))
}

/// 从单个源下载模型，包含 SHA256 流式校验和断点续传
async fn download_from_source(
    app: &AppHandle,
    model_path: &std::path::PathBuf,
    source: &ModelSource,
    model_id: &str,
) -> Result<String, String> {
    let temp_path = model_path.with_extension("tmp");
    let url = &source.url;

    let mut client_builder = reqwest::Client::builder();
    if url.contains("modelscope.cn") {
        client_builder = client_builder
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(reqwest::header::REFERER, reqwest::header::HeaderValue::from_static("https://modelscope.cn/"));
                h.insert(reqwest::header::ACCEPT, reqwest::header::HeaderValue::from_static("*/*"));
                h
            });
    }
    let client = client_builder.build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let downloaded = if temp_path.exists() {
        tokio::fs::metadata(&temp_path).await.map(|m| m.len()).unwrap_or(0)
    } else { 0 };

    let total_size = match client.head(url).send().await {
        Ok(resp) => resp.headers().get("content-length")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0),
        Err(e) => return Err(format!("无法获取文件信息: {}", e)),
    };

    if total_size > 0 && downloaded >= total_size {
        if verify_download_checksum(&temp_path, model_id).await {
            tokio::fs::rename(&temp_path, model_path).await
                .map_err(|e| format!("重命名失败: {}", e))?;
            return Ok(model_path.to_string_lossy().to_string());
        } else {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(format!("[checksum_mismatch] SHA256 校验失败 — 已下载文件与预期不一致"));
        }
    }

    let mut request = client.get(url);
    if downloaded > 0 {
        request = request.header("Range", format!("bytes={}-", downloaded));
    }

    let mut response = request.send().await
        .map_err(|e| format!("下载请求失败: {}", e))?;

    let status = response.status();
    if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
        let error_type = if status == reqwest::StatusCode::NOT_FOUND { "http_404" }
            else if status.is_server_error() { "http_5xx" }
            else { "http_error" };
        return Err(format!("[{}] 服务器返回 HTTP {}", error_type, status));
    }

    let actual_total = if response.headers().contains_key("content-range") {
        total_size
    } else {
        response.content_length().unwrap_or(total_size)
    };

    let mut file = tokio::fs::OpenOptions::new()
        .create(true).append(true).open(&temp_path).await
        .map_err(|e| format!("无法创建临时文件: {}", e))?;

    let downloaded_ref = Arc::new(AtomicU64::new(downloaded));
    let start_time = std::time::Instant::now();
    let last_emit = Arc::new(AtomicU64::new(downloaded));

    // SHA256 流式累加
    let mut hasher = Sha256::new();
    if downloaded > 0 {
        if let Ok(existing) = tokio::fs::read(&temp_path).await {
            hasher.update(&existing);
        }
    }

    while let Some(chunk) = response.chunk().await
        .map_err(|e| format!("下载中断: {}", e))?
    {
        if CANCEL_DOWNLOAD.load(Ordering::SeqCst) {
            return Err("下载已取消".to_string());
        }
        file.write_all(&chunk).await.map_err(|e| format!("写入失败: {}", e))?;
        hasher.update(&chunk);

        let new_downloaded = downloaded_ref.fetch_add(chunk.len() as u64, Ordering::SeqCst) + chunk.len() as u64;
        let last = last_emit.load(Ordering::SeqCst);
        let elapsed = start_time.elapsed().as_secs_f64();

        if new_downloaded.saturating_sub(last) >= 524_288 || elapsed >= 0.2 {
            last_emit.store(new_downloaded, Ordering::SeqCst);
            let speed = if elapsed > 0.0 { (new_downloaded as f64 / elapsed) / 1_048_576.0 } else { 0.0 };
            let eta = if speed > 0.0 && actual_total > 0 {
                ((actual_total.saturating_sub(new_downloaded) as f64) / (speed * 1_048_576.0)) as u64
            } else { 0 };
            let _ = app.emit("model-download-progress", DownloadProgress {
                bytes_downloaded: new_downloaded,
                total_bytes: actual_total,
                percentage: if actual_total > 0 { (new_downloaded as f64 / actual_total as f64) * 100.0 } else { 0.0 },
                speed_mbps: speed,
                eta_seconds: eta,
            });
        }
    }

    // SHA256 校验
    let computed_hash = hex::encode(hasher.finalize());
    if !verify_checksum_against_descriptor(model_id, &computed_hash)? {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(format!("[checksum_mismatch] SHA256 校验失败\n期望: 来自模型描述符\n实际: {}", computed_hash));
    }

    tokio::fs::rename(&temp_path, model_path).await
        .map_err(|e| format!("重命名失败: {}", e))?;

    Ok(model_path.to_string_lossy().to_string())
}

async fn verify_download_checksum(temp_path: &std::path::Path, model_id: &str) -> bool {
    match compute_file_sha256(temp_path) {
        Ok(hash) => verify_checksum_against_descriptor(model_id, &hash).unwrap_or(false),
        Err(_) => false,
    }
}

fn verify_checksum_against_descriptor(model_id: &str, computed_hash: &str) -> Result<bool, String> {
    let models = crate::models::registry::list_models();
    let model = models.iter().find(|m| m.id == model_id)
        .ok_or_else(|| format!("未知模型: {}", model_id))?;
    if let Some(ref expected) = model.checksum {
        Ok(computed_hash == expected)
    } else {
        Ok(true) // 无 checksum → 跳过校验
    }
}

fn classify_download_error(error: &str) -> String {
    if error.contains("http_404") || error.contains("404") { "http_404".into() }
    else if error.contains("timeout") || error.contains("超时") { "timeout".into() }
    else if error.contains("checksum_mismatch") || error.contains("SHA256") { "checksum_mismatch".into() }
    else if error.contains("http_5xx") { "http_5xx".into() }
    else { "network".into() }
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
