use super::events::EndpointProbeEventPublisher;
use crate::benchmark::engines::real::{
    RealProviderClient, RealProviderProtocol, RequestOutcome, StreamDeltaObserver,
};
use crate::domain::workload::WorkloadConfig;
use crate::models::{
    EndpointProbeBatchSummary, EndpointProbeResponseDeltaEvent, EndpointProbeRunDetail,
    EndpointProbeRunFinishedEvent, EndpointProbeRunRecord, EndpointProbeRunSummary,
    ProviderConnectionConfig,
};
use crate::state::AppState;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::{watch, Semaphore};
use tokio::task::JoinSet;

#[derive(Clone)]
pub(crate) struct EndpointProbeExecution {
    pub summary: EndpointProbeRunSummary,
    pub config: ProviderConnectionConfig,
    pub protocol: RealProviderProtocol,
    pub prompt: String,
    pub workload: WorkloadConfig,
    pub timeout_seconds: i64,
    pub save_body: bool,
    pub request_payload: serde_json::Value,
}

pub(crate) fn spawn_endpoint_probe_batch(
    app: AppHandle,
    state: AppState,
    batch: EndpointProbeBatchSummary,
    executions: Vec<EndpointProbeExecution>,
    stop_rx: watch::Receiver<bool>,
) {
    tauri::async_runtime::spawn(async move {
        run_batch(app, state, batch, executions, stop_rx).await;
    });
}

async fn run_batch(
    app: AppHandle,
    state: AppState,
    batch: EndpointProbeBatchSummary,
    executions: Vec<EndpointProbeExecution>,
    stop_rx: watch::Receiver<bool>,
) {
    let publisher = EndpointProbeEventPublisher::new(app);
    publisher.batch_started(&batch);
    let semaphore = Arc::new(Semaphore::new(batch.concurrency.max(1) as usize));
    let mut set = JoinSet::new();

    for execution in executions {
        let state = state.clone();
        let publisher = publisher.clone();
        let semaphore = semaphore.clone();
        let stop_rx = stop_rx.clone();
        set.spawn(async move { run_one(state, publisher, semaphore, stop_rx, execution).await });
    }

    let mut internal_failure = false;
    while let Some(result) = set.join_next().await {
        if !matches!(result, Ok(true)) {
            internal_failure = true;
        }
    }

    let status = if internal_failure {
        "failed"
    } else if *stop_rx.borrow() {
        "cancelled"
    } else {
        "completed"
    };
    let finished_at = Utc::now().to_rfc3339();
    if let Ok(finished) = state
        .finish_endpoint_probe_batch(&batch.id, status, &finished_at)
        .await
    {
        update_provider_statuses(&state, &finished.id, &finished_at).await;
        publisher.batch_finished(&finished);
    }
    state.remove_endpoint_probe_batch(&batch.id).await;
}

async fn run_one(
    state: AppState,
    publisher: EndpointProbeEventPublisher,
    semaphore: Arc<Semaphore>,
    mut stop_rx: watch::Receiver<bool>,
    execution: EndpointProbeExecution,
) -> bool {
    let permit = tokio::select! {
        _ = wait_for_stop(&mut stop_rx) => {
            return finish_cancelled(&state, &publisher, execution).await;
        }
        permit = semaphore.acquire_owned() => match permit {
            Ok(permit) => permit,
            Err(_) => {
                return finish_failed(&state, &publisher, execution, "scheduler", "测活调度器已关闭").await;
            }
        }
    };
    if *stop_rx.borrow() {
        drop(permit);
        return finish_cancelled(&state, &publisher, execution).await;
    }

    if let Err(error) = state
        .mark_endpoint_probe_run_started(&execution.summary.id)
        .await
    {
        drop(permit);
        let _ = finish_failed(&state, &publisher, execution, "storage", &error.to_string()).await;
        return false;
    }
    publisher.run_started(&execution.summary.batch_id, &execution.summary.id);

    let sequence = Arc::new(AtomicU64::new(0));
    let observer = stream_observer(&publisher, &execution, sequence);
    let client = match RealProviderClient::new() {
        Ok(client) => client.with_stream_observer(observer),
        Err(error) => {
            drop(permit);
            return finish_failed(&state, &publisher, execution, "client", &error.to_string())
                .await;
        }
    };

    let request = client.text_generation(
        &execution.config,
        execution.protocol,
        &execution.summary.model,
        &execution.prompt,
        &execution.workload,
        execution.timeout_seconds,
    );
    let outcome = tokio::select! {
        _ = wait_for_stop(&mut stop_rx) => None,
        outcome = request => Some(outcome),
    };
    drop(permit);

    match outcome {
        Some(outcome) => finish_outcome(&state, &publisher, execution, outcome).await,
        None => finish_cancelled(&state, &publisher, execution).await,
    }
}

