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
