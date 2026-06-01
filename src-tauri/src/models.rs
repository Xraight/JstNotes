use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub parent_id: Option<String>,
    pub path: String,
    pub sort_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNoteRequest {
    pub title: String,
    pub content: String,
    pub parent_id: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNoteRequest {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteTreeNode {
    pub id: String,
    pub title: String,
    pub path: String,
    pub children: Vec<NoteTreeNode>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breadcrumb {
    pub id: String,
    pub title: String,
    pub path: String,
}

impl Note {
    pub fn new(req: CreateNoteRequest, notes_dir: &str) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let path = build_note_path(notes_dir, &id, &req.title, req.parent_id.as_deref());

        Note {
            id,
            title: req.title,
            content: req.content,
            parent_id: req.parent_id,
            path,
            sort_order: req.sort_order.unwrap_or(0),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub colors: ColorSettings,
    pub typography: TypographySettings,
    pub layout: LayoutSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            theme: "dark".to_string(),
            colors: ColorSettings::default(),
            typography: TypographySettings::default(),
            layout: LayoutSettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSettings {
    pub bg_primary: String,
    pub bg_secondary: String,
    pub accent: String,
    pub highlight: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub border: String,
}

impl Default for ColorSettings {
    fn default() -> Self {
        ColorSettings {
            bg_primary: "#1A1A2E".to_string(),
            bg_secondary: "#16213E".to_string(),
            accent: "#0F3460".to_string(),
            highlight: "#E94560".to_string(),
            text_primary: "#E0E0E0".to_string(),
            text_secondary: "#A0A0A0".to_string(),
            border: "#2a2a4a".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographySettings {
    pub font_family: String,
    pub font_family_mono: String,
    pub font_size: f64,
    pub line_height: f64,
}

impl Default for TypographySettings {
    fn default() -> Self {
        TypographySettings {
            font_family: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif".to_string(),
            font_family_mono: "'JetBrains Mono', 'Fira Code', monospace".to_string(),
            font_size: 15.0,
            line_height: 1.7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutSettings {
    pub sidebar_width: f64,
}

impl Default for LayoutSettings {
    fn default() -> Self {
        LayoutSettings { sidebar_width: 280.0 }
    }
}

pub fn load_settings(app_dir: &str) -> AppSettings {
    let path = format!("{}/settings.json", app_dir);
    if Path::new(&path).exists() {
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    } else {
        AppSettings::default()
    }
}

pub fn persist_settings(app_dir: &str, settings: &AppSettings) -> Result<(), String> {
    let path = format!("{}/settings.json", app_dir);
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarEntry {
    pub id: String,
    pub date: String,
    pub title: String,
    pub note_id: Option<String>,
    pub note_title: Option<String>,
    pub created_at: String,
}

pub fn build_note_path(notes_dir: &str, id: &str, title: &str, parent_id: Option<&str>) -> String {
    let safe_title = title
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != ' ', "_")
        .replace(' ', "_");
    match parent_id {
        Some(pid) => format!("{}/{}/{}", notes_dir, pid, safe_title),
        None => format!("{}/{}", notes_dir, safe_title),
    }
}
