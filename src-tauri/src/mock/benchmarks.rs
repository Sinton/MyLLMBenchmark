use super::types::MockTaskRecord;
use super::{now, resolve_model, MockDataStore};
use crate::benchmark::plan::BenchmarkPlan;
use crate::domain::benchmark::validate_benchmark_start;
use crate::domain::benchmark_sample::StageSample;
use crate::domain::model_type::normalize_model_type;
use crate::domain::workload::merge_workload_config;
use crate::error::AppError;
use crate::models::{
    BenchmarkErrorRecord, BenchmarkStartInput, BenchmarkTaskSummary, MetricsTick,
    ProviderDiagnosticsResult, ReportStageSummary,
};
use crate::report::analyzer;
use uuid::Uuid;

impl MockDataStore {
    pub async fn create_task(
        &self,
        input: &BenchmarkStartInput,
    ) -> anyhow::Result<BenchmarkTaskSummary> {
        validate_benchmark_start(input)?;

        let mut data = self.inner.write().await;
        let provider = data
            .providers
            .iter()
            .find(|provider| provider.id == input.provider_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("provider"))?;
        let dataset = data
            .datasets
            .iter()
            .find(|dataset| dataset.id == input.dataset_id)
            .cloned()
            .ok_or_else(|| AppError::not_found("dataset"))?;
        let model = resolve_model(&data.models, &input.provider_id, input.model_id.as_deref());
        let model_type = model
            .as_ref()
            .map(|model| normalize_model_type(&model.model_type))
            .unwrap_or_else(|| "text_generation".to_string());
        let workload_config = merge_workload_config(
            &model_type,
            input
                .workload_config
                .clone()
                .unwrap_or_else(|| serde_json::json!({})),
        );
        let plan = BenchmarkPlan::from_input(input);
        let id = Uuid::new_v4().to_string();
        let task = BenchmarkTaskSummary {
            id: id.clone(),
            name: format!("{} 压测任务", input.mode),
            status: "running".to_string(),
            model_type,
            model_name: model
                .as_ref()
                .map(|model| model.name.clone())
                .unwrap_or_else(|| "Unselected Model".to_string()),
            provider_name: provider.name,
            dataset_name: dataset.name,
            concurrency: input.concurrency,
            success_rate: 0.0,
            p95_latency_ms: 0,
            goodput_qps: 0.0,
            created_at: now(),
        };
        data.tasks.push(MockTaskRecord {
            summary: task.clone(),
            provider_id: input.provider_id.clone(),
            mode: input.mode.clone(),
            duration_seconds: input.duration_seconds,
            planned_stages: plan.stages.clone(),
            stage_sample_rounds: plan.stage_sample_rounds,
            warmup_rounds: plan.warmup_rounds,
            request_timeout_seconds: plan.request_timeout_seconds,
            sla_stop_policy: plan.sla_stop_policy.clone(),
            workload_config,
            engine_mode: "mock".to_string(),
            sla_p95_ms: input.sla_p95_ms.unwrap_or(5000),
            min_success_rate: input.min_success_rate.unwrap_or(99.0),
            preflight_result: None,
            diagnostics_snapshot: None,
        });
        data.stages.insert(id.clone(), Vec::new());
        data.ticks.insert(id, Vec::new());
        Ok(task)
    }

    pub async fn update_task_finished(
        &self,
        task_id: &str,
        status: &str,
        success_rate: f64,
        p95_latency_ms: i64,
        goodput_qps: f64,
    ) -> anyhow::Result<()> {
        let mut data = self.inner.write().await;
        let task = data
            .tasks
            .iter_mut()
            .find(|task| task.summary.id == task_id)
            .ok_or_else(|| AppError::not_found("task"))?;
        task.summary.status = status.to_string();
        task.summary.success_rate = success_rate;
        task.summary.p95_latency_ms = p95_latency_ms;
        task.summary.goodput_qps = goodput_qps;
        Ok(())
    }

    pub async fn insert_tick(&self, tick: &MetricsTick) -> anyhow::Result<()> {
        let mut data = self.inner.write().await;
        data.ticks
            .entry(tick.task_id.clone())
            .or_default()
            .push(tick.clone());
        Ok(())
    }

    pub async fn insert_benchmark_error(&self, error: &BenchmarkErrorRecord) -> anyhow::Result<()> {
        let mut data = self.inner.write().await;
        data.errors
            .entry(error.task_id.clone())
            .or_default()
            .push(error.clone());
        Ok(())
    }

    pub async fn update_task_engine_mode(
        &self,
        task_id: &str,
        engine_mode: &str,
    ) -> anyhow::Result<()> {
        let mut data = self.inner.write().await;
        let task = data
            .tasks
            .iter_mut()
            .find(|task| task.summary.id == task_id)
            .ok_or_else(|| AppError::not_found("task"))?;
        task.engine_mode = engine_mode.to_string();
        Ok(())
    }

    pub async fn update_task_preflight(
        &self,
        task_id: &str,
        preflight_result: Option<serde_json::Value>,
        diagnostics_snapshot: Option<ProviderDiagnosticsResult>,
    ) -> anyhow::Result<()> {
        let mut data = self.inner.write().await;
        let task = data
            .tasks
            .iter_mut()
            .find(|task| task.summary.id == task_id)
            .ok_or_else(|| AppError::not_found("task"))?;
        task.preflight_result = preflight_result;
        task.diagnostics_snapshot = diagnostics_snapshot;
        Ok(())
    }

    pub async fn insert_stage(&self, sample: &StageSample) -> anyhow::Result<()> {
        let mut data = self.inner.write().await;
        let status = analyzer::stage_status(sample.p95_latency_ms, sample.success_rate, 5000, 99.0);
        data.stages
            .entry(sample.task_id.clone())
            .or_default()
            .push(ReportStageSummary {
                stage_index: sample.stage_index,
                concurrency: sample.concurrency,
                sample_rounds: sample.sample_rounds,
                warmup_rounds: sample.warmup_rounds,
                request_count: sample.request_count,
                success_count: sample.success_count,
                failure_count: sample.failure_count,
                qps: sample.goodput_qps,
                p95_latency_ms: sample.p95_latency_ms,
                ttft_ms: sample.ttft_ms,
                tps: sample.tps,
                success_rate: sample.success_rate,
                error_rate: sample.error_rate,
                input_tokens: sample.input_tokens,
                output_tokens: sample.output_tokens,
                total_tokens: sample.total_tokens,
                batch_size: sample.batch_size,
                text_count: sample.text_count,
                documents_per_query: sample.documents_per_query,
                pair_count: sample.pair_count,
                image_count: sample.image_count,
                sla_passed: sample.sla_passed,
                stop_reason: sample.stop_reason.clone(),
                status,
            });
        Ok(())
    }

    pub async fn get_task_summary(&self, task_id: &str) -> anyhow::Result<BenchmarkTaskSummary> {
        let data = self.inner.read().await;
        data.tasks
            .iter()
            .find(|task| task.summary.id == task_id)
            .map(|task| task.summary.clone())
            .ok_or_else(|| AppError::not_found("task").into())
    }

    pub async fn list_ticks(&self, task_id: &str) -> anyhow::Result<Vec<MetricsTick>> {
        let data = self.inner.read().await;
        if !data.tasks.iter().any(|task| task.summary.id == task_id) {
            return Err(AppError::not_found("task").into());
        }
        Ok(data.ticks.get(task_id).cloned().unwrap_or_default())
    }
}
