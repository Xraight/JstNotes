use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::models::CalendarEntry;
use crate::storage::hybrid::HybridStorage;

#[tauri::command]
pub fn create_calendar_entry(
    storage: State<'_, HybridStorage>,
    date: String,
    title: String,
    note_id: Option<String>,
) -> Result<CalendarEntry, String> {
    let entry = CalendarEntry {
        id: Uuid::new_v4().to_string(),
        date,
        title,
        note_id,
        note_title: None,
        created_at: Utc::now().to_rfc3339(),
    };
    storage
        .db
        .create_calendar_entry(&entry)
        .map_err(|e| e.to_string())?;
    Ok(entry)
}

#[tauri::command]
pub fn get_calendar_entries(
    storage: State<'_, HybridStorage>,
    year: i32,
    month: i32,
) -> Result<Vec<CalendarEntry>, String> {
    storage
        .db
        .get_calendar_entries(year, month)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_calendar_entry(
    storage: State<'_, HybridStorage>,
    id: String,
    title: String,
    note_id: Option<String>,
) -> Result<(), String> {
    storage
        .db
        .update_calendar_entry(&id, &title, note_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_calendar_entry(
    storage: State<'_, HybridStorage>,
    id: String,
) -> Result<(), String> {
    storage
        .db
        .delete_calendar_entry(&id)
        .map_err(|e| e.to_string())
}