fn stream_observer(
    publisher: &EndpointProbeEventPublisher,
    execution: &EndpointProbeExecution,
    sequence: Arc<AtomicU64>,
) -> StreamDeltaObserver {
    let publisher = publisher.clone();
    let batch_id = execution.summary.batch_id.clone();
    let run_id = execution.summary.id.clone();
    Arc::new(move |delta, elapsed_ms| {
        publisher.response_delta(EndpointProbeResponseDeltaEvent {
            batch_id: batch_id.clone(),
            run_id: run_id.clone(),
            sequence: sequence.fetch_add(1, Ordering::Relaxed),
            delta,
            elapsed_ms,
        });
    })
}

async fn finish_outcome(
    state: &AppState,
    publisher: &EndpointProbeEventPublisher,
    execution: EndpointProbeExecution,
    outcome: RequestOutcome,
) -> bool {
    let detail = detail_from_outcome(&execution, &outcome);
    let record = record_from_outcome(&execution, outcome);
    let mut event_detail = detail;
    match state.finish_endpoint_probe_run(record).await {
        Ok(summary) => {
            event_detail.summary.body_available = summary.body_available;
            publisher.run_finished(EndpointProbeRunFinishedEvent {
                batch_id: execution.summary.batch_id,
                run: event_detail,
            });
            true
        }
        Err(error) => {
            event_detail.summary.status = "failed".to_string();
            event_detail.summary.error_kind = Some("storage".to_string());
            event_detail.summary.error_message = Some(error.to_string());
            event_detail.raw_error = Some(error.to_string());
            publisher.run_finished(EndpointProbeRunFinishedEvent {
                batch_id: execution.summary.batch_id,
                run: event_detail,
            });
            false
        }
    }
}

async fn finish_cancelled(
    state: &AppState,
    publisher: &EndpointProbeEventPublisher,
    execution: EndpointProbeExecution,
) -> bool {
    finish_terminal(
        state,
        publisher,
        execution,
        "cancelled",
        Some("cancelled"),
        Some("用户停止了测活批次"),
    )
    .await
}

async fn finish_failed(
    state: &AppState,
    publisher: &EndpointProbeEventPublisher,
    execution: EndpointProbeExecution,
    kind: &'static str,
    message: &str,
) -> bool {
    finish_terminal(
        state,
        publisher,
        execution,
        "failed",
        Some(kind),
        Some(message),
    )
    .await
}

async fn finish_terminal(
    state: &AppState,
    publisher: &EndpointProbeEventPublisher,
    execution: EndpointProbeExecution,
    status: &str,
    error_kind: Option<&str>,
    error_message: Option<&str>,
) -> bool {
    let mut summary = execution.summary.clone();
    summary.status = status.to_string();
    summary.error_kind = error_kind.map(ToString::to_string);
    summary.error_message = error_message.map(ToString::to_string);
    summary.response_preview = error_message.map(preview_text);
    summary.finished_at = Some(Utc::now().to_rfc3339());
    let detail = EndpointProbeRunDetail {
        summary: summary.clone(),
        prompt: Some(execution.prompt.clone()),
        response_text: None,
        request_payload: Some(execution.request_payload.clone()),
        raw_error: error_message.map(ToString::to_string),
        raw_usage: None,
    };
    let record = EndpointProbeRunRecord {
        summary,
        body_ref: None,
        prompt: execution.save_body.then(|| execution.prompt.clone()),
        response_text: None,
        request_payload: execution
            .save_body
            .then(|| execution.request_payload.clone()),
        raw_error: execution
            .save_body
            .then(|| error_message.map(ToString::to_string))
            .flatten(),
        raw_usage: None,
    };
    let persisted = state.finish_endpoint_probe_run(record).await.is_ok();
    publisher.run_finished(EndpointProbeRunFinishedEvent {
        batch_id: execution.summary.batch_id,
        run: detail,
    });
    persisted
}

