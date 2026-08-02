use crate::models::{
    EndpointProbeBatchSummary, EndpointProbeResponseDeltaEvent, EndpointProbeRunFinishedEvent,
    EndpointProbeRunStartedEvent,
};
use tauri::{AppHandle, Emitter};

pub const BATCH_STARTED: &str = "endpoint_probe:batch_started";
pub const RUN_STARTED: &str = "endpoint_probe:run_started";
pub const RESPONSE_DELTA: &str = "endpoint_probe:response_delta";
pub const RUN_FINISHED: &str = "endpoint_probe:run_finished";
pub const BATCH_FINISHED: &str = "endpoint_probe:batch_finished";

#[derive(Clone)]
pub struct EndpointProbeEventPublisher {
    app: AppHandle,
}

impl EndpointProbeEventPublisher {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    pub fn batch_started(&self, batch: &EndpointProbeBatchSummary) {
        let _ = self.app.emit(BATCH_STARTED, batch.clone());
    }

    pub fn run_started(&self, batch_id: &str, run_id: &str) {
        let _ = self.app.emit(
            RUN_STARTED,
            EndpointProbeRunStartedEvent {
                batch_id: batch_id.to_string(),
                run_id: run_id.to_string(),
            },
        );
    }

    pub fn response_delta(&self, event: EndpointProbeResponseDeltaEvent) {
        let _ = self.app.emit(RESPONSE_DELTA, event);
    }

    pub fn run_finished(&self, event: EndpointProbeRunFinishedEvent) {
        let _ = self.app.emit(RUN_FINISHED, event);
    }

    pub fn batch_finished(&self, batch: &EndpointProbeBatchSummary) {
        let _ = self.app.emit(BATCH_FINISHED, batch.clone());
    }
}
