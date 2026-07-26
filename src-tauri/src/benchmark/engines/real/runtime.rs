use super::client::RealProviderClient;
use super::helpers::{collect_embedding_inputs, collect_rerank_inputs};
use super::outcome::RequestOutcome;
use super::protocol::RealProviderProtocol;
use super::{metrics, request_logs};
use crate::{
    benchmark::{
        persistence::BenchmarkPersistence, plan::BenchmarkPlan, publisher::BenchmarkEventPublisher,
    },
    domain::{benchmark_sample::StageSample, model_type::ModelType, workload::WorkloadConfig},
    models::{
        BenchmarkErrorRecord, BenchmarkStartInput, BenchmarkTaskSummary, DatasetSample,
        MetricsTick, ProviderConnectionConfig, RequestLogConfig, StageChangedEvent,
    },
};
use futures_util::future::join_all;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::Instant;
pub struct RealBenchmarkRuntime {
    task: BenchmarkTaskSummary,
    input: BenchmarkStartInput,
    provider: ProviderConnectionConfig,
    samples: Vec<DatasetSample>,
    stop_rx: watch::Receiver<bool>,
    publisher: BenchmarkEventPublisher,
    persistence: BenchmarkPersistence,
    client: RealProviderClient,
    protocol: RealProviderProtocol,
}

impl RealBenchmarkRuntime {
    pub fn new(
        task: BenchmarkTaskSummary,
        input: BenchmarkStartInput,
        provider: ProviderConnectionConfig,
        samples: Vec<DatasetSample>,
        stop_rx: watch::Receiver<bool>,
        publisher: BenchmarkEventPublisher,
        persistence: BenchmarkPersistence,
    ) -> anyhow::Result<Self> {
        let protocol = RealProviderProtocol::from_interface_type(&provider.interface_type)
            .unwrap_or(RealProviderProtocol::OpenAICompatible);
        Ok(Self {
            task,
            input,
            provider,
            samples,
            stop_rx,
            publisher,
            persistence,
            client: RealProviderClient::new()?,
            protocol,
        })
    }

    pub async fn run(mut self) {
        let task_id = self.task.id.clone();
        let result = self.run_inner().await;

        match result {
            Ok(RuntimeOutcome::Completed(task)) => self.publisher.task_completed(task),
            Ok(RuntimeOutcome::Cancelled) => self.publisher.task_stopped(&task_id),
            Err(error) => {
                let _ = self
                    .persistence
                    .finish_task(&task_id, "failed", 0.0, 0, 0.0)
                    .await;
                self.publisher.stage_changed(StageChangedEvent {
                    task_id: task_id.clone(),
                    stage: "failed".to_string(),
                    message: format!("真实压测运行失败：{error}"),
                    stage_index: None,
                    stage_total: None,
                    concurrency: None,
                });
            }
        }

        self.persistence.remove_task(&task_id).await;
    }