fn detail_from_outcome(
    execution: &EndpointProbeExecution,
    outcome: &RequestOutcome,
) -> EndpointProbeRunDetail {
    let mut summary = summary_from_outcome(execution, outcome);
    summary.body_available = execution.save_body;
    EndpointProbeRunDetail {
        summary,
        prompt: Some(execution.prompt.clone()),
        response_text: outcome.response_text.clone(),
        request_payload: Some(execution.request_payload.clone()),
        raw_error: outcome.error_message.clone(),
        raw_usage: outcome.raw_usage.clone(),
    }
}

fn record_from_outcome(
    execution: &EndpointProbeExecution,
    outcome: RequestOutcome,
) -> EndpointProbeRunRecord {
    EndpointProbeRunRecord {
        summary: summary_from_outcome(execution, &outcome),
        body_ref: None,
        prompt: execution.save_body.then(|| execution.prompt.clone()),
        response_text: execution
            .save_body
            .then(|| outcome.response_text.clone())
            .flatten(),
        request_payload: execution
            .save_body
            .then(|| execution.request_payload.clone()),
        raw_error: execution
            .save_body
            .then(|| outcome.error_message.clone())
            .flatten(),
        raw_usage: execution
            .save_body
            .then(|| outcome.raw_usage.clone())
            .flatten(),
    }
}

fn summary_from_outcome(
    execution: &EndpointProbeExecution,
    outcome: &RequestOutcome,
) -> EndpointProbeRunSummary {
    let mut summary = execution.summary.clone();
    summary.status = if outcome.ok { "passed" } else { "failed" }.to_string();
    summary.latency_ms = outcome.latency_ms;
    summary.ttft_ms = outcome.ttft_ms;
    summary.input_tokens = outcome.usage.input_tokens;
    summary.output_tokens = outcome.usage.output_tokens;
    summary.total_tokens = outcome.usage.total_tokens;
    summary.error_kind = outcome.error_kind.map(ToString::to_string);
    summary.error_message = outcome.error_message.clone();
    summary.response_preview = outcome
        .response_text
        .as_deref()
        .map(preview_text)
        .or_else(|| outcome.error_message.as_deref().map(preview_text));
    summary.finished_at = Some(Utc::now().to_rfc3339());
    summary
}

async fn update_provider_statuses(state: &AppState, batch_id: &str, checked_at: &str) {
    let Ok(detail) = state.get_endpoint_probe_batch_detail(batch_id).await else {
        return;
    };
    let mut statuses: HashMap<String, Vec<String>> = HashMap::new();
    for run in detail.runs {
        let Some(provider_id) = run.provider_id else {
            continue;
        };
        statuses.entry(provider_id).or_default().push(run.status);
    }
    for (provider_id, run_statuses) in statuses {
        let next_status = if run_statuses.iter().any(|status| status == "passed") {
            Some("online")
        } else if run_statuses.iter().all(|status| status == "failed") {
            Some("offline")
        } else {
            None
        };
        let Some(next_status) = next_status else {
            continue;
        };
        let _ = state
            .update_provider_connection_status(&provider_id, next_status, checked_at)
            .await;
    }
}

async fn wait_for_stop(stop_rx: &mut watch::Receiver<bool>) {
    if *stop_rx.borrow() {
        return;
    }
    while stop_rx.changed().await.is_ok() {
        if *stop_rx.borrow() {
            return;
        }
    }
}

fn preview_text(value: &str) -> String {
    const MAX_CHARS: usize = 120;
    let mut preview = value.chars().take(MAX_CHARS).collect::<String>();
    if value.chars().count() > MAX_CHARS {
        preview.push_str("...");
    }
    preview
}
