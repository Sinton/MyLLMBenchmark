use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataMode {
    Mock,
    Sqlite,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkEngineMode {
    Mock,
    OpenaiCompatible,
}

impl Default for BenchmarkEngineMode {
    fn default() -> Self {
        Self::Mock
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationPosition {
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

impl Default for NotificationPosition {
    fn default() -> Self {
        Self::TopRight
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub data_mode: DataMode,
    #[serde(default)]
    pub benchmark_engine: BenchmarkEngineMode,
    #[serde(default)]
    pub notification_position: NotificationPosition,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigUpdateResult {
    pub config: AppConfig,
    pub restart_required: bool,
}

#[derive(Debug, Clone)]
pub struct ConfigStore {
    config_dir: PathBuf,
}

impl ConfigStore {
    pub fn new(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    pub fn load_or_create(&self) -> anyhow::Result<AppConfig> {
        AppConfig::load_or_create(&self.config_dir)
    }

    pub fn save(&self, config: &AppConfig) -> anyhow::Result<()> {
        let path = self.config_dir.join("config.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&path, serde_json::to_string_pretty(config)?)?;
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_mode: DataMode::Mock,
            benchmark_engine: BenchmarkEngineMode::Mock,
            notification_position: NotificationPosition::TopRight,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, DataMode, NotificationPosition};

    #[test]
    fn old_config_defaults_notification_position_to_top_right() {
        let config: AppConfig =
            serde_json::from_str(r#"{"data_mode":"sqlite","benchmark_engine":"mock"}"#).unwrap();

        assert_eq!(config.data_mode, DataMode::Sqlite);
        assert_eq!(config.notification_position, NotificationPosition::TopRight);
    }

    #[test]
    fn notification_position_uses_kebab_case_values() {
        let config: AppConfig = serde_json::from_str(
            r#"{"data_mode":"mock","benchmark_engine":"mock","notification_position":"bottom-left"}"#,
        )
        .unwrap();

        assert_eq!(
            config.notification_position,
            NotificationPosition::BottomLeft
        );
    }
}

impl AppConfig {
    pub fn load_or_create(config_dir: &Path) -> anyhow::Result<Self> {
        let path = config_dir.join("config.json");
        if !path.exists() {
            let config = Self::default();
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, serde_json::to_string_pretty(&config)?)?;
            return Ok(config);
        }

        let content = std::fs::read_to_string(&path)?;
        let config = serde_json::from_str::<Self>(&content)?;
        Ok(config)
    }

    pub fn uses_mock_data(&self) -> bool {
        self.data_mode == DataMode::Mock
    }
}
