mod chat;
pub mod diagnostics;
mod embedding;
mod rerank;
mod vision;

use crate::models::{DiscoveredModel, ProviderConnectionConfig};
use crate::{
    benchmark::{
        persistence::BenchmarkPersistence, plan::BenchmarkPlan, publisher::BenchmarkEventPublisher,
    },
    domain::{benchmark_sample::StageSample, model_type::ModelType, workload::WorkloadConfig},
    models::{
        BenchmarkErrorRecord, BenchmarkRequestLogRecord, BenchmarkRequestLogSummary,
        BenchmarkStartInput, BenchmarkTaskSummary, DatasetSample, MetricsTick, RequestLogConfig,
        StageChangedEvent,
    },
};
use futures_util::{future::join_all, StreamExt};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::Instant;
use uuid::Uuid;

#[derive(Clone)]
pub struct OpenAICompatibleClient {
    client: Client,
}

impl OpenAICompatibleClient {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: Client::builder().build()?,
        })
    }

    pub async fn list_models(
        &self,
        config: &ProviderConnectionConfig,
    ) -> anyhow::Result<Vec<DiscoveredModel>> {
        let url = api_url(&config.base_url, "models");
        let response = self
            .with_auth(self.client.get(url), config)
            .timeout(Duration::from_secs(20))
            .send()
            .await
            .map_err(map_reqwest_error)?;

        ensure_success(response.status(), "models list")?;
        let payload = response.json::<ModelsResponse>().await?;
        Ok(payload
            .data
            .into_iter()
            .map(|model| classify_model(&model.id))
            .collect())
    }

    async fn chat_completion(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        prompt: &str,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        if workload.streaming {
            self.chat_completion_streaming(config, model, prompt, workload, request_timeout_seconds)
                .await
        } else {
            self.chat_completion_non_streaming(
                config,
                model,
                prompt,
                workload,
                request_timeout_seconds,
            )
            .await
        }
    }

    async fn embedding(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        inputs: Vec<String>,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        let started = Instant::now();
        let input_tokens = inputs.iter().map(|item| estimate_tokens(item)).sum::<i64>();
        let text_count = inputs.len() as i64;
        let prompt_for_log = inputs.join("\n---\n");
        let body = embedding::embeddings_body(model, inputs);
        let response = match self
            .with_auth(
                self.client.post(api_url(&config.base_url, "embeddings")),
                config,
            )
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt_for_log),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt_for_log),
                None,
                None,
            );
        }
        let payload = match response.json::<serde_json::Value>().await {
            Ok(payload) => payload,
            Err(error) => {
                return RequestOutcome::failure("parse", &error.to_string(), started.elapsed())
                    .with_body(Some(prompt_for_log), None, None)
            }
        };
        let usage = usage_from_value(&payload).unwrap_or(TokenUsage {
            input_tokens,
            output_tokens: 0,
            total_tokens: input_tokens,
        });
        RequestOutcome::success_with_units(
            started.elapsed(),
            Duration::ZERO,
            usage,
            RequestUnits {
                batch_size: text_count,
                text_count,
                ..RequestUnits::default()
            },
        )
        .with_body(
            Some(prompt_for_log),
            Some(format!(
                "Embedding 返回 {text_count} 条向量，向量正文未保存。"
            )),
            raw_usage_from_value(&payload),
        )
    }

    async fn rerank(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        query: String,
        documents: Vec<String>,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        let started = Instant::now();
        let documents_per_query = documents.len() as i64;
        let query_for_log = query.clone();
        let documents_for_log = documents.clone();
        let prompt_for_log = rerank_prompt_for_log(&query_for_log, &documents_for_log);
        let input_tokens = estimate_tokens(&query)
            + documents
                .iter()
                .map(|item| estimate_tokens(item))
                .sum::<i64>();
        let body = rerank::rerank_body(model, query, documents, workload);
        let response = match self
            .with_auth(
                self.client.post(api_url(&config.base_url, "rerank")),
                config,
            )
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt_for_log),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt_for_log),
                None,
                None,
            );
        }
        let payload = match response.json::<serde_json::Value>().await {
            Ok(payload) => payload,
            Err(error) => {
                return RequestOutcome::failure("parse", &error.to_string(), started.elapsed())
                    .with_body(Some(prompt_for_log), None, None)
            }
        };
        let result_count = payload
            .get("results")
            .and_then(|value| value.as_array())
            .map(|items| items.len())
            .unwrap_or(0);
        RequestOutcome::success_with_units(
            started.elapsed(),
            Duration::ZERO,
            TokenUsage {
                input_tokens,
                output_tokens: 0,
                total_tokens: input_tokens,
            },
            RequestUnits {
                documents_per_query,
                pair_count: documents_per_query,
                ..RequestUnits::default()
            },
        )
        .with_body(
            Some(prompt_for_log),
            Some(format!(
                "Rerank 返回 {result_count} 条结果，候选文档 {documents_per_query} 条。"
            )),
            raw_usage_from_value(&payload),
        )
    }

    async fn chat_completion_non_streaming(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        prompt: &str,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        let started = Instant::now();
        let body = chat::completion_body(model, prompt, workload, false, 0.7);
        let response = match self
            .with_auth(
                self.client
                    .post(api_url(&config.base_url, "chat/completions")),
                config,
            )
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt.to_string()),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt.to_string()),
                None,
                None,
            );
        }
        let payload = match response.json::<serde_json::Value>().await {
            Ok(payload) => payload,
            Err(error) => {
                return RequestOutcome::failure("parse", &error.to_string(), started.elapsed())
                    .with_body(Some(prompt.to_string()), None, None)
            }
        };
        let output = payload
            .pointer("/choices/0/message/content")
            .and_then(|value| value.as_str())
            .unwrap_or("")
            .to_string();
        RequestOutcome::success(
            started.elapsed(),
            started.elapsed(),
            usage_from_value(&payload).unwrap_or_else(|| TokenUsage::estimated(prompt, &output)),
        )
        .with_body(
            Some(prompt.to_string()),
            Some(output),
            raw_usage_from_value(&payload),
        )
    }

    async fn chat_completion_streaming(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        prompt: &str,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        let started = Instant::now();
        let body = chat::streaming_completion_body(model, prompt, workload);
        let response = match self
            .with_auth(
                self.client
                    .post(api_url(&config.base_url, "chat/completions")),
                config,
            )
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt.to_string()),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt.to_string()),
                None,
                None,
            );
        }

        let mut stream = response.bytes_stream();
        let mut output = String::new();
        let mut first_token_at = None;
        let mut usage = None;

        while let Some(chunk) = stream.next().await {
            let chunk =
                match chunk {
                    Ok(chunk) => chunk,
                    Err(error) => {
                        return RequestOutcome::from_reqwest_error(error, started.elapsed())
                            .with_body(Some(prompt.to_string()), Some(output), None)
                    }
                };
            let text = String::from_utf8_lossy(&chunk);
            for line in text.lines() {
                let Some(data) = line.trim().strip_prefix("data:") else {
                    continue;
                };
                let data = data.trim();
                if data.is_empty() || data == "[DONE]" {
                    continue;
                }
                let Ok(value) = serde_json::from_str::<serde_json::Value>(data) else {
                    continue;
                };
                if let Some(found_usage) = usage_from_value(&value) {
                    usage = Some(found_usage);
                }
                let content = value
                    .pointer("/choices/0/delta/content")
                    .and_then(|value| value.as_str())
                    .unwrap_or("");
                if !content.is_empty() {
                    if first_token_at.is_none() {
                        first_token_at = Some(started.elapsed());
                    }
                    output.push_str(content);
                }
            }
        }

        RequestOutcome::success(
            started.elapsed(),
            first_token_at.unwrap_or_else(|| started.elapsed()),
            usage.unwrap_or_else(|| TokenUsage::estimated(prompt, &output)),
        )
        .with_body(Some(prompt.to_string()), Some(output), None)
    }

    async fn vision_completion(
        &self,
        config: &ProviderConnectionConfig,
        model: &str,
        sample: &DatasetSample,
        workload: &WorkloadConfig,
        request_timeout_seconds: i64,
    ) -> RequestOutcome {
        let started = Instant::now();
        let vision_sample = parse_vision_sample(&sample.prompt, workload.image_count);
        let prompt_for_log = sample.prompt.clone();
        let body = vision::vision_completion_body(model, &vision_sample, workload);
        let response = match self
            .with_auth(
                self.client
                    .post(api_url(&config.base_url, "chat/completions")),
                config,
            )
            .timeout(request_timeout(request_timeout_seconds))
            .json(&body)
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt_for_log),
                    None,
                    None,
                )
            }
        };
        let status = response.status();
        if !status.is_success() {
            return RequestOutcome::from_status(status, started.elapsed()).with_body(
                Some(prompt_for_log),
                None,
                None,
            );
        }
        let payload = match response.json::<serde_json::Value>().await {
            Ok(payload) => payload,
            Err(error) => {
                return RequestOutcome::failure("parse", &error.to_string(), started.elapsed())
                    .with_body(Some(prompt_for_log), None, None)
            }
        };
        let output = payload
            .pointer("/choices/0/message/content")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let usage = usage_from_value(&payload)
            .unwrap_or_else(|| TokenUsage::estimated(&sample.prompt, output));
        RequestOutcome::success_with_units(
            started.elapsed(),
            started.elapsed(),
            usage,
            RequestUnits {
                image_count: vision_sample.image_urls.len() as i64,
                ..RequestUnits::default()
            },
        )
        .with_body(
            Some(prompt_for_log),
            Some(output.to_string()),
            raw_usage_from_value(&payload),
        )
    }
}

