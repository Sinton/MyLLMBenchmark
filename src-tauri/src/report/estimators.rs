use crate::domain::model_type::ModelType;
use crate::domain::workload::{default_workload_config, json_i64};
use crate::models::{MetricsTick, ReportStageSummary, ReportSummary};
use crate::report::formatting::round2;

pub fn estimate_stages_from_summary(
    summary: &ReportSummary,
    model_type: &str,
) -> Vec<ReportStageSummary> {
    let stable = summary
        .max_stable_concurrency
        .max(summary.recommended_concurrency)
        .max(1);
    [
        (
            1,
            (summary.recommended_concurrency as f64 * 0.5).round() as i64,
            0.58,
            "stable",
        ),
        (2, summary.recommended_concurrency.max(1), 0.82, "stable"),
        (3, stable, 1.0, "watch"),
        (4, (stable as f64 * 1.35).ceil() as i64, 1.46, "failed"),
    ]
    .into_iter()
    .map(|(stage_index, concurrency, factor, status)| {
        let p95 = (summary.p95_latency_ms as f64 * factor) as i64;
        let qps = estimate_qps(concurrency, p95);
        let ttft = estimate_ttft(model_type, p95);
        let tps = estimate_tps(model_type, concurrency, factor);
        let (input_tokens, output_tokens, total_tokens) =
            token_metrics_for(model_type, qps, tps, concurrency);
        let workload = default_workload_config(model_type);
        let batch_size = json_i64(&workload, "batch_size");
        let text_count = match ModelType::normalize(model_type) {
            ModelType::Embedding => json_i64(&workload, "text_count_per_request"),
            _ => 0,
        };
        let documents_per_query = json_i64(&workload, "documents_per_query");
        let pair_count = if ModelType::normalize(model_type) == ModelType::Rerank {
            (qps * documents_per_query as f64).round() as i64
        } else {
            0
        };
        let image_count = json_i64(&workload, "image_count");
        ReportStageSummary {
            stage_index,
            concurrency,
            sample_rounds: 0,
            warmup_rounds: 0,
            request_count: 0,
            success_count: 0,
            failure_count: 0,
            qps,
            p95_latency_ms: p95,
            ttft_ms: ttft,
            tps,
            success_rate: round2(match status {
                "stable" => (summary.success_rate + 0.5).min(99.99),
                "watch" => summary.success_rate,
                _ => (summary.success_rate - 2.2).max(90.0),
            }),
            error_rate: round2(100.0 - summary.success_rate),
            input_tokens,
            output_tokens,
            total_tokens,
            batch_size,
            text_count,
            documents_per_query,
            pair_count,
            image_count,
            sla_passed: status != "failed",
            stop_reason: None,
            status: status.to_string(),
        }
    })
    .collect()
}

pub fn estimate_ticks_from_summary(summary: &ReportSummary, model_type: &str) -> Vec<MetricsTick> {
    (1..=12)
        .map(|elapsed| {
            let progress = elapsed as f64 / 12.0;
            let concurrency = if elapsed < 4 {
                summary.recommended_concurrency.max(1)
            } else if elapsed < 9 {
                summary.max_stable_concurrency.max(1)
            } else {
                ((summary.max_stable_concurrency as f64) * 1.2).ceil() as i64
            };
            let latency_ms = (summary.p95_latency_ms as f64 * (0.72 + progress * 0.42)) as i64;
            let qps = estimate_qps(concurrency, latency_ms);
            let ttft_ms = estimate_ttft(model_type, latency_ms);
            let tps = estimate_tps(model_type, concurrency, 0.8 + progress);
            let (input_tokens, output_tokens, total_tokens) =
                token_metrics_for(model_type, qps, tps, concurrency);
            let workload = default_workload_config(model_type);
            let documents_per_query = json_i64(&workload, "documents_per_query");
            let pair_count = if ModelType::normalize(model_type) == ModelType::Rerank {
                (qps * documents_per_query as f64).round() as i64
            } else {
                0
            };
            MetricsTick {
                task_id: summary.task_id.clone(),
                elapsed_seconds: elapsed,
                qps,
                latency_ms,
                ttft_ms,
                tps,
                success_rate: round2((summary.success_rate + 0.6 - progress * 0.9).max(90.0)),
                errors: if progress > 0.82 { 2 } else { 0 },
                in_flight: concurrency,
                request_count: concurrency,
                success_count: (concurrency - if progress > 0.82 { 2 } else { 0 }).max(0),
                failure_count: if progress > 0.82 { 2 } else { 0 },
                input_tokens,
                output_tokens,
                total_tokens,
                batch_size: json_i64(&workload, "batch_size"),
                text_count: if ModelType::normalize(model_type) == ModelType::Embedding {
                    (qps * json_i64(&workload, "text_count_per_request") as f64).round() as i64
                } else {
                    0
                },
                documents_per_query,
                pair_count,
                image_count: json_i64(&workload, "image_count"),
            }
        })
        .collect()
}

