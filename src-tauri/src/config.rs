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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub data_mode: DataMode,
    #[serde(default)]
    pub benchmark_engine: BenchmarkEngineMode,
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
        }
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
