use super::{now, MockDataStore};
use crate::error::AppError;
use crate::models::{BenchmarkErrorRecord, ReportDetail, ReportErrorBucket, ReportSummary};
use crate::report::analyzer::{self, ReportContext};
use uuid::Uuid;

impl MockDataStore {
    pub async fn generate_report(&self, task_id: &str) -> anyhow::Result<ReportSummary> {
        let mut data = self.inner.write().await;
        let task = data
            .tasks
            .iter()
            .find(|task| task.summary.id == task_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("task"))?;
        if task.summary.status != "completed" {
            return Err(AppError::invalid_task_state(format!(
                "Only completed benchmark tasks can generate reports, current status: {}",
                task.summary.status
            ))
            .into());
        }
        let stages = data.stages.get(task_id).cloned().unwrap_or_default();
        let capacity = analyzer::capacity_from_stages(
            &stages,
            task.summary.concurrency,
            task.summary.p95_latency_ms.max(1),
            task.summary.success_rate,
        );
        let report = ReportSummary {
            id: Uuid::new_v4().to_string(),
            task_id: task_id.to_string(),
            model_name: task.summary.model_name.clone(),
            provider_name: task.summary.provider_name.clone(),
            recommendation: analyzer::build_recommendation_text(
                &task.summary.model_name,
                &capacity,
            ),
            recommended_concurrency: capacity.recommended_concurrency,
            max_stable_concurrency: capacity.max_stable_concurrency,
            p95_latency_ms: capacity.p95_latency_ms,
            success_rate: capacity.success_rate,
            created_at: now(),
        };
        data.reports.push(report.clone());
        Ok(report)
    }

    pub async fn list_reports(&self) -> anyhow::Result<Vec<ReportSummary>> {
        let data = self.inner.read().await;
        Ok(data.reports.iter().rev().cloned().collect())
    }

    pub async fn get_report_detail(&self, report_id: &str) -> anyhow::Result<ReportDetail> {
        let data = self.inner.read().await;
        let summary = data
            .reports
            .iter()
            .find(|report| report.id == report_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("report"))?;
        let task = data
            .tasks
            .iter()
            .find(|task| task.summary.id == summary.task_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("task"))?;
        let stages = data
            .stages
            .get(&summary.task_id)
            .cloned()
            .unwrap_or_default();
        let trends = data
            .ticks
            .get(&summary.task_id)
            .cloned()
            .unwrap_or_default();
        let dataset_quality = task
            .preflight_result
            .as_ref()
            .and_then(|value| value.get("dataset_quality").cloned())
            .and_then(|value| serde_json::from_value(value).ok());
        let context = ReportContext {
            model_type: task.summary.model_type,
            task_name: task.summary.name,
            dataset_name: task.summary.dataset_name,
            mode: task.mode,
            duration_seconds: task.duration_seconds,
            planned_stages: task.planned_stages,
            stage_sample_rounds: task.stage_sample_rounds,
            warmup_rounds: task.warmup_rounds,
            request_timeout_seconds: task.request_timeout_seconds,
            sla_stop_policy: task.sla_stop_policy,
            sla_p95_ms: task.sla_p95_ms,
            min_success_rate: task.min_success_rate,
            workload_config: task.workload_config,
            preflight_result: task.preflight_result,
            diagnostics_snapshot: task.diagnostics_snapshot,
            dataset_quality,
        };

        let source = if task.engine_mode == "openai_compatible" {
            "measured"
        } else {
            "mock"
        };
        let mut detail = analyzer::build_report_detail(summary, context, stages, trends, source);
        if source == "measured" {
            let errors = error_buckets(data.errors.get(&detail.summary.task_id));
            detail.errors = errors;
        }

        Ok(detail)
    }
}

fn error_buckets(errors: Option<&Vec<BenchmarkErrorRecord>>) -> Vec<ReportErrorBucket> {
    let Some(errors) = errors else {
        return Vec::new();
    };

    let mut buckets = std::collections::BTreeMap::<String, i64>::new();
    for error in errors {
        *buckets.entry(error.error_kind.clone()).or_default() += error.count;
    }
    let total = buckets.values().sum::<i64>();
    if total <= 0 {
        return Vec::new();
    }

    buckets
        .into_iter()
        .map(|(label, value)| ReportErrorBucket {
            label,
            value,
            percent: ((value as f64 / total as f64) * 100.0).round() as i64,
        })
        .collect()
}
