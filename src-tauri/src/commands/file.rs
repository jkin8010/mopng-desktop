use tauri::AppHandle;
use tauri_plugin_dialog::FilePath;

/// 读取图像文件
#[tauri::command]
pub fn read_image_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))
}

/// Read a file and return it as a data URL (data:<mime>;base64,<content>)
/// This avoids canvas tainting issues when loading images from asset:// protocol
#[tauri::command]
pub fn read_file_as_data_url(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))?;
    let mime = match std::path::Path::new(&path).extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("gif") => "image/gif",
        _ => "image/png",
    };
    let b64 = base64::engine::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &bytes,
    );
    Ok(format!("data:{};base64,{}", mime, b64))
}

/// 选择多个图片文件（async 回调模式，避免阻塞主线程）
#[tauri::command]
pub async fn pick_files(app: AppHandle) -> Result<Vec<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = tokio::sync::oneshot::channel();

    app.dialog()
        .file()
        .add_filter("图片", &["png", "jpg", "jpeg", "webp", "bmp", "gif"])
        .pick_files(move |file_paths| {
            let _ = tx.send(file_paths.unwrap_or_default());
        });

    let paths: Vec<String> = rx
        .await
        .map_err(|_| "文件选择被中断".to_string())?
        .into_iter()
        .filter_map(|f| match f {
            FilePath::Path(p) => Some(p.to_string_lossy().to_string()),
            FilePath::Url(u) => Some(u.to_string()),
        })
        .collect();

    Ok(paths)
}

#[tauri::command]
pub async fn select_output_dir(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;

    let (tx, rx) = tokio::sync::oneshot::channel();

    app.dialog().file().pick_folder(move |result| {
        let _ = tx.send(result);
    });

    match rx.await.map_err(|_| "目录选择被中断".to_string())? {
        Some(FilePath::Path(p)) => Ok(Some(p.to_string_lossy().to_string())),
        Some(FilePath::Url(u)) => Ok(Some(u.to_string())),
        _ => Ok(None),
    }
}
