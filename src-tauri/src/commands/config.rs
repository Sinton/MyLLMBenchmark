use super::error_to_string;
use crate::config::{AppConfig, ConfigUpdateResult};
use crate::services;
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    services::get_app_config(state.inner())
        .await
        .map_err(error_to_string)
}

#[tauri::command]
pub async fn update_app_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<ConfigUpdateResult, String> {
    services::update_app_config(state.inner(), config)
        .await
        .map_err(error_to_string)
}
