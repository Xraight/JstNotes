use crate::models::*;
use std::fs;
use tauri::State;
use std::sync::Mutex;

pub struct AppSettingsState {
    pub app_dir: String,
    pub settings: Mutex<AppSettings>,
}

#[tauri::command]
pub fn get_app_dir(state: State<'_, AppSettingsState>) -> Result<String, String> {
    Ok(state.app_dir.clone())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppSettingsState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn save_settings(
    state: State<'_, AppSettingsState>,
    new_settings: AppSettings,
) -> Result<AppSettings, String> {
    {
        let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
        *settings = new_settings.clone();
    }
    crate::models::persist_settings(&state.app_dir, &new_settings)?;
    Ok(new_settings)
}

#[tauri::command]
pub fn save_settings_raw(
    state: State<'_, AppSettingsState>,
    json_content: String,
) -> Result<String, String> {
    let path = format!("{}/settings.json", state.app_dir);
    // Validate JSON before writing
    let _parsed: serde_json::Value =
        serde_json::from_str(&json_content).map_err(|e| format!("Invalid JSON: {}", e))?;
    fs::write(&path, &json_content).map_err(|e| format!("Write failed: {}", e))?;
    // Read back and return
    let back = fs::read_to_string(&path).map_err(|e| format!("Readback failed: {}", e))?;
    // Also update the in-memory settings for other commands
    if let Ok(settings) = serde_json::from_str::<AppSettings>(&back) {
        let mut mem = state.settings.lock().map_err(|e| e.to_string())?;
        *mem = settings;
    }
    Ok(back)
}

#[tauri::command]
pub fn get_settings_raw(state: State<'_, AppSettingsState>) -> Result<String, String> {
    let path = format!("{}/settings.json", state.app_dir);
    match fs::read_to_string(&path) {
        Ok(content) => Ok(content),
        Err(_) => {
            let default = crate::models::AppSettings::default();
            let json = serde_json::to_string_pretty(&default).map_err(|e| e.to_string())?;
            Ok(json)
        }
    }
}
