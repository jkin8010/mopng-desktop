use tauri::AppHandle;
use tauri_plugin_dialog::FilePath;

/// 读取图像文件
#[tauri::command]
pub fn read_image_file(path: String) -> Result<Vec<u8>, String> {
    std::fs::read(&path).map_err(|e| format!("读取文件失败: {}", e))
}

/// 选择多个图片文件（同步命令，内部 async_runtime）
#[tauri::command]
pub fn pick_files(app: AppHandle) -> Result<Vec<String>, String> {
    tauri::async_runtime::block_on(async {
        use tauri_plugin_dialog::DialogExt;

        let file_paths = app
            .dialog()
            .file()
            .add_filter("图片", &["png", "jpg", "jpeg", "webp", "bmp", "gif"])
            .blocking_pick_files()
            .unwrap_or_default();

        let paths: Vec<String> = file_paths
            .into_iter()
            .filter_map(|f| match f {
                FilePath::Path(p) => Some(p.to_string_lossy().to_string()),
                FilePath::Url(u) => Some(u.to_string()),
            })
            .collect();

        Ok(paths)
    })
}

#[tauri::command]
pub fn select_output_dir(app: AppHandle) -> Result<Option<String>, String> {
    tauri::async_runtime::block_on(async {
        use tauri_plugin_dialog::DialogExt;

        let result = app.dialog().file().blocking_pick_folder();
        match result {
            Some(FilePath::Path(p)) => Ok(Some(p.to_string_lossy().to_string())),
            Some(FilePath::Url(u)) => Ok(Some(u.to_string())),
            _ => Ok(None),
        }
    })
}
