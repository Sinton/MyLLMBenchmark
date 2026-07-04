use crate::models::{
    DatasetValidationResult, MetricsTick, ProviderDiagnosticsResult, ReportDetail,
    ReportErrorBucket, ReportRequestLogMeta, ReportStageSummary, ReportSummary,
};
use crate::report::estimators::{
    estimate_stages_from_summary, estimate_ticks_from_summary, hydrate_stage_metrics,
    stage_has_missing_llm_metrics,
};
use crate::report::narrative::{
    bottleneck_for, build_recommendations, capacity_conclusion, verdict_for,
};
use crate::report::specialty::{build_specialty_section, SpecialtyInput};

pub use crate::report::capacity::{
    build_recommendation_text, capacity_from_stages, stage_status, CapacityRecommendation,
};

#[derive(Debug, Clone)]
pub struct ReportContext {
    pub model_type: String,
    pub task_name: String,
    pub dataset_name: String,
    pub mode: String,
    pub duration_seconds: i64,
    pub planned_stages: Vec<i64>,
    pub stage_sample_rounds: i64,
    pub warmup_rounds: i64,
    pub request_timeout_seconds: i64,
    pub sla_stop_policy: String,
    pub sla_p95_ms: i64,
    pub min_success_rate: f64,
    pub workload_config: serde_json::Value,
    pub preflight_result: Option<serde_json::Value>,
    pub diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
    pub dataset_quality: Option<DatasetValidationResult>,
    pub request_log_meta: ReportRequestLogMeta,
}

pub fn build_report_detail(
    summary: ReportSummary,
    context: ReportContext,
    raw_stages: Vec<ReportStageSummary>,
    mut trends: Vec<MetricsTick>,
    measured_source: &str,
) -> ReportDetail {
    let has_incomplete_stage_metrics = raw_stages
        .iter()
        .any(|stage| stage_has_missing_llm_metrics(stage, &context.model_type));
    let source = if trends.is_empty() || raw_stages.is_empty() || has_incomplete_stage_metrics {
        trends = estimate_ticks_from_summary(&summary, &context.model_type);
        "estimated".to_string()
    } else {
        measured_source.to_string()
    };
    let stages = if raw_stages.is_empty() {
        estimate_stages_from_summary(&summary, &context.model_type)
    } else {
        hydrate_stage_metrics(raw_stages, &summary, &context.model_type)
    };
    let stable_stage = stages
        .iter()
        .filter(|stage| stage.status == "stable")
        .max_by_key(|stage| stage.concurrency)
        .or_else(|| stages.first());
    let latest_tick = trends.last();
    let ttft_ms = latest_tick
        .map(|tick| tick.ttft_ms)
        .or_else(|| stable_stage.map(|stage| stage.ttft_ms))
        .unwrap_or(0);
    let tps = latest_tick
        .map(|tick| tick.tps)
        .or_else(|| stable_stage.map(|stage| stage.tps))
        .unwrap_or(0.0);
    let token_throughput = latest_tick
        .map(|tick| tick.total_tokens)
        .or_else(|| stable_stage.map(|stage| stage.total_tokens))
        .unwrap_or(0);
    let input_tokens = latest_tick
        .map(|tick| tick.input_tokens)
        .or_else(|| stable_stage.map(|stage| stage.input_tokens))
        .unwrap_or(0);
    let output_tokens = latest_tick
        .map(|tick| tick.output_tokens)
        .or_else(|| stable_stage.map(|stage| stage.output_tokens))
        .unwrap_or(0);
    let stable_qps = stable_stage
        .map(|stage| stage.qps)
        .unwrap_or(summary.recommended_concurrency as f64);
    let (verdict, verdict_label) = verdict_for(
        summary.p95_latency_ms,
        summary.success_rate,
        context.sla_p95_ms,
        context.min_success_rate,
    );
    let bottleneck = bottleneck_for(
        &context.model_type,
        summary.p95_latency_ms,
        ttft_ms,
        tps,
        summary.success_rate,
    );
    let capacity_conclusion =
        capacity_conclusion(&summary, context.sla_p95_ms, context.min_success_rate);
    let errors = if measured_source == "measured" {
        Vec::new()
    } else {
        build_error_buckets(&trends, summary.success_rate)
    };
    let specialty = build_specialty_section(SpecialtyInput {
        model_type: &context.model_type,
        workload_config: &context.workload_config,
        ttft_ms,
        tps,
        token_throughput,
        input_tokens,
        output_tokens,
        stable_qps,
        p95_latency_ms: summary.p95_latency_ms,
        success_rate: summary.success_rate,
    });
    let recommendations = build_recommendations(&context.model_type, &summary, context.sla_p95_ms);

    let executed_stages = stages
        .iter()
        .map(|stage| stage.concurrency)
        .collect::<Vec<_>>();
    let planned_stages = if context.planned_stages.is_empty() {
        executed_stages.clone()
    } else {
        context.planned_stages.clone()
    };
    let early_stop_reason = build_stop_reason(&planned_stages, &stages);

    ReportDetail {
        summary,
        source,
        model_type: context.model_type,
        task_name: context.task_name,
        dataset_name: context.dataset_name,
        mode: context.mode,
        duration_seconds: context.duration_seconds,
        planned_stages,
        executed_stages,
        stage_sample_rounds: context.stage_sample_rounds,
        warmup_rounds: context.warmup_rounds,
        request_timeout_seconds: context.request_timeout_seconds,
        sla_stop_policy: context.sla_stop_policy,
        early_stop_reason,
        sla_p95_ms: context.sla_p95_ms,
        min_success_rate: context.min_success_rate,
        verdict,
        verdict_label,
        bottleneck,
        capacity_conclusion,
        stable_qps,
        ttft_ms,
        tps,
        token_throughput,
        input_tokens,
        output_tokens,
        stages,
        trends,
        errors,
        specialty,
        recommendations,
        workload_config: context.workload_config,
        preflight_result: context.preflight_result,
        diagnostics_snapshot: context.diagnostics_snapshot,
        dataset_quality: context.dataset_quality,
        request_log_meta: context.request_log_meta,
    }
}

fn build_stop_reason(planned_stages: &[i64], stages: &[ReportStageSummary]) -> Option<String> {
    if let Some(reason) = stages
        .iter()
        .rev()
        .find_map(|stage| stage.stop_reason.clone())
    {
        if stages.len() < planned_stages.len() {
            return Some(format!("压测提前停止：{reason}"));
        }
        return Some(reason);
    }

    if stages.len() < planned_stages.len() {
        return Some(format!(
            "计划执行 {} 个阶段，实际记录 {} 个阶段；历史数据未记录明确停止原因。",
            planned_stages.len(),
            stages.len()
        ));
    }

    None
}

fn build_error_buckets(trends: &[MetricsTick], success_rate: f64) -> Vec<ReportErrorBucket> {
    let total_errors: i64 = trends.iter().map(|tick| tick.errors).sum();
    let fallback = ((100.0 - success_rate).max(0.0) * 10.0).round() as i64;
    let total = total_errors.max(fallback);
    if total <= 0 {
        return Vec::new();
    }
    let timeout = ((total as f64) * 0.64).round() as i64;
    let http500 = ((total as f64) * 0.23).round() as i64;
    let reset = (total - timeout - http500).max(0);
    [
        ("Timeout", timeout),
        ("HTTP 5xx", http500),
        ("Connection Reset", reset),
    ]
    .into_iter()
    .map(|(label, value)| ReportErrorBucket {
        label: label.to_string(),
        value,
        percent: ((value as f64 / total as f64) * 100.0).round() as i64,
    })
    .collect()
}
