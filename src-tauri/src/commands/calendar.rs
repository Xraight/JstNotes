use chrono::Utc;
use tauri::State;
use uuid::Uuid;

use crate::models::CalendarEvent;
use crate::storage::hybrid::HybridStorage;

#[tauri::command]
pub fn create_calendar_event(
    storage: State<'_, HybridStorage>,
    date: String,
    title: String,
    description: String,
    note_ids: Vec<String>,
) -> Result<CalendarEvent, String> {
    let event = CalendarEvent {
        id: Uuid::new_v4().to_string(),
        date,
        title,
        description,
        completed: false,
        note_ids: note_ids.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    storage
        .db
        .create_calendar_event(&event, &note_ids)
        .map_err(|e| e.to_string())?;
    Ok(event)
}

#[tauri::command]
pub fn get_calendar_events(
    storage: State<'_, HybridStorage>,
    year: i32,
    month: i32,
) -> Result<Vec<CalendarEvent>, String> {
    storage
        .db
        .get_calendar_events(year, month)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_calendar_event(
    storage: State<'_, HybridStorage>,
    id: String,
    title: String,
    description: String,
    completed: bool,
) -> Result<(), String> {
    storage
        .db
        .update_calendar_event(&id, &title, &description, completed)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_calendar_event(
    storage: State<'_, HybridStorage>,
    id: String,
) -> Result<(), String> {
    storage
        .db
        .delete_calendar_event(&id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn link_note_to_event(
    storage: State<'_, HybridStorage>,
    event_id: String,
    note_id: String,
) -> Result<(), String> {
    storage
        .db
        .link_note_to_event(&event_id, &note_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unlink_note_from_event(
    storage: State<'_, HybridStorage>,
    event_id: String,
    note_id: String,
) -> Result<(), String> {
    storage
        .db
        .unlink_note_from_event(&event_id, &note_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_events_for_note(
    storage: State<'_, HybridStorage>,
    note_id: String,
) -> Result<Vec<CalendarEvent>, String> {
    storage
        .db
        .get_events_for_note(&note_id)
        .map_err(|e| e.to_string())
}
