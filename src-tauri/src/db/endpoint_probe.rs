use super::Database;
use crate::error::AppError;
use crate::models::{
    DeleteResult, EndpointProbeBatchDetail, EndpointProbeBatchRecord, EndpointProbeBatchSummary,
    EndpointProbeHistoryPage, EndpointProbeHistoryPageInput, EndpointProbeRunDetail,
    EndpointProbeRunRecord, EndpointProbeRunSummary,
};
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, Sqlite};

const BATCH_SUMMARY_SELECT: &str =
    "SELECT b.id, b.name, b.status, b.streaming, b.temperature, b.max_output_tokens,
            b.timeout_seconds, b.save_body, b.concurrency, b.prompt_preview,
            b.created_at, b.finished_at,
            COUNT(r.id) AS total_runs,
            SUM(CASE WHEN r.status = 'pending' THEN 1 ELSE 0 END) AS pending_runs,
            SUM(CASE WHEN r.status = 'running' THEN 1 ELSE 0 END) AS running_runs,
            SUM(CASE WHEN r.status = 'passed' THEN 1 ELSE 0 END) AS passed_runs,
            SUM(CASE WHEN r.status = 'failed' THEN 1 ELSE 0 END) AS failed_runs,
            SUM(CASE WHEN r.status = 'cancelled' THEN 1 ELSE 0 END) AS cancelled_runs
     FROM endpoint_probe_batches b
     LEFT JOIN endpoint_probe_runs r ON r.batch_id = b.id";

const RUN_SELECT: &str =
    "SELECT id, batch_id, source_type, provider_id, name, base_url, interface_type,
            model, status, latency_ms, ttft_ms, input_tokens, output_tokens,
            total_tokens, error_kind, error_message, prompt_preview, response_preview,
            body_ref, created_at, finished_at
     FROM endpoint_probe_runs";

impl Database {
    pub async fn create_endpoint_probe_batch(
        &self,
        batch: &EndpointProbeBatchRecord,
        runs: &[EndpointProbeRunRecord],
    ) -> anyhow::Result<EndpointProbeBatchSummary> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "INSERT INTO endpoint_probe_batches
             (id, name, status, streaming, temperature, max_output_tokens, timeout_seconds, save_body,
              concurrency, prompt_preview, created_at, finished_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
        )
        .bind(&batch.summary.id)
        .bind(&batch.summary.name)
        .bind(&batch.summary.status)
        .bind(bool_to_i64(batch.summary.streaming))
        .bind(batch.summary.temperature)
        .bind(batch.summary.max_output_tokens)
        .bind(batch.summary.timeout_seconds)
        .bind(bool_to_i64(batch.summary.save_body))
        .bind(batch.summary.concurrency)
        .bind(&batch.summary.prompt_preview)
        .bind(&batch.summary.created_at)
        .bind(&batch.summary.finished_at)
        .execute(&mut *tx)
        .await?;

        for run in runs {
            sqlx::query(
                "INSERT INTO endpoint_probe_runs
                 (id, batch_id, source_type, provider_id, name, base_url, interface_type,
                  model, status, latency_ms, ttft_ms, input_tokens, output_tokens,
                  total_tokens, error_kind, error_message, prompt_preview, response_preview,
                  body_ref, created_at, finished_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
            )
            .bind(&run.summary.id)
            .bind(&run.summary.batch_id)
            .bind(&run.summary.source_type)
            .bind(&run.summary.provider_id)
            .bind(&run.summary.name)
            .bind(&run.summary.base_url)
            .bind(&run.summary.interface_type)
            .bind(&run.summary.model)
            .bind(&run.summary.status)
            .bind(run.summary.latency_ms)
            .bind(run.summary.ttft_ms)
            .bind(run.summary.input_tokens)
            .bind(run.summary.output_tokens)
            .bind(run.summary.total_tokens)
            .bind(&run.summary.error_kind)
            .bind(&run.summary.error_message)
            .bind(&run.summary.prompt_preview)
            .bind(&run.summary.response_preview)
            .bind(&run.body_ref)
            .bind(&run.summary.created_at)
            .bind(&run.summary.finished_at)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        self.get_endpoint_probe_batch_summary(&batch.summary.id)
            .await
    }

    pub async fn mark_endpoint_probe_run_started(&self, run_id: &str) -> anyhow::Result<()> {
        let result = sqlx::query(
            "UPDATE endpoint_probe_runs SET status = 'running' WHERE id = ? AND status = 'pending';",
        )
        .bind(run_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::not_found("endpoint_probe_run").into());
        }
        Ok(())
    }

