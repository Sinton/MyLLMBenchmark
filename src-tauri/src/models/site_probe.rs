use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SiteProbeRunInput {
    pub name: Option<String>,
    pub base_url: String,
    pub api_key: Option<String>,
    pub interface_type: String,
    pub model: String,
    pub prompt: String,
    pub streaming: bool,
    pub max_output_tokens: Option<i64>,
    pub timeout_seconds: Option<i64>,
    pub save_body: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct SiteProbeRunSummary {
    pub id: String,
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
}

#[derive(Debug, Clone, Serialize)]
pub struct SiteProbeRunDetail {
    #[serde(flatten)]
    pub summary: SiteProbeRunSummary,
    pub prompt: Option<String>,
    pub response_text: Option<String>,
    pub request_payload: Option<serde_json::Value>,
    pub raw_error: Option<String>,
    pub raw_usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct SiteProbeRunRecord {
    pub summary: SiteProbeRunSummary,
    pub body_ref: Option<String>,
    pub prompt: Option<String>,
    pub response_text: Option<String>,
    pub request_payload: Option<serde_json::Value>,
    pub raw_error: Option<String>,
    pub raw_usage: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SiteProbeHistoryPageInput {
    pub page: i64,
    pub page_size: i64,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

impl SiteProbeHistoryPageInput {
    pub fn normalized(&self) -> Self {
        let page_size = match self.page_size {
            20 | 50 | 100 => self.page_size,
            value if value <= 0 => 20,
            value => value.min(100),
        };
        Self {
            page: self.page.max(1),
            page_size,
            status: self
                .status
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            keyword: self
                .keyword
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SiteProbeHistoryPage {
    pub items: Vec<SiteProbeRunSummary>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
