use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct RequestLogBodyStore {
    data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLogBodyLine {
    pub id: String,
    pub prompt: Option<String>,
    pub response_text: Option<String>,
    pub raw_error: Option<String>,
    pub raw_usage: Option<serde_json::Value>,
}

impl RequestLogBodyStore {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn body_ref(task_id: &str) -> String {
        format!("request_logs/{task_id}.jsonl")
    }

    pub fn path_for_task(&self, task_id: &str) -> PathBuf {
        self.data_dir
            .join("request_logs")
            .join(format!("{task_id}.jsonl"))
    }

    pub async fn append_body(
        &self,
        task_id: &str,
        line: &RequestLogBodyLine,
    ) -> anyhow::Result<()> {
        let dir = self.data_dir.join("request_logs");
        tokio::fs::create_dir_all(&dir).await?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path_for_task(task_id))
            .await?;
        let payload = serde_json::to_string(line)?;
        file.write_all(payload.as_bytes()).await?;
        file.write_all(b"\n").await?;
        Ok(())
    }

    pub async fn read_body(
        &self,
        task_id: &str,
        request_id: &str,
    ) -> anyhow::Result<Option<RequestLogBodyLine>> {
        let path = self.path_for_task(task_id);
        let content = match tokio::fs::read_to_string(path).await {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        for line in content.lines() {
            let Ok(body) = serde_json::from_str::<RequestLogBodyLine>(line) else {
                continue;
            };
            if body.id == request_id {
                return Ok(Some(body));
            }
        }
        Ok(None)
    }

    pub async fn delete_task_bodies(&self, task_id: &str) -> anyhow::Result<()> {
        let path = self.path_for_task(task_id);
        match tokio::fs::remove_file(path).await {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}
