use crate::models::{
    BenchmarkErrorRecord, BenchmarkRequestLogDetail, BenchmarkTaskSummary, DatasetSample,
    DatasetSummary, EndpointProbeBatchSummary, EndpointProbeRunDetail, MetricsTick, ModelSummary,
    ProviderDiagnosticsResult, ProviderSummary, ReportStageSummary, ReportSummary,
};
use std::collections::HashMap;

#[derive(Clone)]
pub(in crate::mock) struct MockTaskRecord {
    pub(in crate::mock) summary: BenchmarkTaskSummary,
    pub(in crate::mock) provider_id: String,
    pub(in crate::mock) mode: String,
    pub(in crate::mock) duration_seconds: i64,
    pub(in crate::mock) planned_stages: Vec<i64>,
    pub(in crate::mock) stage_sample_rounds: i64,
    pub(in crate::mock) warmup_rounds: i64,
    pub(in crate::mock) request_timeout_seconds: i64,
    pub(in crate::mock) sla_stop_policy: String,
    pub(in crate::mock) workload_config: serde_json::Value,
    pub(in crate::mock) engine_mode: String,
    pub(in crate::mock) sla_p95_ms: i64,
    pub(in crate::mock) min_success_rate: f64,
    pub(in crate::mock) preflight_result: Option<serde_json::Value>,
    pub(in crate::mock) diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
}

#[derive(Default)]
pub(in crate::mock) struct MockData {
    pub(in crate::mock) providers: Vec<ProviderSummary>,
    pub(in crate::mock) provider_base_urls: HashMap<String, String>,
    pub(in crate::mock) provider_api_keys: HashMap<String, String>,
    pub(in crate::mock) provider_diagnostics: HashMap<String, ProviderDiagnosticsResult>,
    pub(in crate::mock) models: Vec<ModelSummary>,
    pub(in crate::mock) datasets: Vec<DatasetSummary>,
    pub(in crate::mock) dataset_samples: HashMap<String, Vec<DatasetSample>>,
    pub(in crate::mock) tasks: Vec<MockTaskRecord>,
    pub(in crate::mock) stages: HashMap<String, Vec<ReportStageSummary>>,
    pub(in crate::mock) ticks: HashMap<String, Vec<MetricsTick>>,
    pub(in crate::mock) errors: HashMap<String, Vec<BenchmarkErrorRecord>>,
    pub(in crate::mock) request_logs: HashMap<String, Vec<BenchmarkRequestLogDetail>>,
    pub(in crate::mock) endpoint_probe_batches: Vec<EndpointProbeBatchSummary>,
    pub(in crate::mock) endpoint_probe_runs: Vec<EndpointProbeRunDetail>,
    pub(in crate::mock) reports: Vec<ReportSummary>,
}
