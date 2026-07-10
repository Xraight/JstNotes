use std::fs;
use std::path::Path;
use tauri::{Manager, State};
use uuid::Uuid;

use crate::models::*;
use crate::pdf::extract::PdfExtractor;
use crate::storage::hybrid::HybridStorage;

#[tauri::command]
pub fn list_pdfs(storage: State<HybridStorage>) -> Result<Vec<PdfMetadata>, String> {
    storage.db.get_all_pdfs().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_pdf(
    file_path: String,
    app_handle: tauri::AppHandle,
    storage: State<HybridStorage>,
) -> Result<PdfMetadata, String> {
    let app_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;

    let pdfs_dir = app_dir.join("pdfs");
    fs::create_dir_all(&pdfs_dir).map_err(|e| e.to_string())?;

    let source = Path::new(&file_path);
    let stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("document");
    let ext = source
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("pdf");

    let dest_filename = format!("{}_{}.{}", stem, Uuid::new_v4().to_string().chars().take(8).collect::<String>(), ext);
    let dest_path = pdfs_dir.join(&dest_filename);

    fs::copy(&file_path, &dest_path).map_err(|e| format!("Failed to copy PDF: {}", e))?;

    let extractor = PdfExtractor::new();
    let meta = extractor.import(dest_path.to_str().unwrap())?;

    storage.db.insert_pdf(&meta).map_err(|e| e.to_string())?;

    Ok(meta)
}

#[tauri::command]
pub fn get_pdf_text(pdf_id: String, storage: State<HybridStorage>) -> Result<Option<String>, String> {
    storage.db.get_pdf_text(&pdf_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_annotation(
    input: AnnotationInput,
    storage: State<HybridStorage>,
) -> Result<PdfAnnotation, String> {
    storage.db.save_annotation(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_annotations(pdf_id: String, storage: State<HybridStorage>) -> Result<Vec<PdfAnnotation>, String> {
    storage.db.get_annotations(&pdf_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_annotation(annotation_id: String, storage: State<HybridStorage>) -> Result<(), String> {
    storage.db.delete_annotation(&annotation_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_annotation_content(
    annotation_id: String,
    content: Option<String>,
    storage: State<HybridStorage>,
) -> Result<(), String> {
    storage.db.update_annotation_content(&annotation_id, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_pdfs_for_note(note_id: String, storage: State<HybridStorage>) -> Result<Vec<PdfMetadata>, String> {
    storage.db.get_pdfs_for_note(&note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn link_pdf_to_note(note_id: String, pdf_id: String, storage: State<HybridStorage>) -> Result<(), String> {
    storage.db.link_pdf_to_note(&note_id, &pdf_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn unlink_pdf_from_note(note_id: String, pdf_id: String, storage: State<HybridStorage>) -> Result<(), String> {
    storage.db.unlink_pdf_from_note(&note_id, &pdf_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_pdf(pdf_id: String, storage: State<HybridStorage>) -> Result<(), String> {
    let meta = storage.db.get_pdf(&pdf_id).map_err(|e| e.to_string())?;
    if let Some(pdf) = meta {
        let _ = fs::remove_file(&pdf.file_path);
    }
    storage.db.delete_pdf(&pdf_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_linked_pdf_ids(note_id: String, storage: State<HybridStorage>) -> Result<Vec<String>, String> {
    storage.db.get_linked_pdf_ids(&note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_pdf_reference(
    input: CreatePdfReferenceInput,
    storage: State<HybridStorage>,
) -> Result<PdfReference, String> {
    storage.db.create_pdf_reference(&input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_pdf_references_for_note(note_id: String, storage: State<HybridStorage>) -> Result<Vec<PdfReference>, String> {
    storage.db.get_pdf_references_for_note(&note_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_pdf_reference(ref_id: String, storage: State<HybridStorage>) -> Result<(), String> {
    storage.db.delete_pdf_reference(&ref_id).map_err(|e| e.to_string())
}
