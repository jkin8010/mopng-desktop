mod commands;
mod models;

fn main() {
    // 加载 .env（开发/打包配置，用户无需知晓）
    if let Err(e) = dotenvy::dotenv() {
        log::debug!(".env not loaded: {}", e);
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::process_image,
            commands::generate_thumbnail,
            commands::open_in_folder,
            commands::export_image_dialog,
            commands::get_model_path,
            commands::check_model,
            commands::download_model,
            commands::cancel_download,
            commands::get_model_download_url,
            commands::get_model_dir,
            commands::read_image_file,
            commands::pick_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
