use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

const BODY_DIRECTORY: &str = "endpoint_probe_bodies";
const LEGACY_BODY_DIRECTORY: &str = "site_probe_bodies";

#[derive(Debug, Clone)]
pub struct EndpointProbeBodyStore {
    data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointProbeBodyLine {
    pub id: String,
    pub prompt: Option<String>,
    pub response_text: Option<String>,
    pub request_payload: Option<serde_json::Value>,
    pub raw_error: Option<String>,
    pub raw_usage: Option<serde_json::Value>,
}

impl EndpointProbeBodyStore {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn body_ref(run_id: &str) -> String {
        format!("{BODY_DIRECTORY}/{run_id}.jsonl")
    }

    pub async fn migrate_legacy_directory(&self) -> anyhow::Result<()> {
        let legacy = self.data_dir.join(LEGACY_BODY_DIRECTORY);
        if !legacy.exists() {
            return Ok(());
        }
        let target = self.data_dir.join(BODY_DIRECTORY);
        if !target.exists() {
            tokio::fs::rename(legacy, target).await?;
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(&legacy).await?;
        while let Some(entry) = entries.next_entry().await? {
            let destination = target.join(entry.file_name());
            if !destination.exists() {
                tokio::fs::rename(entry.path(), destination).await?;
            }
        }
        let mut remaining = tokio::fs::read_dir(&legacy).await?;
        if remaining.next_entry().await?.is_none() {
            tokio::fs::remove_dir(legacy).await?;
        }
        Ok(())
    }

    pub fn path_for_run(&self, run_id: &str) -> PathBuf {
        self.data_dir
            .join(BODY_DIRECTORY)
            .join(format!("{run_id}.jsonl"))
    }

    pub async fn write_body(
        &self,
        run_id: &str,
        line: &EndpointProbeBodyLine,
    ) -> anyhow::Result<()> {
        let dir = self.data_dir.join(BODY_DIRECTORY);
        tokio::fs::create_dir_all(&dir).await?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.path_for_run(run_id))
            .await?;
        file.write_all(serde_json::to_string(line)?.as_bytes())
            .await?;
        file.write_all(b"\n").await?;
        Ok(())
    }

    pub async fn read_body(&self, run_id: &str) -> anyhow::Result<Option<EndpointProbeBodyLine>> {
        let content = match tokio::fs::read_to_string(self.path_for_run(run_id)).await {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        Ok(content.lines().find_map(|line| {
            serde_json::from_str::<EndpointProbeBodyLine>(line)
                .ok()
                .filter(|body| body.id == run_id)
        }))
    }

    pub async fn delete_body(&self, run_id: &str) -> anyhow::Result<()> {
        match tokio::fs::remove_file(self.path_for_run(run_id)).await {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EndpointProbeBodyLine, EndpointProbeBodyStore};
    use uuid::Uuid;

    #[tokio::test]
    async fn body_store_writes_reads_deletes_and_migrates_legacy_directory() {
        let data_dir = std::env::temp_dir().join(format!(
            "my-llm-benchmark-endpoint-probe-body-{}",
            Uuid::new_v4()
        ));
        let legacy_dir = data_dir.join("site_probe_bodies");
        tokio::fs::create_dir_all(&legacy_dir).await.unwrap();
        tokio::fs::write(
            legacy_dir.join("legacy.jsonl"),
            r#"{"id":"legacy","prompt":"hello"}
"#,
        )
        .await
        .unwrap();

        let store = EndpointProbeBodyStore::new(&data_dir);
        store.migrate_legacy_directory().await.unwrap();
        assert!(!legacy_dir.exists());
        assert!(store.path_for_run("legacy").exists());

        let line = EndpointProbeBodyLine {
            id: "run-1".to_string(),
            prompt: Some("prompt".to_string()),
            response_text: Some("response".to_string()),
            request_payload: Some(serde_json::json!({"model": "test"})),
            raw_error: None,
            raw_usage: Some(serde_json::json!({"total_tokens": 2})),
        };
        store.write_body("run-1", &line).await.unwrap();
        let restored = store.read_body("run-1").await.unwrap().unwrap();
        assert_eq!(restored.prompt.as_deref(), Some("prompt"));
        assert_eq!(restored.response_text.as_deref(), Some("response"));

        store.delete_body("run-1").await.unwrap();
        assert!(store.read_body("run-1").await.unwrap().is_none());
        let _ = tokio::fs::remove_dir_all(data_dir).await;
    }
}
