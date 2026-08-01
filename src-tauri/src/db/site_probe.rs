use super::Database;
use crate::error::AppError;
use crate::models::{
    DeleteResult, SiteProbeHistoryPage, SiteProbeHistoryPageInput, SiteProbeRunDetail,
    SiteProbeRunRecord, SiteProbeRunSummary,
};
use sqlx::{sqlite::SqliteRow, QueryBuilder, Row, Sqlite};

impl Database {
    pub async fn insert_site_probe_run(
        &self,
        record: &SiteProbeRunRecord,
    ) -> anyhow::Result<SiteProbeRunSummary> {
        sqlx::query(
            "INSERT INTO site_probe_runs
             (id, name, base_url, interface_type, model, status, latency_ms, ttft_ms,
              input_tokens, output_tokens, total_tokens, error_kind, error_message,
              prompt_preview, response_preview, body_ref, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
        )
        .bind(&record.summary.id)
        .bind(&record.summary.name)
        .bind(&record.summary.base_url)
        .bind(&record.summary.interface_type)
        .bind(&record.summary.model)
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
        .bind(&record.summary.created_at)
        .execute(&self.pool)
        .await?;

        self.get_site_probe_run_summary(&record.summary.id).await
    }

    pub async fn list_site_probe_runs_page(
        &self,
        input: SiteProbeHistoryPageInput,
    ) -> anyhow::Result<SiteProbeHistoryPage> {
        let input = input.normalized();
        let offset = (input.page - 1) * input.page_size;
        let keyword_pattern = input.keyword.as_ref().map(|value| format!("%{value}%"));

        let mut count_builder =
            QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM site_probe_runs");
        push_site_probe_filters(&mut count_builder, &input, keyword_pattern.as_deref());
        let total: i64 = count_builder
            .build_query_scalar()
            .fetch_one(&self.pool)
            .await?;

        let mut rows_builder = QueryBuilder::<Sqlite>::new(
            "SELECT id, name, base_url, interface_type, model, status, latency_ms, ttft_ms,
                    input_tokens, output_tokens, total_tokens, error_kind, error_message,
                    prompt_preview, response_preview, body_ref, created_at
             FROM site_probe_runs",
        );
        push_site_probe_filters(&mut rows_builder, &input, keyword_pattern.as_deref());
        rows_builder
            .push(" ORDER BY created_at DESC LIMIT ")
            .push_bind(input.page_size)
            .push(" OFFSET ")
            .push_bind(offset);

        let rows = rows_builder.build().fetch_all(&self.pool).await?;
        Ok(SiteProbeHistoryPage {
            items: rows
                .into_iter()
                .map(|row| site_probe_summary_from_row(&row))
                .collect(),
            total,
            page: input.page,
            page_size: input.page_size,
        })
    }

    pub async fn get_site_probe_run_detail(
        &self,
        run_id: &str,
    ) -> anyhow::Result<SiteProbeRunDetail> {
        let row = self.get_site_probe_run_row(run_id).await?;
        Ok(SiteProbeRunDetail {
            summary: site_probe_summary_from_row(&row),
            prompt: None,
            response_text: None,
            request_payload: None,
            raw_error: row.get("error_message"),
            raw_usage: None,
        })
    }

    pub async fn get_site_probe_run_summary(
        &self,
        run_id: &str,
    ) -> anyhow::Result<SiteProbeRunSummary> {
        let row = self.get_site_probe_run_row(run_id).await?;
        Ok(site_probe_summary_from_row(&row))
    }

    pub async fn delete_site_probe_run(&self, run_id: &str) -> anyhow::Result<DeleteResult> {
        let result = sqlx::query("DELETE FROM site_probe_runs WHERE id = ?;")
            .bind(run_id)
            .execute(&self.pool)
            .await?;
        Ok(DeleteResult {
            id: run_id.to_string(),
            deleted: result.rows_affected() > 0,
        })
    }

    async fn get_site_probe_run_row(&self, run_id: &str) -> anyhow::Result<SqliteRow> {
        sqlx::query(
            "SELECT id, name, base_url, interface_type, model, status, latency_ms, ttft_ms,
                    input_tokens, output_tokens, total_tokens, error_kind, error_message,
                    prompt_preview, response_preview, body_ref, created_at
             FROM site_probe_runs
             WHERE id = ?;",
        )
        .bind(run_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::not_found("site_probe_run").into())
    }
}

fn push_site_probe_filters<'a>(
    builder: &mut QueryBuilder<'a, Sqlite>,
    input: &'a SiteProbeHistoryPageInput,
    keyword_pattern: Option<&'a str>,
) {
    let mut has_where = false;
    if let Some(status) = input.status.as_deref() {
        builder.push(" WHERE status = ");
        builder.push_bind(status);
        has_where = true;
    }

    if let Some(pattern) = keyword_pattern {
        builder.push(if has_where { " AND (" } else { " WHERE (" });
        builder.push("name LIKE ");
        builder.push_bind(pattern);
        builder.push(" OR base_url LIKE ");
        builder.push_bind(pattern);
        builder.push(" OR model LIKE ");
        builder.push_bind(pattern);
        builder.push(" OR prompt_preview LIKE ");
        builder.push_bind(pattern);
        builder.push(" OR response_preview LIKE ");
        builder.push_bind(pattern);
        builder.push(" OR error_message LIKE ");
        builder.push_bind(pattern);
        builder.push(")");
    }
}

fn site_probe_summary_from_row(row: &SqliteRow) -> SiteProbeRunSummary {
    let body_ref: Option<String> = row.get("body_ref");
    SiteProbeRunSummary {
        id: row.get("id"),
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
    }
}