pub struct OpenAICompatibleBenchmarkRuntime {
    task: BenchmarkTaskSummary,
    input: BenchmarkStartInput,
    provider: ProviderConnectionConfig,
    samples: Vec<DatasetSample>,
    stop_rx: watch::Receiver<bool>,
    publisher: BenchmarkEventPublisher,
    persistence: BenchmarkPersistence,
    client: OpenAICompatibleClient,
}

impl OpenAICompatibleBenchmarkRuntime {
    pub fn new(
        task: BenchmarkTaskSummary,
        input: BenchmarkStartInput,
        provider: ProviderConnectionConfig,
        samples: Vec<DatasetSample>,
        stop_rx: watch::Receiver<bool>,
        publisher: BenchmarkEventPublisher,
        persistence: BenchmarkPersistence,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            task,
            input,
            provider,
            samples,
            stop_rx,
            publisher,
            persistence,
            client: OpenAICompatibleClient::new()?,
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
            .mark_engine_mode(&task_id, "openai_compatible")
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
                "真实 OpenAI Compatible 压测开始：共 {} 个阶段，并发序列 {}",
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

            if let Some(stage_tick) = aggregate_stage_tick(&task_id, sampled_ticks) {
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
            let model = self.task.model_name.clone();
            let workload = workload.clone();
            let samples = self.samples.clone();
            async move {
                let outcome = match model_type {
                    ModelType::Embedding => {
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
                    }
                    ModelType::Rerank => {
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
                    }
                    ModelType::Multimodal => {
                        client
                            .vision_completion(
                                &provider,
                                &model,
                                &sample,
                                &workload,
                                request_timeout_seconds,
                            )
                            .await
                    }
                    ModelType::TextGeneration => {
                        client
                            .chat_completion(
                                &provider,
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
        Ok(TickOutcome::Tick(build_tick_from_results(
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
        if !config.enabled {
            return Ok(());
        }

        for result in results
            .iter()
            .filter(|result| result.request_index <= config.max_records_per_stage)
        {
            let status = if result.ok { "success" } else { "failed" }.to_string();
            let prompt_preview = result.prompt.as_deref().map(preview_text);
            let response_preview = result.response_text.as_deref().map(preview_text);
            let summary = BenchmarkRequestLogSummary {
                id: Uuid::new_v4().to_string(),
                task_id: task_id.to_string(),
                stage_index,
                request_index: result.request_index,
                sample_index: result.sample_index,
                status,
                latency_ms: result.latency_ms,
                ttft_ms: result.ttft_ms,
                input_tokens: result.usage.input_tokens,
                output_tokens: result.usage.output_tokens,
                total_tokens: result.usage.total_tokens,
                error_kind: result.error_kind.map(str::to_string),
                prompt_preview,
                response_preview,
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            self.persistence
                .insert_request_log(BenchmarkRequestLogRecord {
                    summary,
                    body_ref: None,
                    prompt: config.capture_body.then(|| result.prompt.clone()).flatten(),
                    response_text: config
                        .capture_body
                        .then(|| result.response_text.clone())
                        .flatten(),
                    raw_error: config
                        .capture_body
                        .then(|| result.error_message.clone())
                        .flatten(),
                    raw_usage: config
                        .capture_body
                        .then(|| result.raw_usage.clone())
                        .flatten(),
                })
                .await?;
        }
        Ok(())
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

pub fn api_url(base_url: &str, path: &str) -> String {
    format!(
        "{}/{}",
        normalized_api_base_url(base_url),
        path.trim_start_matches('/')
    )
}

fn normalized_api_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return trimmed.to_string();
    }

    let without_scheme = trimmed
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(trimmed);
    let path = without_scheme
        .split_once('/')
        .map(|(_, path)| path)
        .unwrap_or("");

    if path.is_empty() {
        format!("{trimmed}/v1")
    } else {
        trimmed.to_string()
    }
}

pub fn classify_model(model_name: &str) -> DiscoveredModel {
    let lower = model_name.to_ascii_lowercase();
    let model_type = if lower.contains("rerank") || lower.contains("reranker") {
        "reranker"
    } else if lower.contains("embedding")
        || lower.contains("embed")
        || lower.contains("bge")
        || lower.contains("e5")
    {
        "embedding"
    } else if lower.contains("vision")
        || lower.contains("qwen-vl")
        || lower.contains("-vl")
        || lower.contains("_vl")
        || lower.contains("llava")
    {
        "multimodal"
    } else {
        "text_generation"
    };

    let capabilities = match model_type {
        "embedding" => vec!["embedding".to_string()],
        "reranker" => vec!["rerank".to_string()],
        "multimodal" => vec!["streaming".to_string(), "image_input".to_string()],
        _ => vec!["streaming".to_string(), "chat".to_string()],
    };

    DiscoveredModel {
        name: model_name.to_string(),
        model_type: model_type.to_string(),
        capabilities,
        supports_streaming: matches!(model_type, "text_generation" | "multimodal"),
        recommended_concurrency: None,
    }
}

impl OpenAICompatibleClient {
    fn with_auth(
        &self,
        request: reqwest::RequestBuilder,
        config: &ProviderConnectionConfig,
    ) -> reqwest::RequestBuilder {
        let key = config.api_key_plaintext.trim();
        if key.is_empty() {
            request
        } else {
            request.bearer_auth(key)
        }
    }
}

fn ensure_success(status: StatusCode, operation: &str) -> anyhow::Result<()> {
    if status.is_success() {
        return Ok(());
    }

    let message = if status == StatusCode::NOT_FOUND {
        format!(
            "{operation} failed with HTTP {status}; check Base URL (OpenAI Compatible usually ends with /v1), API Key and model permissions"
        )
    } else if status.is_client_error() {
        format!(
            "{operation} failed with HTTP {status}; check Base URL, API Key and model permissions"
        )
    } else if status.is_server_error() {
        format!("{operation} failed with HTTP {status}; provider service returned an error")
    } else {
        format!("{operation} failed with HTTP {status}")
    };
    Err(anyhow::anyhow!(message))
}

fn map_reqwest_error(error: reqwest::Error) -> anyhow::Error {
    if error.is_timeout() {
        anyhow::anyhow!("request timeout while connecting provider")
    } else if error.is_connect() {
        anyhow::anyhow!("failed to connect provider endpoint")
    } else {
        anyhow::anyhow!("provider request failed: {}", error)
    }
}

#[derive(Debug, Clone)]
struct RequestOutcome {
    ok: bool,
    request_index: i64,
    sample_index: i64,
    latency_ms: i64,
    ttft_ms: i64,
    usage: TokenUsage,
    units: RequestUnits,
    error_kind: Option<&'static str>,
    error_message: Option<String>,
    prompt: Option<String>,
    response_text: Option<String>,
    raw_usage: Option<serde_json::Value>,
}

impl RequestOutcome {
    fn success(latency: Duration, ttft: Duration, usage: TokenUsage) -> Self {
        Self::success_with_units(latency, ttft, usage, RequestUnits::default())
    }

    fn success_with_units(
        latency: Duration,
        ttft: Duration,
        usage: TokenUsage,
        units: RequestUnits,
    ) -> Self {
        Self {
            ok: true,
            request_index: 0,
            sample_index: 0,
            latency_ms: duration_ms(latency),
            ttft_ms: duration_ms(ttft),
            usage,
            units,
            error_kind: None,
            error_message: None,
            prompt: None,
            response_text: None,
            raw_usage: None,
        }
    }

    fn failure(kind: &'static str, message: &str, latency: Duration) -> Self {
        Self {
            ok: false,
            request_index: 0,
            sample_index: 0,
            latency_ms: duration_ms(latency),
            ttft_ms: 0,
            usage: TokenUsage::default(),
            units: RequestUnits::default(),
            error_kind: Some(kind),
            error_message: Some(message.chars().take(180).collect()),
            prompt: None,
            response_text: None,
            raw_usage: None,
        }
    }

    fn with_metadata(mut self, request_index: i64, sample_index: i64) -> Self {
        self.request_index = request_index;
        self.sample_index = sample_index;
        self
    }

    fn with_body(
        mut self,
        prompt: Option<String>,
        response_text: Option<String>,
        raw_usage: Option<serde_json::Value>,
    ) -> Self {
        self.prompt = prompt;
        self.response_text = response_text;
        self.raw_usage = raw_usage;
        self
    }

    fn from_status(status: StatusCode, latency: Duration) -> Self {
        let kind = if status.is_client_error() {
            "http_4xx"
        } else if status.is_server_error() {
            "http_5xx"
        } else {
            "http"
        };
        Self::failure(kind, &format!("HTTP {status}"), latency)
    }

    fn from_reqwest_error(error: reqwest::Error, latency: Duration) -> Self {
        let kind = if error.is_timeout() {
            "timeout"
        } else if error.is_connect() {
            "connection"
        } else {
            "request"
        };
        Self::failure(kind, &error.to_string(), latency)
    }
}

#[derive(Debug, Clone, Default)]
struct RequestUnits {
    batch_size: i64,
    text_count: i64,
    documents_per_query: i64,
    pair_count: i64,
    image_count: i64,
}

#[derive(Debug, Clone, Default)]
struct TokenUsage {
    input_tokens: i64,
    output_tokens: i64,
    total_tokens: i64,
}

impl TokenUsage {
    fn estimated(prompt: &str, output: &str) -> Self {
        let input_tokens = estimate_tokens(prompt);
        let output_tokens = estimate_tokens(output);
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }
}

fn usage_from_value(value: &serde_json::Value) -> Option<TokenUsage> {
    let usage = value.get("usage")?;
    let input_tokens = usage
        .get("prompt_tokens")
        .or_else(|| usage.get("input_tokens"))
        .and_then(|item| item.as_i64())
        .unwrap_or(0);
    let output_tokens = usage
        .get("completion_tokens")
        .or_else(|| usage.get("output_tokens"))
        .and_then(|item| item.as_i64())
        .unwrap_or(0);
    let total_tokens = usage
        .get("total_tokens")
        .and_then(|item| item.as_i64())
        .unwrap_or(input_tokens + output_tokens);
    Some(TokenUsage {
        input_tokens,
        output_tokens,
        total_tokens,
    })
}

fn raw_usage_from_value(value: &serde_json::Value) -> Option<serde_json::Value> {
    value.get("usage").cloned()
}

fn build_tick_from_results(
    task_id: &str,
    elapsed: i64,
    concurrency: i64,
    model_type: ModelType,
    workload: &WorkloadConfig,
    elapsed_secs: f64,
    results: Vec<RequestOutcome>,
) -> MetricsTick {
    let total = results.len().max(1) as f64;
    let request_count = results.len() as i64;
    let success_count = results.iter().filter(|result| result.ok).count() as i64;
    let failure_count = request_count - success_count;
    let mut latencies = results
        .iter()
        .map(|result| result.latency_ms)
        .collect::<Vec<_>>();
    let mut ttfts = results
        .iter()
        .filter(|result| result.ok && result.ttft_ms > 0)
        .map(|result| result.ttft_ms)
        .collect::<Vec<_>>();
    let input_tokens = results
        .iter()
        .map(|result| result.usage.input_tokens)
        .sum::<i64>();
    let output_tokens = results
        .iter()
        .map(|result| result.usage.output_tokens)
        .sum::<i64>();
    let total_tokens = results
        .iter()
        .map(|result| result.usage.total_tokens)
        .sum::<i64>();
    let text_total = results
        .iter()
        .map(|result| result.units.text_count)
        .sum::<i64>();
    let pair_total = results
        .iter()
        .map(|result| result.units.pair_count)
        .sum::<i64>();
    let image_total = results
        .iter()
        .map(|result| result.units.image_count)
        .sum::<i64>();
    let batch_size = results
        .iter()
        .map(|result| result.units.batch_size)
        .max()
        .unwrap_or(workload.batch_size);
    let documents_per_query = results
        .iter()
        .map(|result| result.units.documents_per_query)
        .max()
        .unwrap_or(workload.documents_per_query);
    let qps = success_count as f64 / elapsed_secs;
    let tps = match model_type {
        ModelType::Embedding => input_tokens as f64 / elapsed_secs,
        ModelType::Rerank => pair_total as f64 / elapsed_secs,
        _ => output_tokens as f64 / elapsed_secs,
    };

    MetricsTick {
        task_id: task_id.to_string(),
        elapsed_seconds: elapsed,
        qps: round2(qps),
        latency_ms: percentile_ms(&mut latencies, 0.95),
        ttft_ms: percentile_ms(&mut ttfts, 0.95),
        tps: round2(tps),
        success_rate: round2(success_count as f64 / total * 100.0),
        errors: failure_count,
        in_flight: concurrency,
        request_count,
        success_count,
        failure_count,
        input_tokens,
        output_tokens,
        total_tokens,
        batch_size,
        text_count: round_i64(text_total as f64 / elapsed_secs),
        documents_per_query,
        pair_count: round_i64(pair_total as f64 / elapsed_secs),
        image_count: if request_count > 0 {
            (image_total / request_count).max(0)
        } else {
            0
        },
    }
}

fn aggregate_stage_tick(task_id: &str, ticks: Vec<MetricsTick>) -> Option<MetricsTick> {
    let first = ticks.first()?.clone();
    let count = ticks.len() as f64;
    let mut latencies = ticks.iter().map(|tick| tick.latency_ms).collect::<Vec<_>>();
    let mut ttfts = ticks.iter().map(|tick| tick.ttft_ms).collect::<Vec<_>>();
    let input_tokens = ticks.iter().map(|tick| tick.input_tokens).sum::<i64>();
    let output_tokens = ticks.iter().map(|tick| tick.output_tokens).sum::<i64>();
    let total_tokens = ticks.iter().map(|tick| tick.total_tokens).sum::<i64>();
    let errors = ticks.iter().map(|tick| tick.errors).sum::<i64>();
    let request_count = ticks.iter().map(|tick| tick.request_count).sum::<i64>();
    let success_count = ticks.iter().map(|tick| tick.success_count).sum::<i64>();
    let failure_count = ticks.iter().map(|tick| tick.failure_count).sum::<i64>();
    let batch_size = ticks.iter().map(|tick| tick.batch_size).max().unwrap_or(0);
    let text_count = ticks.iter().map(|tick| tick.text_count).sum::<i64>() / ticks.len() as i64;
    let documents_per_query = ticks
        .iter()
        .map(|tick| tick.documents_per_query)
        .max()
        .unwrap_or(0);
    let pair_count = ticks.iter().map(|tick| tick.pair_count).sum::<i64>() / ticks.len() as i64;
    let image_count = ticks.iter().map(|tick| tick.image_count).max().unwrap_or(0);
    Some(MetricsTick {
        task_id: task_id.to_string(),
        elapsed_seconds: first.elapsed_seconds,
        qps: round2(ticks.iter().map(|tick| tick.qps).sum::<f64>() / count),
        latency_ms: percentile_ms(&mut latencies, 0.95),
        ttft_ms: percentile_ms(&mut ttfts, 0.95),
        tps: round2(ticks.iter().map(|tick| tick.tps).sum::<f64>() / count),
        success_rate: if request_count > 0 {
            round2(success_count as f64 / request_count as f64 * 100.0)
        } else {
            round2(ticks.iter().map(|tick| tick.success_rate).sum::<f64>() / count)
        },
        errors,
        in_flight: first.in_flight,
        request_count,
        success_count,
        failure_count,
        input_tokens,
        output_tokens,
        total_tokens,
        batch_size,
        text_count,
        documents_per_query,
        pair_count,
        image_count,
    })
}

fn percentile_ms(values: &mut [i64], percentile: f64) -> i64 {
    if values.is_empty() {
        return 0;
    }
    values.sort_unstable();
    let index = ((values.len() as f64 - 1.0) * percentile).ceil() as usize;
    values[index.min(values.len() - 1)]
}

fn estimate_tokens(text: &str) -> i64 {
    ((text.chars().count() as f64) / 4.0).ceil().max(1.0) as i64
}

fn preview_text(text: &str) -> String {
    const MAX_CHARS: usize = 120;
    let mut preview = text.chars().take(MAX_CHARS).collect::<String>();
    if text.chars().count() > MAX_CHARS {
        preview.push('…');
    }
    preview
}

fn collect_embedding_inputs(
    samples: &[DatasetSample],
    start_index: usize,
    count: usize,
) -> Vec<String> {
    if samples.is_empty() {
        return Vec::new();
    }
    (0..count.max(1))
        .map(|offset| {
            samples[(start_index + offset) % samples.len()]
                .prompt
                .clone()
        })
        .collect()
}

fn collect_rerank_inputs(
    samples: &[DatasetSample],
    start_index: usize,
    documents_per_query: usize,
) -> (String, Vec<String>) {
    if samples.is_empty() {
        return ("".to_string(), Vec::new());
    }
    let query = samples[start_index % samples.len()].prompt.clone();
    let documents = (0..documents_per_query.max(1))
        .map(|offset| {
            samples[(start_index + offset + 1) % samples.len()]
                .prompt
                .clone()
        })
        .collect();
    (query, documents)
}

fn rerank_prompt_for_log(query: &str, documents: &[String]) -> String {
    let docs = documents
        .iter()
        .enumerate()
        .map(|(index, document)| format!("{}. {}", index + 1, document))
        .collect::<Vec<_>>()
        .join("\n");
    format!("Query:\n{query}\n\nDocuments:\n{docs}")
}

pub(super) struct VisionSample {
    prompt: String,
    image_urls: Vec<String>,
}

pub(super) fn parse_vision_sample(raw: &str, image_limit: i64) -> VisionSample {
    let limit = image_limit.clamp(1, 8) as usize;
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
        let prompt = value
            .get("prompt")
            .or_else(|| value.get("text"))
            .or_else(|| value.get("input"))
            .and_then(|item| item.as_str())
            .unwrap_or("请分析这张图片。")
            .to_string();
        let mut image_urls = Vec::new();
        if let Some(image_url) = value
            .get("image_url")
            .or_else(|| value.get("image"))
            .and_then(|item| item.as_str())
        {
            image_urls.push(image_url.to_string());
        }
        if let Some(items) = value
            .get("image_urls")
            .or_else(|| value.get("images"))
            .and_then(|item| item.as_array())
        {
            image_urls.extend(
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .map(ToString::to_string),
            );
        }
        image_urls.truncate(limit);
        return VisionSample { prompt, image_urls };
    }

    let trimmed = raw.trim();
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("data:image/")
    {
        return VisionSample {
            prompt: "请分析这张图片。".to_string(),
            image_urls: vec![trimmed.to_string()],
        };
    }

    VisionSample {
        prompt: trimmed.to_string(),
        image_urls: Vec::new(),
    }
}

pub(super) fn duration_ms(duration: Duration) -> i64 {
    duration.as_millis().min(i64::MAX as u128) as i64
}

pub(super) fn request_timeout(seconds: i64) -> Duration {
    Duration::from_secs(seconds.clamp(5, 600) as u64)
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round_i64(value: f64) -> i64 {
    value.round().max(0.0) as i64
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    pub(super) data: Vec<ModelItem>,
}

#[derive(Debug, Deserialize)]
struct ModelItem {
    pub(super) id: String,
}

#[cfg(test)]
mod tests {
    use super::{
        api_url, build_tick_from_results, classify_model, parse_vision_sample,
        OpenAICompatibleClient, RequestOutcome, RequestUnits, TokenUsage,
    };
    use crate::config::BenchmarkEngineMode;
    use crate::domain::{model_type::ModelType, workload::WorkloadConfig};
    use crate::models::{ProviderConnectionConfig, ProviderDiagnosticsInput};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn joins_api_url_without_duplicate_slashes() {
        assert_eq!(
            api_url("https://example.com", "models"),
            "https://example.com/v1/models"
        );
        assert_eq!(
            api_url("https://example.com/", "models"),
            "https://example.com/v1/models"
        );
        assert_eq!(
            api_url("https://example.com/v1/", "/models"),
            "https://example.com/v1/models"
        );
        assert_eq!(
            api_url("https://example.com/openai/v1/", "/models"),
            "https://example.com/openai/v1/models"
        );
    }

    #[test]
    fn classifies_common_model_names() {
        assert_eq!(classify_model("bge-m3").model_type, "embedding");
        assert_eq!(classify_model("bce-reranker").model_type, "reranker");
        assert_eq!(classify_model("qwen-vl").model_type, "multimodal");
        assert_eq!(classify_model("deepseek-r1").model_type, "text_generation");
    }

    #[test]
    fn parses_vision_json_samples_with_image_limit() {
        let sample = parse_vision_sample(
            r#"{"prompt":"描述图片","image_urls":["https://a.test/1.png","https://a.test/2.png"]}"#,
            1,
        );

        assert_eq!(sample.prompt, "描述图片");
        assert_eq!(sample.image_urls, vec!["https://a.test/1.png"]);
    }

    #[test]
    fn embedding_tick_uses_input_token_throughput_and_batch_units() {
        let workload = WorkloadConfig::for_model_type("embedding");
        let results = vec![
            RequestOutcome::success_with_units(
                Duration::from_millis(80),
                Duration::ZERO,
                TokenUsage {
                    input_tokens: 160,
                    output_tokens: 0,
                    total_tokens: 160,
                },
                RequestUnits {
                    batch_size: 16,
                    text_count: 16,
                    ..RequestUnits::default()
                },
            ),
            RequestOutcome::success_with_units(
                Duration::from_millis(90),
                Duration::ZERO,
                TokenUsage {
                    input_tokens: 160,
                    output_tokens: 0,
                    total_tokens: 160,
                },
                RequestUnits {
                    batch_size: 16,
                    text_count: 16,
                    ..RequestUnits::default()
                },
            ),
        ];

        let tick =
            build_tick_from_results("task", 1, 2, ModelType::Embedding, &workload, 2.0, results);

        assert_eq!(tick.request_count, 2);
        assert_eq!(tick.success_count, 2);
        assert_eq!(tick.batch_size, 16);
        assert_eq!(tick.text_count, 16);
        assert_eq!(tick.tps, 160.0);
        assert_eq!(tick.ttft_ms, 0);
    }

    #[test]
    fn rerank_tick_uses_pair_throughput() {
        let workload = WorkloadConfig::for_model_type("rerank");
        let results = vec![RequestOutcome::success_with_units(
            Duration::from_millis(120),
            Duration::ZERO,
            TokenUsage {
                input_tokens: 300,
                output_tokens: 0,
                total_tokens: 300,
            },
            RequestUnits {
                documents_per_query: 30,
                pair_count: 30,
                ..RequestUnits::default()
            },
        )];

        let tick =
            build_tick_from_results("task", 1, 1, ModelType::Rerank, &workload, 1.5, results);

        assert_eq!(tick.documents_per_query, 30);
        assert_eq!(tick.pair_count, 20);
        assert_eq!(tick.tps, 20.0);
        assert_eq!(tick.ttft_ms, 0);
    }

    #[test]
    fn vision_tick_preserves_image_units_and_output_throughput() {
        let workload = WorkloadConfig::for_model_type("multimodal");
        let results = vec![RequestOutcome::success_with_units(
            Duration::from_millis(500),
            Duration::from_millis(180),
            TokenUsage {
                input_tokens: 120,
                output_tokens: 40,
                total_tokens: 160,
            },
            RequestUnits {
                image_count: 2,
                ..RequestUnits::default()
            },
        )];

        let tick =
            build_tick_from_results("task", 1, 1, ModelType::Multimodal, &workload, 2.0, results);

        assert_eq!(tick.image_count, 2);
        assert_eq!(tick.ttft_ms, 180);
        assert_eq!(tick.tps, 20.0);
    }

    #[tokio::test]
    async fn diagnostics_probe_models_and_chat_without_exposing_key() {
        let (base_url, requests, handle) = spawn_test_server();
        let config = ProviderConnectionConfig {
            id: "provider-1".to_string(),
            name: "Local Test".to_string(),
            base_url,
            api_key_plaintext: "sk-secret-for-test".to_string(),
            interface_type: "OpenAI".to_string(),
        };
        let client = OpenAICompatibleClient::new().unwrap();

        let result = client
            .diagnose_provider(
                &config,
                &ProviderDiagnosticsInput {
                    provider_id: config.id.clone(),
                    model_id: None,
                    dataset_id: None,
                },
                &[],
                &[],
                BenchmarkEngineMode::OpenaiCompatible,
                "2026-07-13T00:00:00Z".to_string(),
            )
            .await;

        handle.join().unwrap();
        assert_eq!(result.status, "passed");
        assert!(result
            .endpoints
            .iter()
            .any(|endpoint| endpoint.path == "/models"));
        assert!(result
            .endpoints
            .iter()
            .any(|endpoint| endpoint.path == "/chat/completions"));
        let serialized = serde_json::to_string(&result).unwrap();
        assert!(!serialized.contains("sk-secret-for-test"));
        assert!(requests.lock().unwrap().iter().any(|request| request
            .to_ascii_lowercase()
            .contains("authorization: bearer sk-secret-for-test")));
    }

    fn spawn_test_server() -> (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let requests_for_thread = Arc::clone(&requests);
        let handle = thread::spawn(move || {
            for _ in 0..2 {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0_u8; 8192];
                let size = stream.read(&mut buffer).unwrap();
                let request = String::from_utf8_lossy(&buffer[..size]).to_string();
                requests_for_thread.lock().unwrap().push(request.clone());
                let body = if request.starts_with("GET /v1/models") {
                    r#"{"data":[{"id":"gpt-test"}]}"#
                } else {
                    r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"prompt_tokens":4,"completion_tokens":1,"total_tokens":5}}"#
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        (format!("http://{addr}/v1"), requests, handle)
    }
}
