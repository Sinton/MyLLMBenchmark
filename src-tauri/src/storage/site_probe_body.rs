use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct SiteProbeBodyStore {
    data_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteProbeBodyLine {
    pub id: String,
    pub prompt: Option<String>,
    pub response_text: Option<String>,
    pub request_payload: Option<serde_json::Value>,
    pub raw_error: Option<String>,
    pub raw_usage: Option<serde_json::Value>,
}

impl SiteProbeBodyStore {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        Self {
            data_dir: data_dir.into(),
        }
    }

    pub fn body_ref(run_id: &str) -> String {
        format!("site_probe_bodies/{run_id}.jsonl")
    }

    pub fn path_for_run(&self, run_id: &str) -> PathBuf {
        self.data_dir
            .join("site_probe_bodies")
            .join(format!("{run_id}.jsonl"))
    }

    pub async fn write_body(&self, run_id: &str, line: &SiteProbeBodyLine) -> anyhow::Result<()> {
        let dir = self.data_dir.join("site_probe_bodies");
        tokio::fs::create_dir_all(&dir).await?;
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.path_for_run(run_id))
            .await?;
        let payload = serde_json::to_string(line)?;
        file.write_all(payload.as_bytes()).await?;
        file.write_all(b"\n").await?;
        Ok(())
    }

    pub async fn read_body(&self, run_id: &str) -> anyhow::Result<Option<SiteProbeBodyLine>> {
        let content = match tokio::fs::read_to_string(self.path_for_run(run_id)).await {
            Ok(content) => content,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        for line in content.lines() {
            let Ok(body) = serde_json::from_str::<SiteProbeBodyLine>(line) else {
                continue;
            };
            if body.id == run_id {
                return Ok(Some(body));
            }
        }
        Ok(None)
    }

    pub async fn delete_body(&self, run_id: &str) -> anyhow::Result<()> {
        match tokio::fs::remove_file(self.path_for_run(run_id)).await {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}
