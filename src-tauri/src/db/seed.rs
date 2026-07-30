use super::{now, Database};
use crate::domain::dataset_import::estimate_tokens;
use crate::domain::demo_samples::{
    build_chat_prompts, build_embedding_prompts, build_rerank_prompts, build_vision_prompts,
};
use sqlx::Row;
use uuid::Uuid;

const SEEDED_CHAT_DATASET_NAME: &str = "文本生成标准问答样本";
const SEEDED_EMBEDDING_DATASET_NAME: &str = "向量嵌入知识库段落";
const SEEDED_RERANK_DATASET_NAME: &str = "重排序候选文档组";
const SEEDED_VISION_DATASET_NAME: &str = "视觉多模态图文识别样本";

struct SeededDatasetDefinition {
    name: &'static str,
    legacy_names: &'static [&'static str],
    dataset_type: &'static str,
    build_prompts: fn() -> Vec<String>,
}

impl Database {
    pub(super) async fn seed_defaults(&self) -> anyhow::Result<()> {
        let dataset_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM datasets")
            .fetch_one(&self.pool)
            .await?;

        if dataset_count == 0 {
            let now = now();
            for definition in seeded_dataset_definitions() {
                self.insert_seeded_dataset(&definition, &now).await?;
            }
        } else {
            for definition in seeded_dataset_definitions() {
                self.backfill_seeded_dataset_samples(&definition).await?;
            }
        }

        Ok(())
    }

    async fn insert_seeded_dataset(
        &self,
        definition: &SeededDatasetDefinition,
        created_at: &str,
    ) -> anyhow::Result<()> {
        let dataset_id = Uuid::new_v4().to_string();
        let prompts = (definition.build_prompts)();
        sqlx::query(
            "INSERT INTO datasets (id, name, dataset_type, sample_count, average_tokens, updated_at)
             VALUES (?, ?, ?, ?, ?, ?);",
        )
        .bind(&dataset_id)
        .bind(definition.name)
        .bind(definition.dataset_type)
        .bind(prompts.len() as i64)
        .bind(average_tokens(&prompts))
        .bind(created_at)
        .execute(&self.pool)
        .await?;
        self.insert_dataset_prompts(&dataset_id, &prompts, created_at)
            .await
    }

    async fn backfill_seeded_dataset_samples(
        &self,
        definition: &SeededDatasetDefinition,
    ) -> anyhow::Result<()> {
        for name in std::iter::once(definition.name).chain(definition.legacy_names.iter().copied())
        {
            let rows = sqlx::query(
                "SELECT id, sample_count
                 FROM datasets
                 WHERE name = ? AND dataset_type = ? AND deleted_at IS NULL;",
            )
            .bind(name)
            .bind(definition.dataset_type)
            .fetch_all(&self.pool)
            .await?;

            for row in rows {
                let dataset_id: String = row.get("id");
                let declared_sample_count: i64 = row.get("sample_count");
                let actual_sample_count: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM dataset_samples WHERE dataset_id = ?;",
                )
                .bind(&dataset_id)
                .fetch_one(&self.pool)
                .await?;
                if actual_sample_count > 0 || declared_sample_count <= 0 {
                    continue;
                }

                let now = now();
                let prompts = (definition.build_prompts)();
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
            }
        }

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

fn seeded_dataset_definitions() -> [SeededDatasetDefinition; 4] {
    [
        SeededDatasetDefinition {
            name: SEEDED_CHAT_DATASET_NAME,
            legacy_names: &["Chat 标准问答样本"],
            dataset_type: "Chat",
            build_prompts: build_chat_prompts,
        },
        SeededDatasetDefinition {
            name: SEEDED_EMBEDDING_DATASET_NAME,
            legacy_names: &["Embedding 知识库段落"],
            dataset_type: "Embedding",
            build_prompts: build_embedding_prompts,
        },
        SeededDatasetDefinition {
            name: SEEDED_RERANK_DATASET_NAME,
            legacy_names: &["Reranker 候选文档组"],
            dataset_type: "Reranker",
            build_prompts: build_rerank_prompts,
        },
        SeededDatasetDefinition {
            name: SEEDED_VISION_DATASET_NAME,
            legacy_names: &["Vision 图文识别样本"],
            dataset_type: "Vision",
            build_prompts: build_vision_prompts,
        },
    ]
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