    pub async fn finish_endpoint_probe_run(
        &self,
        record: &EndpointProbeRunRecord,
    ) -> anyhow::Result<EndpointProbeRunSummary> {
        let result = sqlx::query(
            "UPDATE endpoint_probe_runs
             SET status = ?, latency_ms = ?, ttft_ms = ?, input_tokens = ?,
                 output_tokens = ?, total_tokens = ?, error_kind = ?, error_message = ?,
                 prompt_preview = ?, response_preview = ?, body_ref = ?, finished_at = ?
             WHERE id = ?;",
        )
        .bind(&record.summary.status)
        .bind(record.summary.latency_ms)
        .bind(record.summary.ttft_ms)
        .bind(record.summary.input_tokens)
        .bind(record.summary.output_tokens)
        .bind(record.summary.total_tokens)
        .bind(&record.summary.error_kind)
        .bind(&record.summary.error_message)
        .bind(&record.summary.prompt_preview)
        .bind(&record.summary.response_preview)
        .bind(&record.body_ref)
        .bind(&record.summary.finished_at)
        .bind(&record.summary.id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::not_found("endpoint_probe_run").into());
        }
        self.get_endpoint_probe_run_summary(&record.summary.id)
            .await
    }

    pub async fn finish_endpoint_probe_batch(
        &self,
        batch_id: &str,
        status: &str,
        finished_at: &str,
    ) -> anyhow::Result<EndpointProbeBatchSummary> {
        let result = sqlx::query(
            "UPDATE endpoint_probe_batches SET status = ?, finished_at = ? WHERE id = ?;",
        )
        .bind(status)
        .bind(finished_at)
        .bind(batch_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(AppError::not_found("endpoint_probe_batch").into());
        }
        self.get_endpoint_probe_batch_summary(batch_id).await
    }

    pub async fn list_endpoint_probe_batches_page(
        &self,
        input: EndpointProbeHistoryPageInput,
    ) -> anyhow::Result<EndpointProbeHistoryPage> {
        let input = input.normalized();
        let offset = (input.page - 1) * input.page_size;
        let keyword_pattern = input.keyword.as_ref().map(|value| format!("%{value}%"));

        let mut count =
            QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM endpoint_probe_batches b");
        push_batch_filters(&mut count, &input, keyword_pattern.as_deref());
        let total: i64 = count.build_query_scalar().fetch_one(&self.pool).await?;

        let mut rows = QueryBuilder::<Sqlite>::new(BATCH_SUMMARY_SELECT);
        push_batch_filters(&mut rows, &input, keyword_pattern.as_deref());
        rows.push(" GROUP BY b.id ORDER BY b.created_at DESC LIMIT ")
            .push_bind(input.page_size)
            .push(" OFFSET ")
            .push_bind(offset);
        let items = rows
            .build()
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| endpoint_probe_batch_from_row(&row))
            .collect();
        Ok(EndpointProbeHistoryPage {
            items,
            total,
            page: input.page,
            page_size: input.page_size,
        })
    }

    pub async fn get_endpoint_probe_batch_summary(
        &self,
        batch_id: &str,
    ) -> anyhow::Result<EndpointProbeBatchSummary> {
        let query = format!("{BATCH_SUMMARY_SELECT} WHERE b.id = ? GROUP BY b.id;");
        let row = sqlx::query(&query)
            .bind(batch_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::not_found("endpoint_probe_batch"))?;
        Ok(endpoint_probe_batch_from_row(&row))
    }

    pub async fn get_endpoint_probe_batch_detail(
        &self,
        batch_id: &str,
    ) -> anyhow::Result<EndpointProbeBatchDetail> {
        let summary = self.get_endpoint_probe_batch_summary(batch_id).await?;
        let query = format!("{RUN_SELECT} WHERE batch_id = ? ORDER BY created_at ASC;");
        let runs = sqlx::query(&query)
            .bind(batch_id)
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|row| endpoint_probe_run_from_row(&row))
            .collect();
        Ok(EndpointProbeBatchDetail { summary, runs })
    }

    pub async fn get_endpoint_probe_run_summary(
        &self,
        run_id: &str,
    ) -> anyhow::Result<EndpointProbeRunSummary> {
        let query = format!("{RUN_SELECT} WHERE id = ?;");
        let row = sqlx::query(&query)
            .bind(run_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| AppError::not_found("endpoint_probe_run"))?;
        Ok(endpoint_probe_run_from_row(&row))
    }

    pub async fn get_endpoint_probe_run_detail(
        &self,
        run_id: &str,
    ) -> anyhow::Result<EndpointProbeRunDetail> {
        let summary = self.get_endpoint_probe_run_summary(run_id).await?;
        Ok(EndpointProbeRunDetail {
            raw_error: summary.error_message.clone(),
            summary,
            prompt: None,
            response_text: None,
            request_payload: None,
            raw_usage: None,
        })
    }

    pub async fn delete_endpoint_probe_batch(
        &self,
        batch_id: &str,
    ) -> anyhow::Result<DeleteResult> {
        let result = sqlx::query(
            "DELETE FROM endpoint_probe_batches WHERE id = ? AND status NOT IN ('pending', 'running');",
        )
        .bind(batch_id)
        .execute(&self.pool)
        .await?;
        Ok(DeleteResult {
            id: batch_id.to_string(),
            deleted: result.rows_affected() > 0,
        })
    }

    pub async fn recover_endpoint_probe_batches(&self, message: &str) -> anyhow::Result<()> {
        let finished_at = chrono::Utc::now().to_rfc3339();
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            "UPDATE endpoint_probe_runs
             SET status = 'failed', error_kind = 'orphaned', error_message = ?, finished_at = ?
             WHERE status IN ('pending', 'running');",
        )
        .bind(message)
        .bind(&finished_at)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE endpoint_probe_batches SET status = 'failed', finished_at = ?
             WHERE status IN ('pending', 'running');",
        )
        .bind(&finished_at)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

