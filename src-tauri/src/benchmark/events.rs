use crate::models::{BenchmarkTaskSummary, MetricsTick, StageChangedEvent};
use tauri::{AppHandle, Emitter};

pub const TASK_STARTED: &str = "benchmark:task_started";
pub const METRICS_TICK: &str = "benchmark:metrics_tick";
pub const STAGE_CHANGED: &str = "benchmark:stage_changed";
pub const TASK_COMPLETED: &str = "benchmark:task_completed";
pub const TASK_STOPPED: &str = "benchmark:task_stopped";
pub const REPORT_READY: &str = "benchmark:report_ready";

pub fn emit_task_started(app: &AppHandle, task: &BenchmarkTaskSummary) {
    let _ = app.emit(TASK_STARTED, task.clone());
}

pub fn emit_metrics_tick(app: &AppHandle, tick: MetricsTick) {
    let _ = app.emit(METRICS_TICK, tick);
}

pub fn emit_stage_changed(app: &AppHandle, event: StageChangedEvent) {
    let _ = app.emit(STAGE_CHANGED, event);
}

pub fn emit_task_completed(app: &AppHandle, task: BenchmarkTaskSummary) {
    let _ = app.emit(TASK_COMPLETED, task);
}

pub fn emit_task_stopped(app: &AppHandle, task_id: &str) {
    let _ = app.emit(TASK_STOPPED, task_id.to_string());
}
