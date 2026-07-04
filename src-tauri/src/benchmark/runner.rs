use crate::benchmark::persistence::BenchmarkPersistence;
use crate::benchmark::publisher::BenchmarkEventPublisher;
use crate::benchmark::runtime::MockBenchmarkRuntime;
use crate::models::{
    BenchmarkStartInput, BenchmarkTaskSummary, DatasetSample, ProviderConnectionConfig,
};
use crate::state::AppState;
use tauri::AppHandle;
use tokio::sync::watch;

pub fn spawn_mock_benchmark(
    app: AppHandle,
    state: AppState,
    task: BenchmarkTaskSummary,
    input: BenchmarkStartInput,
    stop_rx: watch::Receiver<bool>,
) {
    tauri::async_runtime::spawn(async move {
        MockBenchmarkRuntime::new(
            task,
            input,
            stop_rx,
            BenchmarkEventPublisher::new(app),
            BenchmarkPersistence::new(state),
        )
        .run()
        .await;
    });
}

pub fn spawn_openai_compatible_benchmark(
    app: AppHandle,
    state: AppState,
    task: BenchmarkTaskSummary,
    input: BenchmarkStartInput,
    provider: ProviderConnectionConfig,
    samples: Vec<DatasetSample>,
    stop_rx: watch::Receiver<bool>,
) {
    tauri::async_runtime::spawn(async move {
        let publisher = BenchmarkEventPublisher::new(app);
        let persistence = BenchmarkPersistence::new(state);
        let task_id = task.id.clone();
        match crate::benchmark::engines::openai::OpenAICompatibleBenchmarkRuntime::new(
            task,
            input,
            provider,
            samples,
            stop_rx,
            publisher.clone(),
            persistence.clone(),
        ) {
            Ok(runtime) => runtime.run().await,
            Err(error) => {
                let _ = persistence
                    .finish_task(&task_id, "failed", 0.0, 0, 0.0)
                    .await;
                publisher.stage_changed(crate::models::StageChangedEvent {
                    task_id: task_id.clone(),
                    stage: "failed".to_string(),
                    message: format!("真实压测初始化失败：{error}"),
                    stage_index: None,
                    stage_total: None,
                    concurrency: None,
                });
                persistence.remove_task(&task_id).await;
            }
        }
    });
}
