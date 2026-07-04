use super::rows::{task_from_row, tick_from_row};
use super::{now, Database};
use crate::benchmark::plan::BenchmarkPlan;
use crate::domain::benchmark::validate_benchmark_start;
use crate::domain::benchmark_sample::StageSample;
use crate::error::AppError;
use crate::models::{
    BenchmarkErrorRecord, BenchmarkRequestLogDetail, BenchmarkRequestLogPage,
    BenchmarkRequestLogPageInput, BenchmarkRequestLogRecord, BenchmarkRequestLogSummary,
    BenchmarkStartInput, BenchmarkTaskSummary, DeleteResult, MetricsTick,
    ProviderDiagnosticsResult,
};
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, Sqlite};
use uuid::Uuid;

impl Database {
    pub async fn create_task(
        &self,
        input: &BenchmarkStartInput,
    ) -> anyhow::Result<BenchmarkTaskSummary> {
        validate_benchmark_start(input)?;
        self.ensure_benchmark_refs(input).await?;

        let id = Uuid::new_v4().to_string();
        let now = now();
        let model_id = self.resolve_model_id(input).await?;
        let task_name = format!("{} 压测任务", input.mode);
        let plan = BenchmarkPlan::from_input(input);
        let planned_stages = serde_json::to_string(&plan.stages)?;
        let workload_config = serde_json::to_string(
            input
                .workload_config
                .as_ref()
                .unwrap_or(&serde_json::json!({})),
        )?;

        sqlx::query(
            "INSERT INTO benchmark_tasks
             (id, name, provider_id, model_id, dataset_id, mode, concurrency, duration_seconds,
              workload_config, engine_mode, stage_sample_rounds, warmup_rounds,
              request_timeout_seconds, sla_stop_policy, planned_stages, sla_p95_ms,
              min_success_rate, status, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
        )
        .bind(&id)
        .bind(&task_name)
        .bind(&input.provider_id)
        .bind(&model_id)
        .bind(&input.dataset_id)
        .bind(&input.mode)
        .bind(input.concurrency)
        .bind(input.duration_seconds)
        .bind(workload_config)
        .bind("mock")
        .bind(plan.stage_sample_rounds)
        .bind(plan.warmup_rounds)
        .bind(plan.request_timeout_seconds)
        .bind(&plan.sla_stop_policy)
        .bind(planned_stages)
        .bind(input.sla_p95_ms.unwrap_or(5000))
        .bind(input.min_success_rate.unwrap_or(99.0))
        .bind("running")
        .bind(&now)
        .execute(&self.pool)
        .await?;

        self.get_task_summary(&id).await
    }

    async fn resolve_model_id(
        &self,
        input: &BenchmarkStartInput,
    ) -> anyhow::Result<Option<String>> {
        if let Some(model_id) = input.model_id.as_ref().filter(|id| !id.is_empty()) {
            return Ok(Some(model_id.clone()));
        }

        let model_id: Option<String> = sqlx::query_scalar(
            "SELECT id FROM models WHERE provider_id = ? ORDER BY created_at ASC LIMIT 1;",
        )
        .bind(&input.provider_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(model_id)
    }

    pub async fn update_task_finished(
        &self,
        task_id: &str,
        status: &str,
        success_rate: f64,
        p95_latency_ms: i64,
        goodput_qps: f64,
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            "UPDATE benchmark_tasks
             SET status = ?, success_rate = ?, p95_latency_ms = ?, goodput_qps = ?, completed_at = ?
             WHERE id = ?;",
        )
        .bind(status)
        .bind(success_rate)
        .bind(p95_latency_ms)
        .bind(goodput_qps)
        .bind(now())
        .bind(task_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::not_found("task").into());
        }
        Ok(())
    }

