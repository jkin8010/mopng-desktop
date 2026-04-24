use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::FilePath;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
}

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
                _ => None,
            })
            .collect();

        Ok(paths)
    })
}
