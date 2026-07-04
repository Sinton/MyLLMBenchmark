use super::Database;
use crate::domain::dataset_import::{estimate_tokens, parse_dataset_import};
use crate::domain::dataset_tools::{
    dataset_export_result, render_dataset_export, validate_dataset_samples,
};
use crate::error::AppError;
use crate::models::{
    DatasetAppendInput, DatasetExportInput, DatasetExportResult, DatasetImportInput, DatasetSample,
    DatasetSampleBatchDeleteInput, DatasetSampleCreateInput, DatasetSamplePage,
    DatasetSamplePageInput, DatasetSamplePreview, DatasetSampleUpdateInput, DatasetSummary,
    DatasetUpdateInput, DatasetValidationResult, DeleteResult,
};
use sqlx::Row;
use uuid::Uuid;

impl Database {
    pub async fn list_datasets(&self) -> anyhow::Result<Vec<DatasetSummary>> {
        let rows = sqlx::query(
            "SELECT id, name, dataset_type, sample_count, average_tokens, updated_at
             FROM datasets
             WHERE deleted_at IS NULL
             ORDER BY updated_at DESC;",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(dataset_from_row).collect())
    }

    pub async fn import_dataset(
        &self,
        input: DatasetImportInput,
    ) -> anyhow::Result<DatasetSummary> {
        let parsed = parse_dataset_import(input)?;
        let id = Uuid::new_v4().to_string();
        let now = super::now();
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO datasets (id, name, dataset_type, sample_count, average_tokens, updated_at, deleted_at)
             VALUES (?, ?, ?, ?, ?, ?, NULL);",
        )
        .bind(&id)
        .bind(&parsed.name)
        .bind(&parsed.dataset_type)
        .bind(parsed.prompts.len() as i64)
        .bind(parsed.average_tokens)
        .bind(&now)
        .execute(&mut *tx)
        .await?;

        for (index, prompt) in parsed.prompts.iter().enumerate() {
            sqlx::query(
                "INSERT INTO dataset_samples
                 (id, dataset_id, sample_index, prompt, estimated_tokens, created_at)
                 VALUES (?, ?, ?, ?, ?, ?);",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(&id)
            .bind(index as i64)
            .bind(prompt)
            .bind(estimate_tokens(prompt))
            .bind(&now)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        self.get_dataset_summary(&id).await
    }

    pub async fn update_dataset(
        &self,
        input: DatasetUpdateInput,
    ) -> anyhow::Result<DatasetSummary> {
        let name = normalize_required(&input.name, "数据集名称不能为空")?;
        let dataset_type = normalize_required(&input.dataset_type, "数据集类型不能为空")?;
        let now = super::now();
        let result = sqlx::query(
            "UPDATE datasets
             SET name = ?, dataset_type = ?, updated_at = ?
             WHERE id = ? AND deleted_at IS NULL;",
        )
        .bind(name)
        .bind(dataset_type)
        .bind(now)
        .bind(&input.id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("dataset").into());
        }

        self.get_dataset_summary(&input.id).await
    }

    pub async fn delete_dataset(&self, dataset_id: &str) -> anyhow::Result<DeleteResult> {
        let now = super::now();
        let mut tx = self.pool.begin().await?;
        let result = sqlx::query(
            "UPDATE datasets
             SET deleted_at = ?, sample_count = 0, average_tokens = 0, updated_at = ?
             WHERE id = ? AND deleted_at IS NULL;",
        )
        .bind(&now)
        .bind(&now)
        .bind(dataset_id)
        .execute(&mut *tx)
        .await?;

        if result.rows_affected() > 0 {
            sqlx::query("DELETE FROM dataset_samples WHERE dataset_id = ?;")
                .bind(dataset_id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(DeleteResult {
            id: dataset_id.to_string(),
            deleted: result.rows_affected() > 0,
        })
    }

    pub async fn preview_dataset_samples(
        &self,
        dataset_id: &str,
        limit: i64,
    ) -> anyhow::Result<Vec<DatasetSamplePreview>> {
        self.ensure_active_dataset(dataset_id).await?;
        let rows = if limit <= 0 {
            sqlx::query(
                "SELECT id, sample_index, prompt, estimated_tokens
                 FROM dataset_samples
                 WHERE dataset_id = ?
                 ORDER BY sample_index ASC;",
            )
            .bind(dataset_id)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query(
                "SELECT id, sample_index, prompt, estimated_tokens
                 FROM dataset_samples
                 WHERE dataset_id = ?
                 ORDER BY sample_index ASC
                 LIMIT ?;",
            )
            .bind(dataset_id)
            .bind(limit.clamp(1, 10_000))
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rows.into_iter().map(sample_preview_from_row).collect())
    }

    pub async fn list_dataset_samples_page(
        &self,
        input: DatasetSamplePageInput,
    ) -> anyhow::Result<DatasetSamplePage> {
        self.ensure_active_dataset(&input.dataset_id).await?;
        let page = input.page.max(1);
        let page_size = normalize_page_size(input.page_size)?;
        let keyword = normalize_keyword(input.keyword);
        let offset = (page - 1) * page_size;

        let (total, rows) = if let Some(keyword) = keyword {
            let pattern = format!("%{keyword}%");
            let total = sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM dataset_samples
                 WHERE dataset_id = ? AND prompt LIKE ?;",
            )
            .bind(&input.dataset_id)
            .bind(&pattern)
            .fetch_one(&self.pool)
            .await?;
            let rows = sqlx::query(
                "SELECT id, sample_index, prompt, estimated_tokens
                 FROM dataset_samples
                 WHERE dataset_id = ? AND prompt LIKE ?
                 ORDER BY sample_index ASC
                 LIMIT ? OFFSET ?;",
            )
            .bind(&input.dataset_id)
            .bind(&pattern)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
            (total, rows)
        } else {
            let total = sqlx::query_scalar(
                "SELECT COUNT(*)
                 FROM dataset_samples
                 WHERE dataset_id = ?;",
            )
            .bind(&input.dataset_id)
            .fetch_one(&self.pool)
            .await?;
            let rows = sqlx::query(
                "SELECT id, sample_index, prompt, estimated_tokens
                 FROM dataset_samples
                 WHERE dataset_id = ?
                 ORDER BY sample_index ASC
                 LIMIT ? OFFSET ?;",
            )
            .bind(&input.dataset_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
            (total, rows)
        };

        Ok(DatasetSamplePage {
            items: rows.into_iter().map(sample_preview_from_row).collect(),
            total,
            page,
            page_size,
        })
    }

    pub async fn list_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<Vec<DatasetSample>> {
        self.ensure_active_dataset(dataset_id).await?;
        let rows = sqlx::query(
            "SELECT id, dataset_id, sample_index, prompt, estimated_tokens
             FROM dataset_samples
             WHERE dataset_id = ?
             ORDER BY sample_index ASC;",
        )
        .bind(dataset_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| DatasetSample {
                id: row.get("id"),
                dataset_id: row.get("dataset_id"),
                sample_index: row.get("sample_index"),
                prompt: row.get("prompt"),
                estimated_tokens: row.get("estimated_tokens"),
            })
            .collect())
    }

    pub async fn create_dataset_sample(
        &self,
        input: DatasetSampleCreateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        let prompt = normalize_prompt(&input.prompt)?;
        self.ensure_active_dataset(&input.dataset_id).await?;
        let sample_id = Uuid::new_v4().to_string();
        let sample_index: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sample_index) + 1, 0)
             FROM dataset_samples
             WHERE dataset_id = ?;",
        )
        .bind(&input.dataset_id)
        .fetch_one(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO dataset_samples
             (id, dataset_id, sample_index, prompt, estimated_tokens, created_at)
             VALUES (?, ?, ?, ?, ?, ?);",
        )
        .bind(&sample_id)
        .bind(&input.dataset_id)
        .bind(sample_index)
        .bind(&prompt)
        .bind(estimate_tokens(&prompt))
        .bind(super::now())
        .execute(&self.pool)
        .await?;

