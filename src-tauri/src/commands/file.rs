use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_fs::FsExt;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
}

/// 获取应用数据目录
#[tauri::command]
pub fn get_app_dir(app: AppHandle) -> Result<String, String> {
    let path = app.path_resolver().app_data_dir()
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

/// 获取模型目录
#[tauri::command]
pub fn get_model_dir(app: AppHandle) -> Result<String, String> {
    let path = app.path_resolver().app_data_dir()
        .map_err(|e| e.to_string())?
        .join("models");
    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

/// 检查模型文件是否存在
#[tauri::command]
pub fn check_model_file(app: AppHandle) -> Result<bool, String> {
    let model_dir = get_model_dir(app)?;
    let model_path = PathBuf::from(model_dir).join("birefnet.onnx");
    Ok(model_path.exists())
}
