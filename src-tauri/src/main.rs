mod commands;
mod models;

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // 加载 .env（可从源码目录或运行目录配置环境变量）
    // 支持的环境变量: MODEL_URL, MODEL_FILENAME
    let env_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    if let Err(e) = dotenvy::from_path(&env_path) {
        if let Err(e2) = dotenvy::dotenv() {
            log::debug!(".env not loaded: {} / {}", e, e2);
        }
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
            models::init_model,
            models::is_model_loaded,
            models::list_models,
            models::switch_model,
            models::scan_models,
            commands::check_model,
            commands::download_model,
            commands::cancel_download,
            commands::get_model_dir,
            commands::read_image_file,
            commands::read_file_as_data_url,
            commands::pick_files,
            commands::get_model_sources,
            commands::select_output_dir,
            commands::save_data_url,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
