use crate::benchmark::adapters::{BenchmarkAdapter, TickInput};
use crate::benchmark::messages;
use crate::benchmark::persistence::BenchmarkPersistence;
use crate::benchmark::plan::BenchmarkPlan;
use crate::benchmark::publisher::BenchmarkEventPublisher;
use crate::domain::benchmark_sample::StageSample;
use crate::models::{BenchmarkStartInput, BenchmarkTaskSummary, MetricsTick, StageChangedEvent};
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

pub struct MockBenchmarkRuntime {
    task: BenchmarkTaskSummary,
    input: BenchmarkStartInput,
    stop_rx: watch::Receiver<bool>,
    publisher: BenchmarkEventPublisher,
    persistence: BenchmarkPersistence,
}

enum RuntimeOutcome {
    Completed(BenchmarkTaskSummary),
    Cancelled,
}

impl MockBenchmarkRuntime {
    pub fn new(
        task: BenchmarkTaskSummary,
        input: BenchmarkStartInput,
        stop_rx: watch::Receiver<bool>,
        publisher: BenchmarkEventPublisher,
        persistence: BenchmarkPersistence,
    ) -> Self {
        Self {
            task,
            input,
            stop_rx,
            publisher,
            persistence,
        }
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
                    message: messages::runtime_failed(&error),
                    stage_index: None,
                    stage_total: None,
                    concurrency: None,
                });
            }
        }

        self.persistence.remove_task(&task_id).await;
    }

    async fn run_inner(&mut self) -> anyhow::Result<RuntimeOutcome> {
        let task_id = self.task.id.clone();
        let model_type = self.task.model_type.clone();
        let plan = BenchmarkPlan::from_input(&self.input);
        let adapter = BenchmarkAdapter::from_input(&model_type, &self.input);

        self.publisher.task_started(&self.task);
        self.publisher.stage_changed(StageChangedEvent {
            task_id: task_id.clone(),
            stage: "warmup".to_string(),
            message: messages::warmup_started(plan.is_staircase, &plan.stages),
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
                message: messages::stage_running(
                    stage_number,
                    plan.stages.len(),
                    *concurrency,
                    plan.warmup_rounds,
                    plan.stage_sample_rounds,
                ),
                stage_index: Some(stage_number),
                stage_total: Some(plan.stages.len() as i64),
                concurrency: Some(*concurrency),
            });

            self.emit_initial_tick(
                &adapter,
                &task_id,
                &plan,
                stage_number,
                *concurrency,
                elapsed_global,
                &mut final_tick,
            )
            .await?;

            for elapsed_in_stage in 1..=(plan.warmup_rounds + plan.stage_sample_rounds) {
                if self.wait_or_stop(&task_id).await? {
                    return Ok(RuntimeOutcome::Cancelled);
                }

                elapsed_global += 1;
                let sample_elapsed = elapsed_in_stage.saturating_sub(plan.warmup_rounds);
                let tick = adapter.build_tick(TickInput {
                    task_id: &task_id,
                    elapsed: elapsed_global,
                    elapsed_in_stage: sample_elapsed.max(1),
                    stage_total: plan.stage_sample_rounds.max(1),
                    concurrency: *concurrency,
                    stage_index: stage_number,
                    stage_count: plan.stages.len() as i64,
                });
                final_tick = Some(tick.clone());
                self.persistence.insert_tick(&tick).await?;
                self.publisher.metrics_tick(tick.clone());

                if elapsed_in_stage > plan.warmup_rounds {
                    final_tick = Some(tick);
                }
            }

            if let Some(tick) = final_tick.clone() {
                let should_stop = self
                    .finish_stage(&plan, &task_id, stage_number, *concurrency, &tick)
                    .await?;

                if should_stop && plan.is_staircase {
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

    async fn emit_initial_tick(
        &self,
        adapter: &BenchmarkAdapter,
        task_id: &str,
        plan: &BenchmarkPlan,
        stage_number: i64,
        concurrency: i64,
        elapsed_global: i64,
        final_tick: &mut Option<MetricsTick>,
    ) -> anyhow::Result<()> {
        let tick = adapter.build_tick(TickInput {
            task_id,
            elapsed: elapsed_global,
            elapsed_in_stage: 1,
            stage_total: plan.stage_sample_rounds.max(1),
            concurrency,
            stage_index: stage_number,
            stage_count: plan.stages.len() as i64,
        });
        *final_tick = Some(tick.clone());
        self.persistence.insert_tick(&tick).await?;
        self.publisher.metrics_tick(tick);
        Ok(())
    }

    async fn wait_or_stop(&mut self, task_id: &str) -> anyhow::Result<bool> {
        tokio::select! {
            changed = self.stop_rx.changed() => {
                changed?;
                if *self.stop_rx.borrow() {
                    self.persistence
                        .finish_task(task_id, "cancelled", 0.0, 0, 0.0)
                        .await?;
                    return Ok(true);
                }
            }
            _ = sleep(Duration::from_secs(1)) => {}
        }

        Ok(false)
    }

    async fn finish_stage(
        &self,
        plan: &BenchmarkPlan,
        task_id: &str,
        stage_number: i64,
        concurrency: i64,
        tick: &MetricsTick,
    ) -> anyhow::Result<bool> {
        let should_stop =
            tick.latency_ms > plan.sla_p95_ms || tick.success_rate < plan.min_success_rate;
        let stop_reason = if should_stop {
            Some(format!(
                "P95 {}ms / 成功率 {:.2}% 未达到 SLA（P95 <= {}ms，成功率 >= {:.2}%）",
                tick.latency_ms, tick.success_rate, plan.sla_p95_ms, plan.min_success_rate
            ))
        } else {
            None
        };
        let sample = StageSample::from_tick_with_evidence(
            stage_number,
            concurrency,
            tick,
            plan.stage_sample_rounds,
            plan.warmup_rounds,
            !should_stop,
            stop_reason,
        );
        self.persistence.insert_stage(&sample).await?;

        self.publisher.stage_changed(StageChangedEvent {
            task_id: task_id.to_string(),
            stage: if should_stop {
                "threshold_reached".to_string()
            } else {
                "stage_completed".to_string()
            },
            message: if should_stop {
                if plan.should_stop_on_sla_failure() {
                    messages::threshold_reached_and_stop(
                        stage_number,
                        tick.latency_ms,
                        tick.success_rate,
                    )
                } else {
                    messages::threshold_reached_and_continue(
                        stage_number,
                        tick.latency_ms,
                        tick.success_rate,
                    )
                }
            } else {
                messages::stage_completed(
                    stage_number,
                    tick.qps,
                    tick.latency_ms,
                    tick.success_rate,
                )
            },
            stage_index: Some(stage_number),
            stage_total: Some(plan.stages.len() as i64),
            concurrency: Some(concurrency),
        });

        Ok(should_stop && plan.should_stop_on_sla_failure())
    }
}