    async fn ensure_benchmark_refs(&self, input: &BenchmarkStartInput) -> anyhow::Result<()> {
        let provider_exists: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM providers WHERE id = ? LIMIT 1;")
                .bind(&input.provider_id)
                .fetch_optional(&self.pool)
                .await?;
        if provider_exists.is_none() {
            return Err(AppError::not_found("provider").into());
        }

        let dataset_exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM datasets WHERE id = ? AND deleted_at IS NULL LIMIT 1;",
        )
        .bind(&input.dataset_id)
        .fetch_optional(&self.pool)
        .await?;
        if dataset_exists.is_none() {
            return Err(AppError::not_found("dataset").into());
        }

        Ok(())
    }

    pub async fn insert_stage(&self, sample: &StageSample) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO benchmark_stages
             (id, task_id, stage_index, concurrency, goodput_qps, p95_latency_ms, ttft_ms, tps,
              success_rate, error_rate, input_tokens, output_tokens, total_tokens, batch_size,
              text_count, documents_per_query, pair_count, image_count, sample_rounds,
              warmup_rounds, request_count, success_count, failure_count, sla_passed,
              stop_reason, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&sample.task_id)
        .bind(sample.stage_index)
        .bind(sample.concurrency)
        .bind(sample.goodput_qps)
        .bind(sample.p95_latency_ms)
        .bind(sample.ttft_ms)
        .bind(sample.tps)
        .bind(sample.success_rate)
        .bind(sample.error_rate)
        .bind(sample.input_tokens)
        .bind(sample.output_tokens)
        .bind(sample.total_tokens)
        .bind(sample.batch_size)
        .bind(sample.text_count)
        .bind(sample.documents_per_query)
        .bind(sample.pair_count)
        .bind(sample.image_count)
        .bind(sample.sample_rounds)
        .bind(sample.warmup_rounds)
        .bind(sample.request_count)
        .bind(sample.success_count)
        .bind(sample.failure_count)
        .bind(if sample.sla_passed { 1 } else { 0 })
        .bind(&sample.stop_reason)
        .bind(now())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_tick(&self, tick: &MetricsTick) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO benchmark_ticks
             (id, task_id, elapsed_seconds, qps, latency_ms, ttft_ms, tps, success_rate, errors,
              in_flight, input_tokens, output_tokens, total_tokens, batch_size, text_count,
              documents_per_query, pair_count, image_count, request_count, success_count,
              failure_count, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&tick.task_id)
        .bind(tick.elapsed_seconds)
        .bind(tick.qps)
        .bind(tick.latency_ms)
        .bind(tick.ttft_ms)
        .bind(tick.tps)
        .bind(tick.success_rate)
        .bind(tick.errors)
        .bind(tick.in_flight)
        .bind(tick.input_tokens)
        .bind(tick.output_tokens)
        .bind(tick.total_tokens)
        .bind(tick.batch_size)
        .bind(tick.text_count)
        .bind(tick.documents_per_query)
        .bind(tick.pair_count)
        .bind(tick.image_count)
        .bind(tick.request_count)
        .bind(tick.success_count)
        .bind(tick.failure_count)
        .bind(now())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_benchmark_error(&self, error: &BenchmarkErrorRecord) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO benchmark_errors (id, task_id, error_kind, message, count, created_at)
             VALUES (?, ?, ?, ?, ?, ?);",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(&error.task_id)
        .bind(&error.error_kind)
        .bind(&error.message)
        .bind(error.count)
        .bind(now())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_request_log(&self, log: &BenchmarkRequestLogRecord) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO benchmark_request_logs
             (id, task_id, stage_index, request_index, sample_index, status, latency_ms, ttft_ms,
              input_tokens, output_tokens, total_tokens, error_kind, error_message,
              prompt_preview, response_preview, body_ref, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
        )
        .bind(&log.summary.id)
        .bind(&log.summary.task_id)
        .bind(log.summary.stage_index)
        .bind(log.summary.request_index)
        .bind(log.summary.sample_index)
        .bind(&log.summary.status)
        .bind(log.summary.latency_ms)
        .bind(log.summary.ttft_ms)
        .bind(log.summary.input_tokens)
        .bind(log.summary.output_tokens)
        .bind(log.summary.total_tokens)
        .bind(&log.summary.error_kind)
        .bind(&log.raw_error)
        .bind(&log.summary.prompt_preview)
        .bind(&log.summary.response_preview)
        .bind(&log.body_ref)
        .bind(&log.summary.created_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_request_logs_page(
        &self,
        input: BenchmarkRequestLogPageInput,
    ) -> anyhow::Result<BenchmarkRequestLogPage> {
        let input = input.normalized();
        let offset = (input.page - 1) * input.page_size;
        let keyword_pattern = input.keyword.as_ref().map(|value| format!("%{value}%"));

        let mut count_builder =
            QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM benchmark_request_logs");
        push_request_log_filters(&mut count_builder, &input, keyword_pattern.as_deref());
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await?;

        let mut rows_builder = QueryBuilder::<Sqlite>::new(
            "SELECT id, task_id, stage_index, request_index, sample_index, status, latency_ms,
                    ttft_ms, input_tokens, output_tokens, total_tokens, error_kind,
                    prompt_preview, response_preview, created_at
             FROM benchmark_request_logs",
        );
        push_request_log_filters(&mut rows_builder, &input, keyword_pattern.as_deref());
        rows_builder
            .push(" ORDER BY stage_index ASC, request_index ASC LIMIT ")
            .push_bind(input.page_size)
            .push(" OFFSET ")
            .push_bind(offset);
        let rows = rows_builder.build().fetch_all(&self.pool).await?;

        Ok(BenchmarkRequestLogPage {
            items: rows
                .into_iter()
                .map(|row| request_log_summary_from_row(&row))
                .collect(),
            total,
            page: input.page,
            page_size: input.page_size,
        })
    }

    pub async fn get_request_log_detail(
        &self,
        request_id: &str,
    ) -> anyhow::Result<BenchmarkRequestLogDetail> {
        let row = sqlx::query(
            "SELECT id, task_id, stage_index, request_index, sample_index, status, latency_ms,
                    ttft_ms, input_tokens, output_tokens, total_tokens, error_kind,
                    error_message, prompt_preview, response_preview, body_ref, created_at
             FROM benchmark_request_logs
             WHERE id = ?;",
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await?;
        let row = row.ok_or_else(|| AppError::not_found("request_log"))?;
        let summary = request_log_summary_from_row(&row);
        let body_ref: Option<String> = row.get("body_ref");
        Ok(BenchmarkRequestLogDetail {
            summary,
            prompt: None,
            response_text: None,
            raw_error: row.get("error_message"),
            raw_usage: None,
            body_available: body_ref.is_some(),
        })
    }

    pub async fn delete_request_logs(&self, task_id: &str) -> anyhow::Result<DeleteResult> {
        let result = sqlx::query("DELETE FROM benchmark_request_logs WHERE task_id = ?;")
            .bind(task_id)
            .execute(&self.pool)
            .await?;
        Ok(DeleteResult {
            id: task_id.to_string(),
            deleted: result.rows_affected() > 0,
        })
    }

    pub async fn update_task_engine_mode(
        &self,
        task_id: &str,
        engine_mode: &str,
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            "UPDATE benchmark_tasks
             SET engine_mode = ?
             WHERE id = ?;",
        )
        .bind(engine_mode)
        .bind(task_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::not_found("task").into());
        }
        Ok(())
    }

    pub async fn update_task_preflight(
        &self,
        task_id: &str,
        preflight_result: Option<serde_json::Value>,
        diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
    ) -> anyhow::Result<()> {
        let preflight_json = preflight_result
            .map(|value| serde_json::to_string(&value))
            .transpose()?;
        let diagnostics_json = diagnostics_snapshot
            .map(|value| serde_json::to_string(&value))
            .transpose()?;
        let result = sqlx::query(
            "UPDATE benchmark_tasks
             SET preflight_result = ?, diagnostics_snapshot = ?
             WHERE id = ?;",
        )
        .bind(preflight_json)
        .bind(diagnostics_json)
        .bind(task_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::not_found("task").into());
        }
        Ok(())
    }

    pub async fn get_task_summary(&self, task_id: &str) -> anyhow::Result<BenchmarkTaskSummary> {
        let row = sqlx::query(
            "SELECT t.id, t.name, t.status, t.concurrency, t.success_rate, t.p95_latency_ms,
                    t.goodput_qps, t.created_at,
                    p.name AS provider_name,
                    COALESCE(m.name, '未选择模型') AS model_name,
                    COALESCE(m.model_type, 'text_generation') AS model_type,
                    d.name AS dataset_name
             FROM benchmark_tasks t
             JOIN providers p ON p.id = t.provider_id
             JOIN datasets d ON d.id = t.dataset_id
             LEFT JOIN models m ON m.id = t.model_id
             WHERE t.id = ?;",
        )
        .bind(task_id)
        .fetch_optional(&self.pool)
        .await?;

        row.map(task_from_row)
            .ok_or_else(|| AppError::not_found("task").into())
    }

    pub async fn list_recent_tasks(&self, limit: i64) -> anyhow::Result<Vec<BenchmarkTaskSummary>> {
        let rows = sqlx::query(
            "SELECT t.id, t.name, t.status, t.concurrency, t.success_rate, t.p95_latency_ms,
                    t.goodput_qps, t.created_at,
                    p.name AS provider_name,
                    COALESCE(m.name, '未选择模型') AS model_name,
                    COALESCE(m.model_type, 'text_generation') AS model_type,
                    d.name AS dataset_name
             FROM benchmark_tasks t
             JOIN providers p ON p.id = t.provider_id
             JOIN datasets d ON d.id = t.dataset_id
             LEFT JOIN models m ON m.id = t.model_id
             ORDER BY t.created_at DESC
             LIMIT ?;",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(task_from_row).collect())
    }

    pub async fn list_ticks(&self, task_id: &str) -> anyhow::Result<Vec<MetricsTick>> {
        let rows = sqlx::query(
            "SELECT task_id, elapsed_seconds, qps, latency_ms, ttft_ms, tps, success_rate,
                    errors, in_flight, input_tokens, output_tokens, total_tokens, batch_size,
                    text_count, documents_per_query, pair_count, image_count,
                    request_count, success_count, failure_count
             FROM benchmark_ticks
             WHERE task_id = ?
             ORDER BY elapsed_seconds ASC;",
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(tick_from_row).collect())
    }
}

