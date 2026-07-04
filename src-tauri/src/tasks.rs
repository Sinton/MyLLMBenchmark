use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

#[derive(Clone, Default)]
pub struct TaskManager {
    running: Arc<Mutex<HashMap<String, watch::Sender<bool>>>>,
}

impl TaskManager {
    pub async fn register(&self, task_id: String, tx: watch::Sender<bool>) {
        self.running.lock().await.insert(task_id, tx);
    }

    pub async fn stop(&self, task_id: &str) -> bool {
        let running = self.running.lock().await;
        if let Some(tx) = running.get(task_id) {
            let _ = tx.send(true);
            return true;
        }
        false
    }

    pub async fn remove(&self, task_id: &str) {
        self.running.lock().await.remove(task_id);
    }
}