pub(crate) fn stage_has_missing_llm_metrics(stage: &ReportStageSummary, model_type: &str) -> bool {
    let model_type = ModelType::normalize(model_type);
    let missing_tokens = stage.total_tokens == 0;
    let missing_throughput = stage.tps == 0.0;
    let missing_ttft = model_type.has_ttft() && stage.ttft_ms == 0;
    missing_tokens || missing_throughput || missing_ttft
}

pub(crate) fn hydrate_stage_metrics(
    stages: Vec<ReportStageSummary>,
    summary: &ReportSummary,
    model_type: &str,
) -> Vec<ReportStageSummary> {
    stages
        .into_iter()
        .map(|mut stage| {
            let factor = if summary.p95_latency_ms > 0 {
                (stage.p95_latency_ms as f64 / summary.p95_latency_ms as f64).max(0.5)
            } else {
                1.0
            };
            if stage.ttft_ms == 0 && ModelType::normalize(model_type).has_ttft() {
                stage.ttft_ms = estimate_ttft(model_type, stage.p95_latency_ms);
            }
            if stage.tps == 0.0 {
                stage.tps = estimate_tps(model_type, stage.concurrency, factor);
            }
            if stage.total_tokens == 0 {
                let (input_tokens, output_tokens, total_tokens) =
                    token_metrics_for(model_type, stage.qps, stage.tps, stage.concurrency);
                stage.input_tokens = input_tokens;
                stage.output_tokens = output_tokens;
                stage.total_tokens = total_tokens;
            }
            hydrate_stage_workload_fields(&mut stage, model_type);
            hydrate_stage_evidence_fields(&mut stage);
            stage
        })
        .collect()
}

fn hydrate_stage_evidence_fields(stage: &mut ReportStageSummary) {
    if stage.sample_rounds == 0 {
        stage.sample_rounds = 1;
    }
    if stage.request_count == 0 {
        stage.request_count = stage.concurrency.max(1) * stage.sample_rounds.max(1);
    }
    if stage.success_count == 0 && stage.failure_count == 0 {
        stage.failure_count =
            ((stage.request_count as f64) * (stage.error_rate / 100.0)).round() as i64;
        stage.success_count = (stage.request_count - stage.failure_count).max(0);
    }
}

fn hydrate_stage_workload_fields(stage: &mut ReportStageSummary, model_type: &str) {
    let model_type_kind = ModelType::normalize(model_type);
    let workload = default_workload_config(model_type);
    if stage.batch_size == 0 {
        stage.batch_size = json_i64(&workload, "batch_size");
    }
    if stage.text_count == 0 && model_type_kind == ModelType::Embedding {
        stage.text_count =
            (stage.qps * json_i64(&workload, "text_count_per_request") as f64).round() as i64;
    }
    if stage.documents_per_query == 0 {
        stage.documents_per_query = json_i64(&workload, "documents_per_query");
    }
    if stage.pair_count == 0 && model_type_kind == ModelType::Rerank {
        stage.pair_count = (stage.qps * stage.documents_per_query as f64).round() as i64;
    }
    if stage.image_count == 0 {
        stage.image_count = json_i64(&workload, "image_count");
    }
}

fn estimate_qps(concurrency: i64, p95_ms: i64) -> f64 {
    round2((concurrency as f64 / (p95_ms as f64 / 1000.0).max(0.8)) * 0.86)
}

fn estimate_ttft(model_type: &str, p95_ms: i64) -> i64 {
    match ModelType::normalize(model_type) {
        ModelType::Embedding | ModelType::Rerank => 0,
        ModelType::Multimodal => (p95_ms as f64 * 0.42) as i64,
        ModelType::TextGeneration => (p95_ms as f64 * 0.32) as i64,
    }
}

fn estimate_tps(model_type: &str, concurrency: i64, factor: f64) -> f64 {
    let pressure = concurrency as f64 / 32.0;
    round2(match ModelType::normalize(model_type) {
        ModelType::Embedding => 1400.0 / factor.max(0.6),
        ModelType::Rerank => 280.0 / factor.max(0.6),
        ModelType::Multimodal => (44.0 - pressure * 2.8).max(10.0),
        ModelType::TextGeneration => (62.0 - pressure * 3.5 - factor * 2.0).max(12.0),
    })
}

fn token_metrics_for(model_type: &str, qps: f64, tps: f64, concurrency: i64) -> (i64, i64, i64) {
    let pressure = (concurrency as f64 / 32.0).max(0.5);
    match ModelType::normalize(model_type) {
        ModelType::Embedding => {
            let input = (qps * 180.0 * pressure).round() as i64;
            (input, 0, input)
        }
        ModelType::Rerank => {
            let input = (qps * 760.0 * pressure).round() as i64;
            let output = (qps * 8.0).round() as i64;
            (input, output, input + output)
        }
        ModelType::Multimodal => {
            let input = (qps * 420.0 * pressure).round() as i64;
            let output = (tps * 0.75).round() as i64;
            (input, output, input + output)
        }
        ModelType::TextGeneration => {
            let input = (qps * 320.0).round() as i64;
            let output = tps.round() as i64;
            (input, output, input + output)
        }
    }
}