    async fn run_inner(&mut self) -> anyhow::Result<RuntimeOutcome> {
        if self.samples.is_empty() {
            return Err(anyhow::anyhow!(
                "当前数据集没有样本，请先导入适配当前模型类型的数据集"
            ));
        }

        let task_id = self.task.id.clone();
        let model_type = ModelType::normalize(&self.task.model_type);
        self.persistence
            .mark_engine_mode(&task_id, self.protocol.engine_mode())
            .await?;
        let plan = BenchmarkPlan::from_input(&self.input);
        let workload =
            WorkloadConfig::from_value(&self.task.model_type, self.input.workload_config.as_ref());
        let request_log_config =
            RequestLogConfig::normalized(self.input.request_log_config.as_ref());

        self.publisher.task_started(&self.task);
        self.publisher.stage_changed(StageChangedEvent {
            task_id: task_id.clone(),
            stage: "warmup".to_string(),
            message: format!(
                "真实 {} 压测开始：共 {} 个阶段，并发序列 {}",
                self.protocol.label(),
                plan.stages.len(),
                plan.stages
                    .iter()
                    .map(|value| value.to_string())
                    .collect::<Vec<_>>()
                    .join(" -> ")
            ),
            stage_index: None,
            stage_total: Some(plan.stages.len() as i64),
            concurrency: None,
        });

        let mut final_tick = None;
        let mut elapsed_global = 0;

        for (stage_index, concurrency) in plan.stages.iter().enumerate() {
            let stage_number = stage_index as i64 + 1;
            self.publisher.stage_changed(StageChangedEvent {
                task_id: task_id.clone(),
                stage: "stage_running".to_string(),
                message: format!(
                    "阶段 {}/{}：并发 {}，预热 {} 轮，请求采样 {} 轮，单请求超时 {}s",
                    stage_number,
                    plan.stages.len(),
                    concurrency,
                    plan.warmup_rounds,
                    plan.stage_sample_rounds,
                    plan.request_timeout_seconds
                ),
                stage_index: Some(stage_number),
                stage_total: Some(plan.stages.len() as i64),
                concurrency: Some(*concurrency),
            });

            let mut sampled_ticks = Vec::new();
            for elapsed_in_stage in 1..=(plan.warmup_rounds + plan.stage_sample_rounds) {
                if *self.stop_rx.borrow() {
                    self.finish_cancelled(&task_id).await?;
                    return Ok(RuntimeOutcome::Cancelled);
                }

                elapsed_global += 1;
                let tick = match self
                    .run_tick(
                        &task_id,
                        elapsed_global,
                        stage_number,
                        elapsed_in_stage,
                        *concurrency,
                        model_type,
                        &workload,
                        plan.request_timeout_seconds,
                        &request_log_config,
                    )
                    .await?
                {
                    TickOutcome::Tick(tick) => tick,
                    TickOutcome::Cancelled => {
                        self.finish_cancelled(&task_id).await?;
                        return Ok(RuntimeOutcome::Cancelled);
                    }
                };
                self.persistence.insert_tick(&tick).await?;
                self.publisher.metrics_tick(tick.clone());
                final_tick = Some(tick.clone());

                if elapsed_in_stage > plan.warmup_rounds {
                    sampled_ticks.push(tick);
                }
            }

            if let Some(stage_tick) = metrics::aggregate_stage_tick(&task_id, sampled_ticks) {
                let should_stop = stage_tick.latency_ms > plan.sla_p95_ms
                    || stage_tick.success_rate < plan.min_success_rate;
                let stop_reason = if should_stop {
                    Some(format!(
                        "P95 {}ms / 成功率 {:.2}% 未达到 SLA（P95 <= {}ms，成功率 >= {:.2}%）",
                        stage_tick.latency_ms,
                        stage_tick.success_rate,
                        plan.sla_p95_ms,
                        plan.min_success_rate
                    ))
                } else {
                    None
                };
                let sample = StageSample::from_tick_with_evidence(
                    stage_number,
                    *concurrency,
                    &stage_tick,
                    plan.stage_sample_rounds,
                    plan.warmup_rounds,
                    !should_stop,
                    stop_reason,
                );
                self.persistence.insert_stage(&sample).await?;
                self.publisher.stage_changed(StageChangedEvent {
                    task_id: task_id.clone(),
                    stage: if should_stop {
                        "threshold_reached".to_string()
                    } else {
                        "stage_completed".to_string()
                    },
                    message: if should_stop {
                        if plan.should_stop_on_sla_failure() {
                            format!(
                                "阶段 {} 未达 SLA：P95 {}ms / 成功率 {:.2}%，已触发保护性停止",
                                stage_number, stage_tick.latency_ms, stage_tick.success_rate
                            )
                        } else {
                            format!(
                                "阶段 {} 未达 SLA：P95 {}ms / 成功率 {:.2}%，按当前策略继续执行后续阶梯",
                                stage_number, stage_tick.latency_ms, stage_tick.success_rate
                            )
                        }
                    } else {
                        format!(
                            "阶段 {} 完成：QPS {:.2}，P95 {}ms，成功率 {:.2}%",
                            stage_number,
                            stage_tick.qps,
                            stage_tick.latency_ms,
                            stage_tick.success_rate
                        )
                    },
                    stage_index: Some(stage_number),
                    stage_total: Some(plan.stages.len() as i64),
                    concurrency: Some(*concurrency),
                });
                if should_stop && plan.is_staircase && plan.should_stop_on_sla_failure() {
                    break;
                }
            }
        }

        let completed = if let Some(tick) = final_tick {
            self.persistence
                .finish_task(
                    &task_id,
                    "completed",
                    tick.success_rate,
                    tick.latency_ms,
                    tick.qps,
                )
                .await?;
            self.persistence
                .task_summary(&task_id)
                .await
                .unwrap_or_else(|_| self.task.clone())
        } else {
            self.task.clone()
        };

        Ok(RuntimeOutcome::Completed(completed))
    }

