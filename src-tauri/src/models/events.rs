use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StageChangedEvent {
    pub task_id: String,
    pub stage: String,
    pub message: String,
    pub stage_index: Option<i64>,
    pub stage_total: Option<i64>,
    pub concurrency: Option<i64>,
}
