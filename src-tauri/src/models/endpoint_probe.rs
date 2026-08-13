use super::{DiscoveredModel, ProviderSummary};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum EndpointProbeTargetInput {
    Provider {
        provider_id: String,
        models: Vec<String>,
    },
    Temporary {
        name: Option<String>,
        base_url: String,
        api_key: Option<String>,
        interface_type: String,
        models: Vec<String>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndpointProbeStartInput {
    pub targets: Vec<EndpointProbeTargetInput>,
    pub prompt: String,
    pub streaming: bool,
    pub temperature: Option<f64>,
    pub max_output_tokens: Option<i64>,
    pub timeout_seconds: Option<i64>,
    pub save_body: bool,
    pub concurrency: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum EndpointProbeModelScanInput {
    Provider {
        provider_id: String,
    },
    Temporary {
        base_url: String,
        api_key: Option<String>,
        interface_type: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbeModelOption {
    pub name: String,
    pub model_type: String,
    pub capabilities: Vec<String>,
    pub supports_streaming: bool,
}

impl From<DiscoveredModel> for EndpointProbeModelOption {
    fn from(model: DiscoveredModel) -> Self {
        Self {
            name: model.name,
            model_type: model.model_type,
            capabilities: model.capabilities,
            supports_streaming: model.supports_streaming,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbeModelScanResult {
    pub provider_id: Option<String>,
    pub models: Vec<EndpointProbeModelOption>,
    pub message: String,
    pub scanned_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbeBatchSummary {
    pub id: String,
    pub name: String,
    pub status: String,
    pub total_runs: i64,
    pub pending_runs: i64,
    pub running_runs: i64,
    pub passed_runs: i64,
    pub failed_runs: i64,
    pub cancelled_runs: i64,
    pub streaming: bool,
    pub temperature: f64,
    pub max_output_tokens: i64,
    pub timeout_seconds: i64,
    pub save_body: bool,
    pub concurrency: i64,
    pub prompt_preview: Option<String>,
    pub created_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbeRunSummary {
    pub id: String,
    pub batch_id: String,
    pub source_type: String,
    pub provider_id: Option<String>,
    pub name: String,
    pub base_url: String,
    pub interface_type: String,
    pub model: String,
    pub status: String,
    pub latency_ms: i64,
    pub ttft_ms: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub error_kind: Option<String>,
    pub error_message: Option<String>,
    pub prompt_preview: Option<String>,
    pub response_preview: Option<String>,
    pub body_available: bool,
    pub created_at: String,
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbeRunDetail {
    #[serde(flatten)]
    pub summary: EndpointProbeRunSummary,
    pub prompt: Option<String>,
    pub response_text: Option<String>,
    pub request_payload: Option<serde_json::Value>,
    pub raw_error: Option<String>,
    pub raw_usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbeBatchDetail {
    #[serde(flatten)]
    pub summary: EndpointProbeBatchSummary,
    pub runs: Vec<EndpointProbeRunSummary>,
}

#[derive(Debug, Clone)]
pub struct EndpointProbeBatchRecord {
    pub summary: EndpointProbeBatchSummary,
}

#[derive(Debug, Clone)]
pub struct EndpointProbeRunRecord {
    pub summary: EndpointProbeRunSummary,
    pub body_ref: Option<String>,
    pub prompt: Option<String>,
    pub response_text: Option<String>,
    pub request_payload: Option<serde_json::Value>,
    pub raw_error: Option<String>,
    pub raw_usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndpointProbeHistoryPageInput {
    pub page: i64,
    pub page_size: i64,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

impl EndpointProbeHistoryPageInput {
    pub fn normalized(&self) -> Self {
        let page_size = match self.page_size {
            20 | 50 | 100 => self.page_size,
            value if value <= 0 => 20,
            value => value.min(100),
        };
        Self {
            page: self.page.max(1),
            page_size,
            status: normalize_optional(&self.status),
            keyword: normalize_optional(&self.keyword),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbeHistoryPage {
    pub items: Vec<EndpointProbeBatchSummary>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbeStopResult {
    pub batch_id: String,
    pub stopped: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndpointProbePromotionInput {
    pub run_id: String,
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub sync_models: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbePromotionResult {
    pub status: String,
    pub provider: ProviderSummary,
    pub models_synced: bool,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbeRunStartedEvent {
    pub batch_id: String,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbeResponseDeltaEvent {
    pub batch_id: String,
    pub run_id: String,
    pub sequence: u64,
    pub delta: String,
    pub elapsed_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EndpointProbeRunFinishedEvent {
    pub batch_id: String,
    pub run: EndpointProbeRunDetail,
}

fn normalize_optional(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