    async fn run_tick(
        &mut self,
        task_id: &str,
        elapsed: i64,
        stage_index: i64,
        round_index: i64,
        concurrency: i64,
        model_type: ModelType,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
        request_log_config: &RequestLogConfig,
    ) -> anyhow::Result<TickOutcome> {
        let started = Instant::now();
        let request_count = concurrency.clamp(1, 256) as usize;
        let futures = (0..request_count).map(|index| {
            let sample_index = ((elapsed as usize * request_count) + index) % self.samples.len();
            let request_index = ((round_index - 1) * request_count as i64) + index as i64 + 1;
            let sample = self.samples[sample_index].clone();
            let client = self.client.clone();
            let provider = self.provider.clone();
            let protocol = self.protocol;
            let model = self.task.model_name.clone();
            let workload = workload.clone();
            let samples = self.samples.clone();
            async move {
                let outcome = match model_type {
                    ModelType::Embedding => {
                        if protocol == RealProviderProtocol::OpenAICompatible {
                            let inputs = collect_embedding_inputs(
                                &samples,
                                sample_index,
                                workload
                                    .text_count_per_request
                                    .max(workload.batch_size)
                                    .max(1) as usize,
                            );
                            client
                                .embedding(&provider, &model, inputs, request_timeout_seconds)
                                .await
                        } else {
                            RequestOutcome::failure(
                                "unsupported",
                                &format!("{} 当前不支持 Embedding 压测", protocol.label()),
                                Duration::ZERO,
                            )
                        }
                    }
                    ModelType::Rerank => {
                        if protocol == RealProviderProtocol::OpenAICompatible {
                            let (query, documents) = collect_rerank_inputs(
                                &samples,
                                sample_index,
                                workload.documents_per_query.max(1) as usize,
                            );
                            client
                                .rerank(
                                    &provider,
                                    &model,
                                    query,
                                    documents,
                                    &workload,
                                    request_timeout_seconds,
                                )
                                .await
                        } else {
                            RequestOutcome::failure(
                                "unsupported",
                                &format!("{} 当前不支持 Rerank 压测", protocol.label()),
                                Duration::ZERO,
                            )
                        }
                    }
                    ModelType::Multimodal => {
                        client
                            .vision_completion(
                                &provider,
                                protocol,
                                &model,
                                &sample,
                                &workload,
                                request_timeout_seconds,
                            )
                            .await
                    }
                    ModelType::TextGeneration => {
                        client
                            .text_generation(
                                &provider,
                                protocol,
                                &model,
                                &sample.prompt,
                                &workload,
                                request_timeout_seconds,
                            )
                            .await
                    }
                };
                outcome.with_metadata(request_index, sample_index as i64 + 1)
            }
        });
        let results = tokio::select! {
            _ = self.stop_rx.changed() => {
                if *self.stop_rx.borrow() {
                    return Ok(TickOutcome::Cancelled);
                }
                Vec::new()
            }
            results = join_all(futures) => results,
        };
        let elapsed_secs = started.elapsed().as_secs_f64().max(0.001);
        self.record_errors(task_id, &results).await?;
        self.record_request_logs(task_id, stage_index, request_log_config, &results)
            .await?;
        Ok(TickOutcome::Tick(metrics::build_tick_from_results(
            task_id,
            elapsed,
            concurrency,
            model_type,
            workload,
            elapsed_secs,
            results,
        )))
    }

    async fn record_errors(&self, task_id: &str, results: &[RequestOutcome]) -> anyhow::Result<()> {
        let mut buckets: HashMap<(String, String), i64> = HashMap::new();
        for result in results.iter().filter(|result| !result.ok) {
            let kind = result.error_kind.unwrap_or("unknown").to_string();
            let message = result
                .error_message
                .clone()
                .unwrap_or_else(|| "unknown request error".to_string());
            *buckets.entry((kind, message)).or_insert(0) += 1;
        }

        for ((error_kind, message), count) in buckets {
            self.persistence
                .insert_error(&BenchmarkErrorRecord {
                    task_id: task_id.to_string(),
                    error_kind,
                    message,
                    count,
                })
                .await?;
        }
        Ok(())
    }

    async fn record_request_logs(
        &self,
        task_id: &str,
        stage_index: i64,
        config: &RequestLogConfig,
        results: &[RequestOutcome],
    ) -> anyhow::Result<()> {
        request_logs::record_request_logs(&self.persistence, task_id, stage_index, config, results)
            .await
    }

    async fn finish_cancelled(&self, task_id: &str) -> anyhow::Result<()> {
        self.persistence
            .insert_error(&BenchmarkErrorRecord {
                task_id: task_id.to_string(),
                error_kind: "cancelled".to_string(),
                message: "任务被用户停止".to_string(),
                count: 1,
            })
            .await?;
        self.persistence
            .finish_task(task_id, "cancelled", 0.0, 0, 0.0)
            .await
    }
}

enum RuntimeOutcome {
    Completed(BenchmarkTaskSummary),
    Cancelled,
}

enum TickOutcome {
    Tick(MetricsTick),
    Cancelled,
}
