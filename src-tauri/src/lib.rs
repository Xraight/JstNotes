//! JSTNotes — Tauri application entry point.
//!
//! Manages global state:
//!   - HybridStorage (SQLite DB + Markdown files)
//!   - AppSettingsState (colors, typography, layout, AI config)
//!
//! Registers 35+ IPC commands across notes, PDF, AI, calendar, and settings.

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
                        .filter(|metadata| {
                            !metadata.target().starts_with("pdf_extract")
                                && !metadata.target().starts_with("cff_parser")
                        })
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

            // Initialize managed state objects — available to all commands via State<T>.
            let storage = HybridStorage::new(&app_dir_str)
                .expect("Failed to initialize storage");
            app.manage(storage);

            // Settings loaded from disk, exposed via Mutex for thread-safe mutation.
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
            commands::ai::test_ai_connection,
            commands::ai::fetch_ai_models,
            commands::ai::generate_study_questions,
            commands::ai::generate_elaboration_questions,
            commands::ai::generate_concrete_example,
            commands::ai::get_due_reviews,
            commands::ai::rate_review,
            commands::ai::generate_feynman_prompt,
            commands::ai::evaluate_feynman,
            commands::ai::get_study_items,
            commands::pdf::list_pdfs,
            commands::pdf::import_pdf,
            commands::pdf::get_pdf_text,
            commands::pdf::save_annotation,
            commands::pdf::get_annotations,
            commands::pdf::delete_annotation,
            commands::pdf::update_annotation_content,
            commands::pdf::get_pdfs_for_note,
            commands::pdf::link_pdf_to_note,
            commands::pdf::unlink_pdf_from_note,
            commands::pdf::delete_pdf,
            commands::pdf::get_linked_pdf_ids,
            commands::pdf::create_pdf_reference,
            commands::pdf::get_pdf_references_for_note,
            commands::pdf::delete_pdf_reference,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::calendar::create_calendar_event,
            commands::calendar::get_calendar_events,
            commands::calendar::update_calendar_event,
            commands::calendar::delete_calendar_event,
            commands::calendar::link_note_to_event,
            commands::calendar::unlink_note_from_event,
            commands::calendar::get_events_for_note,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
