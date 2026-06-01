use crate::models::*;
use tauri::State;
use std::sync::Mutex;

pub struct AppSettingsState {
    pub app_dir: String,
    pub settings: Mutex<AppSettings>,
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
