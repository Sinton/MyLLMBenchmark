use super::{now, Database};
use crate::domain::dataset_import::estimate_tokens;
use crate::domain::demo_samples::build_chat_prompts;
use sqlx::Row;
use uuid::Uuid;

const SEEDED_CHAT_DATASET_NAME: &str = "文本生成标准问答样本";

impl Database {
    pub(super) async fn seed_defaults(&self) -> anyhow::Result<()> {
        let dataset_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM datasets")
            .fetch_one(&self.pool)
            .await?;

        if dataset_count == 0 {
            let now = now();
            let chat_dataset_id = Uuid::new_v4().to_string();
            let chat_prompts = build_chat_prompts();
            sqlx::query(
                "INSERT INTO datasets (id, name, dataset_type, sample_count, average_tokens, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?);",
            )
            .bind(&chat_dataset_id)
            .bind(SEEDED_CHAT_DATASET_NAME)
            .bind("Chat")
            .bind(chat_prompts.len() as i64)
            .bind(average_tokens(&chat_prompts))
            .bind(&now)
            .execute(&self.pool)
            .await?;
            self.insert_dataset_prompts(&chat_dataset_id, &chat_prompts, &now)
                .await?;

            for (name, dataset_type, sample_count, average_tokens) in [
                ("向量嵌入知识库段落", "Embedding", 2048, 180),
                ("重排序候选文档组", "Reranker", 512, 760),
                ("视觉多模态图文识别样本", "Vision", 96, 120),
            ] {
                sqlx::query(
                    "INSERT INTO datasets (id, name, dataset_type, sample_count, average_tokens, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?);",
                )
                .bind(Uuid::new_v4().to_string())
                .bind(name)
                .bind(dataset_type)
                .bind(sample_count)
                .bind(average_tokens)
                .bind(&now)
                .execute(&self.pool)
                .await?;
            }
        }

        self.backfill_seeded_chat_samples().await?;

        Ok(())
    }

    async fn backfill_seeded_chat_samples(&self) -> anyhow::Result<()> {
        let Some(row) = sqlx::query(
            "SELECT id
             FROM datasets
             WHERE name = ? AND dataset_type = 'Chat' AND deleted_at IS NULL
             LIMIT 1;",
        )
        .bind(SEEDED_CHAT_DATASET_NAME)
        .fetch_optional(&self.pool)
        .await?
        else {
            return Ok(());
        };

        let dataset_id: String = row.get("id");
        let sample_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM dataset_samples WHERE dataset_id = ?;")
                .bind(&dataset_id)
                .fetch_one(&self.pool)
                .await?;
        if sample_count > 0 {
            return Ok(());
        }

        let now = now();
        let prompts = build_chat_prompts();
        self.insert_dataset_prompts(&dataset_id, &prompts, &now)
            .await?;
        sqlx::query(
            "UPDATE datasets
             SET sample_count = ?, average_tokens = ?, updated_at = ?
             WHERE id = ?;",
        )
        .bind(prompts.len() as i64)
        .bind(average_tokens(&prompts))
        .bind(&now)
        .bind(&dataset_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn insert_dataset_prompts(
        &self,
        dataset_id: &str,
        prompts: &[String],
        created_at: &str,
    ) -> anyhow::Result<()> {
        for (index, prompt) in prompts.iter().enumerate() {
            sqlx::query(
                "INSERT INTO dataset_samples
                 (id, dataset_id, sample_index, prompt, estimated_tokens, created_at)
                 VALUES (?, ?, ?, ?, ?, ?);",
            )
            .bind(Uuid::new_v4().to_string())
            .bind(dataset_id)
            .bind(index as i64)
            .bind(prompt)
            .bind(estimate_tokens(prompt))
            .bind(created_at)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}

fn average_tokens(prompts: &[String]) -> i64 {
    if prompts.is_empty() {
        return 0;
    }
    prompts
        .iter()
        .map(|prompt| estimate_tokens(prompt))
        .sum::<i64>()
        / prompts.len() as i64
}