fn push_batch_filters<'a>(
    builder: &mut QueryBuilder<'a, Sqlite>,
    input: &'a EndpointProbeHistoryPageInput,
    keyword_pattern: Option<&'a str>,
) {
    let mut has_where = false;
    if let Some(status) = input.status.as_deref() {
        builder.push(" WHERE b.status = ").push_bind(status);
        has_where = true;
    }
    if let Some(pattern) = keyword_pattern {
        builder.push(if has_where { " AND (" } else { " WHERE (" });
        builder
            .push("b.name LIKE ")
            .push_bind(pattern)
            .push(" OR b.prompt_preview LIKE ")
            .push_bind(pattern)
            .push(" OR EXISTS (SELECT 1 FROM endpoint_probe_runs f WHERE f.batch_id = b.id AND (")
            .push("f.name LIKE ")
            .push_bind(pattern)
            .push(" OR f.base_url LIKE ")
            .push_bind(pattern)
            .push(" OR f.model LIKE ")
            .push_bind(pattern)
            .push(" OR f.error_message LIKE ")
            .push_bind(pattern)
            .push(" OR f.response_preview LIKE ")
            .push_bind(pattern)
            .push(")))");
    }
}

fn endpoint_probe_batch_from_row(row: &SqliteRow) -> EndpointProbeBatchSummary {
    EndpointProbeBatchSummary {
        id: row.get("id"),
        name: row.get("name"),
        status: row.get("status"),
        total_runs: row.get("total_runs"),
        pending_runs: row.get("pending_runs"),
        running_runs: row.get("running_runs"),
        passed_runs: row.get("passed_runs"),
        failed_runs: row.get("failed_runs"),
        cancelled_runs: row.get("cancelled_runs"),
        streaming: row.get::<i64, _>("streaming") != 0,
        temperature: row.get("temperature"),
        max_output_tokens: row.get("max_output_tokens"),
        timeout_seconds: row.get("timeout_seconds"),
        save_body: row.get::<i64, _>("save_body") != 0,
        concurrency: row.get("concurrency"),
        prompt_preview: row.get("prompt_preview"),
        created_at: row.get("created_at"),
        finished_at: row.get("finished_at"),
    }
}

fn endpoint_probe_run_from_row(row: &SqliteRow) -> EndpointProbeRunSummary {
    let body_ref: Option<String> = row.get("body_ref");
    EndpointProbeRunSummary {
        id: row.get("id"),
        batch_id: row.get("batch_id"),
        source_type: row.get("source_type"),
        provider_id: row.get("provider_id"),
        name: row.get("name"),
        base_url: row.get("base_url"),
        interface_type: row.get("interface_type"),
        model: row.get("model"),
        status: row.get("status"),
        latency_ms: row.get("latency_ms"),
        ttft_ms: row.get("ttft_ms"),
        input_tokens: row.get("input_tokens"),
        output_tokens: row.get("output_tokens"),
        total_tokens: row.get("total_tokens"),
        error_kind: row.get("error_kind"),
        error_message: row.get("error_message"),
        prompt_preview: row.get("prompt_preview"),
        response_preview: row.get("response_preview"),
        body_available: body_ref.is_some(),
        created_at: row.get("created_at"),
        finished_at: row.get("finished_at"),
    }
}

fn bool_to_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}
