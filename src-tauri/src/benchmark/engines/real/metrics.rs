use super::RequestOutcome;
use crate::domain::{model_type::ModelType, workload::WorkloadConfig};
use crate::models::MetricsTick;

pub(super) fn build_tick_from_results(
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

pub(super) fn aggregate_stage_tick(task_id: &str, ticks: Vec<MetricsTick>) -> Option<MetricsTick> {
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

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

fn round_i64(value: f64) -> i64 {
    value.round().max(0.0) as i64
}
