use super::rows::{report_from_row, stage_from_row, TaskMeta};
use super::{now, Database};
use crate::domain::model_type::normalize_model_type;
use crate::domain::workload::parse_workload_config;
use crate::error::AppError;
use crate::models::{ReportDetail, ReportErrorBucket, ReportStageSummary, ReportSummary};
use crate::report::analyzer::{self, ReportContext};
use sqlx::Row;
use uuid::Uuid;

impl Database {
    pub async fn generate_report(&self, task_id: &str) -> anyhow::Result<ReportSummary> {
        let task = self.get_task_summary(task_id).await?;
        if task.status != "completed" {
            return Err(AppError::invalid_task_state(format!(
                "只有已完成的压测任务可以生成报告，当前任务状态为 {}",
                task.status
            ))
            .into());
        }
        let id = Uuid::new_v4().to_string();
        let task_meta = self.get_task_meta(task_id).await?;
        let stages = self
            .list_stage_summaries(task_id, task_meta.sla_p95_ms, task_meta.min_success_rate)
            .await?;
        let capacity = analyzer::capacity_from_stages(
            &stages,
            task.concurrency,
            if task.p95_latency_ms == 0 {
                2100
            } else {
                task.p95_latency_ms
            },
            if task.success_rate == 0.0 {
                99.82
            } else {
                task.success_rate
            },
        );
        let recommendation = analyzer::build_recommendation_text(&task.model_name, &capacity);
        let now = now();

        sqlx::query(
            "INSERT INTO reports
             (id, task_id, model_name, provider_name, recommendation, recommended_concurrency,
              max_stable_concurrency, p95_latency_ms, success_rate, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
        )
        .bind(&id)
        .bind(task_id)
        .bind(&task.model_name)
        .bind(&task.provider_name)
        .bind(&recommendation)
        .bind(capacity.recommended_concurrency)
        .bind(capacity.max_stable_concurrency)
        .bind(capacity.p95_latency_ms)
        .bind(capacity.success_rate)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        self.get_report(&id).await
    }

    pub async fn get_report(&self, report_id: &str) -> anyhow::Result<ReportSummary> {
        let row = sqlx::query(
            "SELECT id, task_id, model_name, provider_name, recommendation,
                    recommended_concurrency, max_stable_concurrency, p95_latency_ms,
                    success_rate, created_at
             FROM reports
             WHERE id = ?;",
        )
        .bind(report_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(report_from_row)
            .ok_or_else(|| AppError::not_found("report").into())
    }

    pub async fn list_reports(&self) -> anyhow::Result<Vec<ReportSummary>> {
        self.list_reports_limit(100).await
    }

    pub(super) async fn list_reports_limit(
        &self,
        limit: i64,
    ) -> anyhow::Result<Vec<ReportSummary>> {
        let rows = sqlx::query(
            "SELECT id, task_id, model_name, provider_name, recommendation,
                    recommended_concurrency, max_stable_concurrency, p95_latency_ms,
                    success_rate, created_at
             FROM reports
             ORDER BY created_at DESC
             LIMIT ?;",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(report_from_row).collect())
    }

    pub async fn get_report_detail(&self, report_id: &str) -> anyhow::Result<ReportDetail> {
        let summary = self.get_report(report_id).await?;
        let task_meta = self.get_task_meta(&summary.task_id).await?;
        let raw_stages = self
            .list_stage_summaries(
                &summary.task_id,
                task_meta.sla_p95_ms,
                task_meta.min_success_rate,
            )
            .await?;
        let trends = self.list_ticks(&summary.task_id).await?;
        let source = if task_meta.engine_mode == "openai_compatible" {
            "measured"
        } else {
            "mock"
        };
        let dataset_quality = task_meta
            .preflight_result
            .as_ref()
            .and_then(|value| value.get("dataset_quality").cloned())
            .and_then(|value| serde_json::from_value(value).ok());
        let context = ReportContext {
            model_type: task_meta.model_type,
            task_name: task_meta.task_name,
            dataset_name: task_meta.dataset_name,
            mode: task_meta.mode,
            duration_seconds: task_meta.duration_seconds,
            planned_stages: task_meta.planned_stages,
            stage_sample_rounds: task_meta.stage_sample_rounds,
            warmup_rounds: task_meta.warmup_rounds,
            request_timeout_seconds: task_meta.request_timeout_seconds,
            sla_stop_policy: task_meta.sla_stop_policy,
            sla_p95_ms: task_meta.sla_p95_ms,
            min_success_rate: task_meta.min_success_rate,
            workload_config: task_meta.workload_config,
            preflight_result: task_meta.preflight_result,
            diagnostics_snapshot: task_meta.diagnostics_snapshot,
            dataset_quality,
        };

        let mut detail =
            analyzer::build_report_detail(summary, context, raw_stages, trends, source);
        if source == "measured" {
            let errors = self.list_error_buckets(&detail.summary.task_id).await?;
            detail.errors = errors;
        }
        Ok(detail)
    }

    async fn get_task_meta(&self, task_id: &str) -> anyhow::Result<TaskMeta> {
        let row = sqlx::query(
            "SELECT t.name AS task_name, t.mode, t.duration_seconds, t.planned_stages,
                    t.stage_sample_rounds, t.warmup_rounds, t.request_timeout_seconds,
                    t.sla_stop_policy, t.sla_p95_ms, t.min_success_rate,
                    t.workload_config, t.preflight_result, t.diagnostics_snapshot,
                    d.name AS dataset_name,
                    t.engine_mode,
                    COALESCE(m.model_type, 'text_generation') AS model_type
             FROM benchmark_tasks t
             JOIN datasets d ON d.id = t.dataset_id
             LEFT JOIN models m ON m.id = t.model_id
             WHERE t.id = ?;",
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        let row = row.ok_or_else(|| AppError::not_found("task"))?;

        let model_type = normalize_model_type(&row.get::<String, _>("model_type"));
        let workload_config =
            parse_workload_config(row.get::<String, _>("workload_config"), &model_type);
        let planned_stages =
            serde_json::from_str::<Vec<i64>>(&row.get::<String, _>("planned_stages"))
                .unwrap_or_default();
        let preflight_result = row
            .get::<Option<String>, _>("preflight_result")
            .and_then(|value| serde_json::from_str::<serde_json::Value>(&value).ok());
        let diagnostics_snapshot = row
            .get::<Option<String>, _>("diagnostics_snapshot")
            .and_then(|value| serde_json::from_str(&value).ok());

        Ok(TaskMeta {
            task_name: row.get("task_name"),
            mode: row.get("mode"),
            dataset_name: row.get("dataset_name"),
            duration_seconds: row.get("duration_seconds"),
            planned_stages,
            stage_sample_rounds: row.get("stage_sample_rounds"),
            warmup_rounds: row.get("warmup_rounds"),
            request_timeout_seconds: row.get("request_timeout_seconds"),
            sla_stop_policy: row.get("sla_stop_policy"),
            sla_p95_ms: row.get("sla_p95_ms"),
            min_success_rate: row.get("min_success_rate"),
            model_type,
            engine_mode: row.get("engine_mode"),
            workload_config,
            preflight_result,
            diagnostics_snapshot,
        })
    }

    async fn list_error_buckets(&self, task_id: &str) -> anyhow::Result<Vec<ReportErrorBucket>> {
        let rows = sqlx::query(
            "SELECT error_kind, SUM(count) AS value
             FROM benchmark_errors
             WHERE task_id = ?
             GROUP BY error_kind
             ORDER BY value DESC;",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        let total = rows
            .iter()
            .map(|row| row.get::<i64, _>("value"))
            .sum::<i64>();
        if total <= 0 {
            return Ok(Vec::new());
        }

        Ok(rows
            .into_iter()
            .map(|row| {
                let value = row.get::<i64, _>("value");
                ReportErrorBucket {
                    label: row.get("error_kind"),
                    value,
                    percent: ((value as f64 / total as f64) * 100.0).round() as i64,
                }
            })
            .collect())
    }

    async fn list_stage_summaries(
        &self,
        task_id: &str,
        sla_p95_ms: i64,
        min_success_rate: f64,
    ) -> anyhow::Result<Vec<ReportStageSummary>> {
        let rows = sqlx::query(
            "SELECT stage_index, concurrency, goodput_qps, p95_latency_ms, ttft_ms, tps,
                    success_rate, error_rate, input_tokens, output_tokens, total_tokens,
                    batch_size, text_count, documents_per_query, pair_count, image_count,
                    sample_rounds, warmup_rounds, request_count, success_count, failure_count,
                    sla_passed, stop_reason
             FROM benchmark_stages
             WHERE task_id = ?
             ORDER BY stage_index ASC;",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| stage_from_row(row, sla_p95_ms, min_success_rate))
            .collect())
    }
}
