use crate::domain::model_type::{default_capabilities, normalize_model_type};
use crate::models::{
    BenchmarkTaskSummary, MetricsTick, ModelSummary, ProviderDiagnosticsResult, ReportStageSummary,
    ReportSummary,
};
use crate::report::analyzer;
use sqlx::{Row, SqlitePool};

pub(super) struct TaskMeta {
    pub task_name: String,
    pub mode: String,
    pub dataset_name: String,
    pub duration_seconds: i64,
    pub planned_stages: Vec<i64>,
    pub stage_sample_rounds: i64,
    pub warmup_rounds: i64,
    pub request_timeout_seconds: i64,
    pub sla_stop_policy: String,
    pub sla_p95_ms: i64,
    pub min_success_rate: f64,
    pub model_type: String,
    pub engine_mode: String,
    pub workload_config: serde_json::Value,
    pub preflight_result: Option<serde_json::Value>,
    pub diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
}

pub(super) async fn count(pool: &SqlitePool, table: &str) -> anyhow::Result<i64> {
    let sql = format!("SELECT COUNT(*) FROM {table}");
    let value = sqlx::query_scalar::<_, i64>(&sql).fetch_one(pool).await?;
    Ok(value)
}

pub(super) fn task_from_row(row: sqlx::sqlite::SqliteRow) -> BenchmarkTaskSummary {
    BenchmarkTaskSummary {
        id: row.get("id"),
        name: row.get("name"),
        status: row.get("status"),
        model_type: normalize_model_type(&row.get::<String, _>("model_type")),
        model_name: row.get("model_name"),
        provider_name: row.get("provider_name"),
        dataset_name: row.get("dataset_name"),
        concurrency: row.get("concurrency"),
        success_rate: row.get("success_rate"),
        p95_latency_ms: row.get("p95_latency_ms"),
        goodput_qps: row.get("goodput_qps"),
        created_at: row.get("created_at"),
    }
}

pub(super) fn report_from_row(row: sqlx::sqlite::SqliteRow) -> ReportSummary {
    ReportSummary {
        id: row.get("id"),
        task_id: row.get("task_id"),
        model_name: row.get("model_name"),
        provider_name: row.get("provider_name"),
        recommendation: row.get("recommendation"),
        recommended_concurrency: row.get("recommended_concurrency"),
        max_stable_concurrency: row.get("max_stable_concurrency"),
        p95_latency_ms: row.get("p95_latency_ms"),
        success_rate: row.get("success_rate"),
        created_at: row.get("created_at"),
    }
}

pub(super) fn stage_from_row(
    row: sqlx::sqlite::SqliteRow,
    sla_p95_ms: i64,
    min_success_rate: f64,
) -> ReportStageSummary {
    let p95_latency_ms = row.get("p95_latency_ms");
    let success_rate = row.get("success_rate");
    let status = analyzer::stage_status(p95_latency_ms, success_rate, sla_p95_ms, min_success_rate);

    ReportStageSummary {
        stage_index: row.get("stage_index"),
        concurrency: row.get("concurrency"),
        sample_rounds: row.get("sample_rounds"),
        warmup_rounds: row.get("warmup_rounds"),
        request_count: row.get("request_count"),
        success_count: row.get("success_count"),
        failure_count: row.get("failure_count"),
        qps: row.get("goodput_qps"),
        p95_latency_ms,
        ttft_ms: row.get("ttft_ms"),
        tps: row.get("tps"),
        success_rate,
        error_rate: row.get("error_rate"),
        input_tokens: row.get("input_tokens"),
        output_tokens: row.get("output_tokens"),
        total_tokens: row.get("total_tokens"),
        batch_size: row.get("batch_size"),
        text_count: row.get("text_count"),
        documents_per_query: row.get("documents_per_query"),
        pair_count: row.get("pair_count"),
        image_count: row.get("image_count"),
        sla_passed: p95_latency_ms <= sla_p95_ms && success_rate >= min_success_rate,
        stop_reason: row.get::<Option<String>, _>("stop_reason"),
        status,
    }
}

pub(super) fn tick_from_row(row: sqlx::sqlite::SqliteRow) -> MetricsTick {
    MetricsTick {
        task_id: row.get("task_id"),
        elapsed_seconds: row.get("elapsed_seconds"),
        qps: row.get("qps"),
        latency_ms: row.get("latency_ms"),
        ttft_ms: row.get("ttft_ms"),
        tps: row.get("tps"),
        success_rate: row.get("success_rate"),
        errors: row.get("errors"),
        in_flight: row.get("in_flight"),
        request_count: row.get("request_count"),
        success_count: row.get("success_count"),
        failure_count: row.get("failure_count"),
        input_tokens: row.get("input_tokens"),
        output_tokens: row.get("output_tokens"),
        total_tokens: row.get("total_tokens"),
        batch_size: row.get("batch_size"),
        text_count: row.get("text_count"),
        documents_per_query: row.get("documents_per_query"),
        pair_count: row.get("pair_count"),
        image_count: row.get("image_count"),
    }
}

pub(super) fn model_from_row(row: sqlx::sqlite::SqliteRow) -> ModelSummary {
    let model_type = normalize_model_type(&row.get::<String, _>("model_type"));
    let capabilities = parse_capabilities(row.get::<String, _>("capabilities"), &model_type);
    ModelSummary {
        id: row.get("id"),
        provider_id: row.get("provider_id"),
        name: row.get("name"),
        model_type,
        capabilities,
        supports_streaming: row.get::<i64, _>("supports_streaming") == 1,
        recommended_concurrency: row.get("recommended_concurrency"),
    }
}

fn parse_capabilities(value: String, model_type: &str) -> Vec<String> {
    let mut capabilities: Vec<String> = serde_json::from_str(&value).unwrap_or_default();
    if capabilities.is_empty() {
        capabilities = default_capabilities(model_type);
    }
    capabilities
}
