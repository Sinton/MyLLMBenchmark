use crate::config::{AppConfig, ConfigUpdateResult};
use crate::error::AppResult;
use crate::state::AppState;

pub async fn get_app_config(state: &AppState) -> AppResult<AppConfig> {
    Ok(state.current_config()?)
}

pub async fn update_app_config(
    state: &AppState,
    config: AppConfig,
) -> AppResult<ConfigUpdateResult> {
    Ok(state.save_config(config)?)
}
