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

pub const DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID: &str = "basic-liveness";
pub const DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_NAME: &str = "基础测活";
pub const DEFAULT_ENDPOINT_PROBE_PROMPT: &str = "请回复1+1 是否等于2，回答是或者不是。";

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EndpointProbePromptTemplate {
    pub id: String,
    pub name: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct EndpointProbePromptTemplatesConfig {
    #[serde(default = "default_endpoint_probe_prompt_template_id")]
    pub selected_id: String,
    #[serde(default)]
    pub items: Vec<EndpointProbePromptTemplate>,
}

fn default_endpoint_probe_prompt_template_id() -> String {
    DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID.to_string()
}

impl Default for EndpointProbePromptTemplatesConfig {
    fn default() -> Self {
        Self {
            selected_id: DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID.to_string(),
            items: vec![EndpointProbePromptTemplate {
                id: DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID.to_string(),
                name: DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_NAME.to_string(),
                prompt: DEFAULT_ENDPOINT_PROBE_PROMPT.to_string(),
            }],
        }
    }
}

impl EndpointProbePromptTemplatesConfig {
    pub fn normalized(mut self) -> Self {
        self.items.retain(|item| {
            !item.id.trim().is_empty()
                && !item.name.trim().is_empty()
                && !item.prompt.trim().is_empty()
        });
        if self.items.is_empty() {
            return Self::default();
        }
        if !self.items.iter().any(|item| item.id == self.selected_id) {
            self.selected_id = self
                .items
                .first()
                .map(|item| item.id.clone())
                .unwrap_or_else(|| DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID.to_string());
        }
        self
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub data_mode: DataMode,
    #[serde(default)]
    pub benchmark_engine: BenchmarkEngineMode,
    #[serde(default)]
    pub notification_position: NotificationPosition,
    #[serde(default)]
    pub endpoint_probe_prompt_templates: EndpointProbePromptTemplatesConfig,
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
            endpoint_probe_prompt_templates: EndpointProbePromptTemplatesConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AppConfig, DataMode, EndpointProbePromptTemplatesConfig, NotificationPosition,
        DEFAULT_ENDPOINT_PROBE_PROMPT, DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID,
    };

    #[test]
    fn old_config_defaults_missing_fields() {
        let config: AppConfig =
            serde_json::from_str(r#"{"data_mode":"sqlite","benchmark_engine":"mock"}"#).unwrap();

        assert_eq!(config.data_mode, DataMode::Sqlite);
        assert_eq!(config.notification_position, NotificationPosition::TopRight);
        assert_eq!(
            config.endpoint_probe_prompt_templates.selected_id,
            DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID
        );
        assert_eq!(config.endpoint_probe_prompt_templates.items.len(), 1);
        assert_eq!(
            config.endpoint_probe_prompt_templates.items[0].prompt,
            DEFAULT_ENDPOINT_PROBE_PROMPT
        );
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

    #[test]
    fn endpoint_probe_prompt_templates_normalize_empty_config() {
        let config = EndpointProbePromptTemplatesConfig {
            selected_id: "missing".to_string(),
            items: vec![],
        }
        .normalized();

        assert_eq!(config.items.len(), 1);
        assert_eq!(
            config.selected_id,
            DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID
        );
    }

    #[test]
    fn partial_endpoint_probe_prompt_templates_config_is_accepted() {
        let config: AppConfig =
            serde_json::from_str(r#"{"data_mode":"mock","endpoint_probe_prompt_templates":{}}"#)
                .unwrap();

        assert_eq!(
            config.endpoint_probe_prompt_templates.selected_id,
            DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID
        );
        assert!(config.endpoint_probe_prompt_templates.items.is_empty());
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
        let mut config = serde_json::from_str::<Self>(&content)?;
        config.endpoint_probe_prompt_templates =
            config.endpoint_probe_prompt_templates.normalized();
        Ok(config)
    }

    pub fn uses_mock_data(&self) -> bool {
        self.data_mode == DataMode::Mock
    }
}
