use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BenchmarkStartInput {
    pub provider_id: String,
    pub model_id: Option<String>,
    pub dataset_id: String,
    pub mode: String,
    pub concurrency: i64,
    pub duration_seconds: i64,
    pub start_concurrency: Option<i64>,
    pub end_concurrency: Option<i64>,
    pub step_strategy: Option<String>,
    pub step_value: Option<i64>,
    pub stage_sample_rounds: Option<i64>,
    pub stage_duration_seconds: Option<i64>,
    pub warmup_rounds: Option<i64>,
    pub warmup_seconds: Option<i64>,
    pub request_timeout_seconds: Option<i64>,
    pub sla_p95_ms: Option<i64>,
    pub min_success_rate: Option<f64>,
    pub sla_stop_policy: Option<String>,
    pub workload_config: Option<serde_json::Value>,
    pub request_log_config: Option<RequestLogConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestLogConfig {
    pub enabled: bool,
    pub capture_body: bool,
    pub max_records_per_stage: i64,
}

impl RequestLogConfig {
    pub fn normalized(input: Option<&Self>) -> Self {
        let Some(input) = input else {
            return Self {
                enabled: false,
                capture_body: false,
                max_records_per_stage: 200,
            };
        };
        Self {
            enabled: input.enabled,
            capture_body: input.enabled && input.capture_body,
            max_records_per_stage: input.max_records_per_stage.clamp(1, 1000),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkTaskSummary {
    pub id: String,
    pub name: String,
    pub status: String,
    pub model_type: String,
    pub model_name: String,
    pub provider_name: String,
    pub dataset_name: String,
    pub concurrency: i64,
    pub success_rate: f64,
    pub p95_latency_ms: i64,
    pub goodput_qps: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct StopResult {
    pub task_id: String,
    pub stopped: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricsTick {
    pub task_id: String,
    pub elapsed_seconds: i64,
    pub qps: f64,
    pub latency_ms: i64,
    pub ttft_ms: i64,
    pub tps: f64,
    pub success_rate: f64,
    pub errors: i64,
    pub in_flight: i64,
    pub request_count: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub batch_size: i64,
    pub text_count: i64,
    pub documents_per_query: i64,
    pub pair_count: i64,
    pub image_count: i64,
}

#[derive(Debug, Clone)]
pub struct BenchmarkErrorRecord {
    pub task_id: String,
    pub error_kind: String,
    pub message: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkRequestLogSummary {
    pub id: String,
    pub task_id: String,
    pub stage_index: i64,
    pub request_index: i64,
    pub sample_index: i64,
    pub status: String,
    pub latency_ms: i64,
    pub ttft_ms: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub error_kind: Option<String>,
    pub prompt_preview: Option<String>,
    pub response_preview: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct BenchmarkRequestLogDetail {
    #[serde(flatten)]
    pub summary: BenchmarkRequestLogSummary,
    pub prompt: Option<String>,
    pub response_text: Option<String>,
    pub raw_error: Option<String>,
    pub raw_usage: Option<serde_json::Value>,
    pub body_available: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BenchmarkRequestLogPageInput {
    pub task_id: String,
    pub page: i64,
    pub page_size: i64,
    pub stage_index: Option<i64>,
    pub status: Option<String>,
    pub keyword: Option<String>,
}

impl BenchmarkRequestLogPageInput {
    pub fn normalized(&self) -> Self {
        let page_size = match self.page_size {
            20 | 50 | 100 | 200 => self.page_size,
            value if value <= 0 => 50,
            value => value.min(200),
        };
        Self {
            task_id: self.task_id.clone(),
            page: self.page.max(1),
            page_size,
            stage_index: self.stage_index,
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
pub struct BenchmarkRequestLogPage {
    pub items: Vec<BenchmarkRequestLogSummary>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone)]
pub struct BenchmarkRequestLogRecord {
    pub summary: BenchmarkRequestLogSummary,
    pub body_ref: Option<String>,
    pub prompt: Option<String>,
    pub response_text: Option<String>,
    pub raw_error: Option<String>,
    pub raw_usage: Option<serde_json::Value>,
}
