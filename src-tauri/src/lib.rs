mod commands;
mod models;
mod pdf;
mod storage;
mod ai;

use commands::settings::AppSettingsState;
use storage::hybrid::HybridStorage;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;

                if let Some(window) = app.get_webview_window("main") {
                    window.open_devtools();
                }
            }

            let app_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| e.to_string())
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            std::fs::create_dir_all(&app_dir).ok();
            let app_dir_str = app_dir.to_string_lossy().to_string();

            let storage = HybridStorage::new(&app_dir_str)
                .expect("Failed to initialize storage");
            app.manage(storage);

            let settings = models::load_settings(&app_dir_str);
            app.manage(AppSettingsState {
                app_dir: app_dir_str.clone(),
                settings: std::sync::Mutex::new(settings),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::notes::create_note,
            commands::notes::get_note,
            commands::notes::update_note,
            commands::notes::delete_note,
            commands::notes::list_notes,
            commands::notes::get_children,
            commands::notes::get_breadcrumbs,
            commands::notes::build_tree,
            commands::ai::generate_flashcards,
            commands::pdf::import_pdf,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::calendar::create_calendar_entry,
            commands::calendar::get_calendar_entries,
            commands::calendar::update_calendar_entry,
            commands::calendar::delete_calendar_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
