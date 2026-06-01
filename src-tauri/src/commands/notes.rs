use crate::storage::hybrid::HybridStorage;
use crate::models::*;
use tauri::State;

#[tauri::command]
pub fn create_note(
    storage: State<'_, HybridStorage>,
    req: CreateNoteRequest,
) -> Result<Note, String> {
    storage.create_note(req)
}

#[tauri::command]
pub fn get_note(
    storage: State<'_, HybridStorage>,
    id: String,
) -> Result<Option<Note>, String> {
    storage.get_note(&id)
}

#[tauri::command]
pub fn update_note(
    storage: State<'_, HybridStorage>,
    req: UpdateNoteRequest,
) -> Result<Note, String> {
    storage.update_note(req)
}

#[tauri::command]
pub fn delete_note(
    storage: State<'_, HybridStorage>,
    id: String,
) -> Result<(), String> {
    storage.delete_note(&id)
}

#[tauri::command]
pub fn list_notes(
    storage: State<'_, HybridStorage>,
) -> Result<Vec<Note>, String> {
    storage.list_notes()
}

#[tauri::command]
pub fn get_children(
    storage: State<'_, HybridStorage>,
    parent_id: String,
) -> Result<Vec<Note>, String> {
    storage.get_children(&parent_id)
}

#[tauri::command]
pub fn get_breadcrumbs(
    storage: State<'_, HybridStorage>,
    note_id: String,
) -> Result<Vec<Breadcrumb>, String> {
    storage.get_breadcrumbs(&note_id)
}

#[tauri::command]
pub fn build_tree(
    storage: State<'_, HybridStorage>,
) -> Result<Vec<NoteTreeNode>, String> {
    storage.build_tree()
}
