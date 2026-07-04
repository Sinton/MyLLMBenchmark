use super::{DatasetValidationResult, MetricsTick, ProviderDiagnosticsResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ReportSummary {
    pub id: String,
    pub task_id: String,
    pub model_name: String,
    pub provider_name: String,
    pub recommendation: String,
    pub recommended_concurrency: i64,
    pub max_stable_concurrency: i64,
    pub p95_latency_ms: i64,
    pub success_rate: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportStageSummary {
    pub stage_index: i64,
    pub concurrency: i64,
    pub sample_rounds: i64,
    pub warmup_rounds: i64,
    pub request_count: i64,
    pub success_count: i64,
    pub failure_count: i64,
    pub qps: f64,
    pub p95_latency_ms: i64,
    pub ttft_ms: i64,
    pub tps: f64,
    pub success_rate: f64,
    pub error_rate: f64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub batch_size: i64,
    pub text_count: i64,
    pub documents_per_query: i64,
    pub pair_count: i64,
    pub image_count: i64,
    pub sla_passed: bool,
    pub stop_reason: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportErrorBucket {
    pub label: String,
    pub value: i64,
    pub percent: i64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ReportRequestLogMeta {
    pub enabled: bool,
    pub total_records: i64,
    pub body_records: i64,
    pub body_available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportSpecialtyMetric {
    pub label: String,
    pub value: serde_json::Value,
    pub unit: Option<String>,
    pub hint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportSpecialtySection {
    pub title: String,
    pub description: String,
    pub metrics: Vec<ReportSpecialtyMetric>,
    pub guidance: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportDetail {
    pub summary: ReportSummary,
    pub source: String,
    pub model_type: String,
    pub task_name: String,
    pub dataset_name: String,
    pub mode: String,
    pub duration_seconds: i64,
    pub planned_stages: Vec<i64>,
    pub executed_stages: Vec<i64>,
    pub stage_sample_rounds: i64,
    pub warmup_rounds: i64,
    pub request_timeout_seconds: i64,
    pub sla_stop_policy: String,
    pub early_stop_reason: Option<String>,
    pub sla_p95_ms: i64,
    pub min_success_rate: f64,
    pub verdict: String,
    pub verdict_label: String,
    pub bottleneck: String,
    pub capacity_conclusion: String,
    pub stable_qps: f64,
    pub ttft_ms: i64,
    pub tps: f64,
    pub token_throughput: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub stages: Vec<ReportStageSummary>,
    pub trends: Vec<MetricsTick>,
    pub errors: Vec<ReportErrorBucket>,
    pub specialty: ReportSpecialtySection,
    pub recommendations: Vec<String>,
    pub workload_config: serde_json::Value,
    pub preflight_result: Option<serde_json::Value>,
    pub diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
    pub dataset_quality: Option<DatasetValidationResult>,
    pub request_log_meta: ReportRequestLogMeta,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReportExportInput {
    pub report_id: String,
    pub format: String,
    pub template: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportExportResult {
    pub report_id: String,
    pub format: String,
    pub file_name: String,
    pub file_path: String,
    pub mime_type: String,
    pub message: String,
}
