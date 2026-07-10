use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tauri::State;

use crate::commands::settings::AppSettingsState;
use crate::models::Note;
use crate::storage::hybrid::HybridStorage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub notes_count: usize,
    pub pdfs_count: usize,
    pub dest_path: String,
}

fn safe_filename(title: &str) -> String {
    title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ' && c != '-' && c != '_', "")
        .replace(' ', "_")
        .chars()
        .take(120)
        .collect::<String>()
        + ".md"
}

#[tauri::command]
pub fn export_all(
    dest_path: String,
    storage: State<'_, HybridStorage>,
    state: State<'_, AppSettingsState>,
) -> Result<ExportResult, String> {
    let app_dir = &state.app_dir;
    let export_path = Path::new(&dest_path);

    fs::create_dir_all(export_path).map_err(|e| format!("Cannot create export dir: {}", e))?;

    let notes = storage.list_notes().map_err(|e| format!("Failed to list notes: {}", e))?;
    let note_map: std::collections::HashMap<String, &Note> =
        notes.iter().map(|n| (n.id.clone(), n)).collect();

    let mut notes_count = 0;

    for note in &notes {
        let mut ancestors: Vec<&Note> = Vec::new();
        let mut current = note.parent_id.as_deref();
        while let Some(pid) = current {
            if let Some(parent) = note_map.get(pid) {
                ancestors.push(parent);
                current = parent.parent_id.as_deref();
            } else {
                break;
            }
        }
        ancestors.reverse();

        let mut note_dir = export_path.to_path_buf();
        for ancestor in &ancestors {
            let dir_name = safe_filename(&ancestor.title).replace(".md", "");
            note_dir = note_dir.join(&dir_name);
        }

        fs::create_dir_all(&note_dir).map_err(|e| format!("Cannot create dir: {}", e))?;

        let content = storage
            .get_note_content(&note.id)
            .map_err(|e| format!("Failed to read note content: {}", e))?
            .unwrap_or_default();

        let created = note.created_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string();
        let updated = note.updated_at.format("%Y-%m-%dT%H:%M:%S%.fZ").to_string();
        let frontmatter = format!(
            "---\ntitle: \"{}\"\nid: {}\ncreated: {}\nupdated: {}\n---\n\n",
            note.title.replace('"', r#"\""#),
            note.id,
            created,
            updated
        );

        let final_content = format!("{}{}", frontmatter, content);

        let file_path = note_dir.join(safe_filename(&note.title));
        fs::write(&file_path, &final_content)
            .map_err(|e| format!("Cannot write note file: {}", e))?;
        notes_count += 1;
    }

    let mut pdfs_count = 0;
    let pdfs_src = Path::new(app_dir).join("pdfs");
    if pdfs_src.exists() {
        let pdfs_dest = export_path.join("pdfs");
        fs::create_dir_all(&pdfs_dest).map_err(|e| format!("Cannot create pdfs dir: {}", e))?;
        if let Ok(entries) = fs::read_dir(&pdfs_src) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let dest = pdfs_dest.join(path.file_name().unwrap_or_default());
                    fs::copy(&path, &dest).map_err(|e| format!("Cannot copy PDF: {}", e))?;
                    pdfs_count += 1;
                }
            }
        }
    }

    let settings_src = Path::new(app_dir).join("settings.json");
    if settings_src.exists() {
        let settings_dest = export_path.join("settings.json");
        fs::copy(&settings_src, &settings_dest).ok();
    }

    Ok(ExportResult {
        notes_count,
        pdfs_count,
        dest_path,
    })
}
