use crate::benchmark::events;
use crate::models::{BenchmarkTaskSummary, MetricsTick, StageChangedEvent};
use tauri::AppHandle;

#[derive(Clone)]
pub struct BenchmarkEventPublisher {
    app: AppHandle,
}

impl BenchmarkEventPublisher {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn task_started(&self, task: &BenchmarkTaskSummary) {
        events::emit_task_started(&self.app, task);
    }

    pub fn metrics_tick(&self, tick: MetricsTick) {
        events::emit_metrics_tick(&self.app, tick);
    }

    pub fn stage_changed(&self, event: StageChangedEvent) {
        events::emit_stage_changed(&self.app, event);
    }

    pub fn task_completed(&self, task: BenchmarkTaskSummary) {
        events::emit_task_completed(&self.app, task);
    }

    pub fn task_stopped(&self, task_id: &str) {
        events::emit_task_stopped(&self.app, task_id);
    }
}
