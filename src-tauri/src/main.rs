mod commands;
mod models;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::process_image,
            commands::generate_thumbnail,
            commands::open_in_folder,
            commands::export_image,
            commands::get_model_path,
            commands::set_output_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