        self.recompute_dataset_stats(&input.dataset_id).await?;
        self.get_sample_preview(&sample_id).await
    }

    pub async fn update_dataset_sample(
        &self,
        input: DatasetSampleUpdateInput,
    ) -> anyhow::Result<DatasetSamplePreview> {
        let prompt = normalize_prompt(&input.prompt)?;
        let dataset_id = self.active_dataset_id_for_sample(&input.sample_id).await?;
        let result = sqlx::query(
            "UPDATE dataset_samples
             SET prompt = ?, estimated_tokens = ?
             WHERE id = ?;",
        )
        .bind(&prompt)
        .bind(estimate_tokens(&prompt))
        .bind(&input.sample_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::not_found("dataset sample").into());
        }

        self.recompute_dataset_stats(&dataset_id).await?;
        self.get_sample_preview(&input.sample_id).await
    }

    pub async fn delete_dataset_sample(&self, sample_id: &str) -> anyhow::Result<DeleteResult> {
        let dataset_id = self.active_dataset_id_for_sample(sample_id).await?;
        let result = sqlx::query("DELETE FROM dataset_samples WHERE id = ?;")
            .bind(sample_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() > 0 {
            self.renumber_dataset_samples(&dataset_id).await?;
            self.recompute_dataset_stats(&dataset_id).await?;
        }

        Ok(DeleteResult {
            id: sample_id.to_string(),
            deleted: result.rows_affected() > 0,
        })
    }

    pub async fn append_dataset_samples(
        &self,
        input: DatasetAppendInput,
    ) -> anyhow::Result<DatasetSummary> {
        let dataset = self.get_dataset_summary(&input.dataset_id).await?;
        let parsed = parse_dataset_import(DatasetImportInput {
            name: dataset.name.clone(),
            dataset_type: dataset.dataset_type.clone(),
            format: input.format,
            file_name: input.file_name,
            content_base64: input.content_base64,
        })?;
        let current_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM dataset_samples WHERE dataset_id = ?;")
                .bind(&input.dataset_id)
                .fetch_one(&self.pool)
                .await?;
        if current_count + parsed.prompts.len() as i64 > 10_000 {
            return Err(AppError::validation("数据集最多保留 10000 条样本").into());
        }

        let mut tx = self.pool.begin().await?;
        let mut next_index: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sample_index) + 1, 0)
             FROM dataset_samples
             WHERE dataset_id = ?;",
        )
        .bind(&input.dataset_id)
        .fetch_one(&mut *tx)
        .await?;
        let now = super::now();
        for prompt in parsed.prompts {
            sqlx::query(
                "INSERT INTO dataset_samples
                 (id, dataset_id, sample_index, prompt, estimated_tokens, created_at)
                 VALUES (?, ?, ?, ?, ?, ?);",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(&input.dataset_id)
            .bind(next_index)
            .bind(&prompt)
            .bind(estimate_tokens(&prompt))
            .bind(&now)
            .execute(&mut *tx)
            .await?;
            next_index += 1;
        }
        tx.commit().await?;
        self.recompute_dataset_stats(&input.dataset_id).await?;
        self.get_dataset_summary(&input.dataset_id).await
    }

    pub async fn delete_dataset_samples_batch(
        &self,
        input: DatasetSampleBatchDeleteInput,
    ) -> anyhow::Result<DeleteResult> {
        self.ensure_active_dataset(&input.dataset_id).await?;
        if input.sample_ids.is_empty() {
            return Ok(DeleteResult {
                id: input.dataset_id,
                deleted: false,
            });
        }
        let mut deleted = 0_u64;
        let mut tx = self.pool.begin().await?;
        for sample_id in input.sample_ids {
            let result = sqlx::query(
                "DELETE FROM dataset_samples
                 WHERE id = ? AND dataset_id = ?;",
            )
            .bind(sample_id)
            .bind(&input.dataset_id)
            .execute(&mut *tx)
            .await?;
            deleted += result.rows_affected();
        }
        tx.commit().await?;
        if deleted > 0 {
            self.renumber_dataset_samples(&input.dataset_id).await?;
            self.recompute_dataset_stats(&input.dataset_id).await?;
        }
        Ok(DeleteResult {
            id: input.dataset_id,
            deleted: deleted > 0,
        })
    }

    pub async fn export_dataset(
        &self,
        input: DatasetExportInput,
    ) -> anyhow::Result<DatasetExportResult> {
        let dataset = self.get_dataset_summary(&input.dataset_id).await?;
        let samples = self.list_dataset_samples(&input.dataset_id).await?;
        let payload = render_dataset_export(&samples, &input.format);
        Ok(dataset_export_result(
            &dataset,
            &payload,
            String::new(),
            String::new(),
        ))
    }

    pub async fn validate_dataset_samples(
        &self,
        dataset_id: &str,
    ) -> anyhow::Result<DatasetValidationResult> {
        let dataset = self.get_dataset_summary(dataset_id).await?;
        let samples = self.list_dataset_samples(dataset_id).await?;
        Ok(validate_dataset_samples(
            dataset_id,
            &dataset.dataset_type,
            &samples,
        ))
    }

    async fn get_dataset_summary(&self, dataset_id: &str) -> anyhow::Result<DatasetSummary> {
        let row = sqlx::query(
            "SELECT id, name, dataset_type, sample_count, average_tokens, updated_at
             FROM datasets
             WHERE id = ? AND deleted_at IS NULL;",
        )
        .bind(dataset_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::not_found("dataset"))?;
        Ok(dataset_from_row(row))
    }

    async fn ensure_active_dataset(&self, dataset_id: &str) -> anyhow::Result<()> {
        let exists: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM datasets WHERE id = ? AND deleted_at IS NULL LIMIT 1;",
        )
        .bind(dataset_id)
        .fetch_optional(&self.pool)
        .await?;
        if exists.is_none() {
            return Err(AppError::not_found("dataset").into());
        }
        Ok(())
    }

    async fn active_dataset_id_for_sample(&self, sample_id: &str) -> anyhow::Result<String> {
        sqlx::query_scalar(
            "SELECT s.dataset_id
             FROM dataset_samples s
             JOIN datasets d ON d.id = s.dataset_id
             WHERE s.id = ? AND d.deleted_at IS NULL;",
        )
        .bind(sample_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::not_found("dataset sample").into())
    }

    async fn get_sample_preview(&self, sample_id: &str) -> anyhow::Result<DatasetSamplePreview> {
        let row = sqlx::query(
            "SELECT s.id, s.sample_index, s.prompt, s.estimated_tokens
             FROM dataset_samples s
             JOIN datasets d ON d.id = s.dataset_id
             WHERE s.id = ? AND d.deleted_at IS NULL;",
        )
        .bind(sample_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::not_found("dataset sample"))?;
        Ok(sample_preview_from_row(row))
    }

    async fn recompute_dataset_stats(&self, dataset_id: &str) -> anyhow::Result<()> {
        let row = sqlx::query(
            "SELECT COUNT(*) AS sample_count,
                    COALESCE(CAST(ROUND(AVG(estimated_tokens)) AS INTEGER), 0) AS average_tokens
             FROM dataset_samples
             WHERE dataset_id = ?;",
        )
        .bind(dataset_id)
        .fetch_one(&self.pool)
        .await?;
        sqlx::query(
            "UPDATE datasets
             SET sample_count = ?, average_tokens = ?, updated_at = ?
             WHERE id = ? AND deleted_at IS NULL;",
        )
        .bind(row.get::<i64, _>("sample_count"))
        .bind(row.get::<i64, _>("average_tokens"))
        .bind(super::now())
        .bind(dataset_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn renumber_dataset_samples(&self, dataset_id: &str) -> anyhow::Result<()> {
        let rows = sqlx::query(
            "SELECT id
             FROM dataset_samples
             WHERE dataset_id = ?
             ORDER BY sample_index ASC;",
        )
        .bind(dataset_id)
        .fetch_all(&self.pool)
        .await?;

        for (index, row) in rows.into_iter().enumerate() {
            sqlx::query("UPDATE dataset_samples SET sample_index = ? WHERE id = ?;")
                .bind(index as i64)
                .bind(row.get::<String, _>("id"))
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }
}

fn dataset_from_row(row: sqlx::sqlite::SqliteRow) -> DatasetSummary {
    DatasetSummary {
        id: row.get("id"),
        name: row.get("name"),
        dataset_type: row.get("dataset_type"),
        sample_count: row.get("sample_count"),
        average_tokens: row.get("average_tokens"),
        updated_at: row.get("updated_at"),
    }
}

fn sample_preview_from_row(row: sqlx::sqlite::SqliteRow) -> DatasetSamplePreview {
    let prompt: String = row.get("prompt");
    DatasetSamplePreview {
        id: row.get("id"),
        sample_index: row.get("sample_index"),
        prompt_preview: preview_prompt(&prompt),
        estimated_tokens: row.get("estimated_tokens"),
        prompt,
    }
}

fn normalize_required(value: &str, message: &str) -> anyhow::Result<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(AppError::validation(message).into());
    }
    Ok(value.to_string())
}

fn normalize_prompt(prompt: &str) -> anyhow::Result<String> {
    normalize_required(prompt, "Prompt 样本不能为空")
}

fn normalize_keyword(keyword: Option<String>) -> Option<String> {
    keyword
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn normalize_page_size(page_size: i64) -> anyhow::Result<i64> {
    match page_size {
        0 => Ok(50),
        20 | 50 | 100 | 200 => Ok(page_size),
        value if value > 200 => Ok(200),
        _ => Err(AppError::validation("page_size 只支持 20、50、100、200").into()),
    }
}

fn preview_prompt(prompt: &str) -> String {
    const MAX_CHARS: usize = 120;
    let mut preview = prompt.chars().take(MAX_CHARS).collect::<String>();
    if prompt.chars().count() > MAX_CHARS {
        preview.push_str("...");
    }
    preview
}