fn push_request_log_filters<'a>(
    builder: &mut QueryBuilder<'a, Sqlite>,
    input: &'a BenchmarkRequestLogPageInput,
    keyword_pattern: Option<&'a str>,
) {
    builder.push(" WHERE task_id = ");
    builder.push_bind(&input.task_id);

    if let Some(stage_index) = input.stage_index {
        builder.push(" AND stage_index = ");
        builder.push_bind(stage_index);
    }

    if let Some(status) = input.status.as_deref() {
        builder.push(" AND status = ");
        builder.push_bind(status);
    }

    if let Some(pattern) = keyword_pattern {
        builder.push(" AND (prompt_preview LIKE ");
        builder.push_bind(pattern);
        builder.push(" OR response_preview LIKE ");
        builder.push_bind(pattern);
        builder.push(" OR error_kind LIKE ");
        builder.push_bind(pattern);
        builder.push(" OR error_message LIKE ");
        builder.push_bind(pattern);
        builder.push(")");
    }
}

fn request_log_summary_from_row(row: &SqliteRow) -> BenchmarkRequestLogSummary {
    BenchmarkRequestLogSummary {
        id: row.get("id"),
        task_id: row.get("task_id"),
        stage_index: row.get("stage_index"),
        request_index: row.get("request_index"),
        sample_index: row.get("sample_index"),
        status: row.get("status"),
        latency_ms: row.get("latency_ms"),
        ttft_ms: row.get("ttft_ms"),
        input_tokens: row.get("input_tokens"),
        output_tokens: row.get("output_tokens"),
        total_tokens: row.get("total_tokens"),
        error_kind: row.get("error_kind"),
        prompt_preview: row.get("prompt_preview"),
        response_preview: row.get("response_preview"),
        created_at: row.get("created_at"),
    }
}
