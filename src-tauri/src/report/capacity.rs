use crate::models::ReportStageSummary;

#[derive(Debug, Clone)]
pub struct CapacityRecommendation {
    pub recommended_concurrency: i64,
    pub max_stable_concurrency: i64,
    pub p95_latency_ms: i64,
    pub success_rate: f64,
    pub stable_qps: f64,
}

pub fn stage_status(
    p95_latency_ms: i64,
    success_rate: f64,
    sla_p95_ms: i64,
    min_success_rate: f64,
) -> String {
    if p95_latency_ms <= sla_p95_ms && success_rate >= min_success_rate {
        "stable"
    } else if p95_latency_ms <= (sla_p95_ms as f64 * 1.25) as i64
        && success_rate >= min_success_rate - 1.0
    {
        "watch"
    } else {
        "failed"
    }
    .to_string()
}

pub fn capacity_from_stages(
    stages: &[ReportStageSummary],
    fallback_concurrency: i64,
    fallback_p95_latency_ms: i64,
    fallback_success_rate: f64,
) -> CapacityRecommendation {
    let stable_stage = stages
        .iter()
        .filter(|stage| stage.status == "stable")
        .max_by_key(|stage| stage.concurrency)
        .or_else(|| stages.first());
    let max_stable = stable_stage
        .map(|stage| stage.concurrency)
        .unwrap_or(fallback_concurrency)
        .max(1);
    let recommended = (max_stable * 7 / 10).max(1);
    let p95 = stable_stage
        .map(|stage| stage.p95_latency_ms)
        .unwrap_or(fallback_p95_latency_ms.max(1));
    let success_rate = stable_stage
        .map(|stage| stage.success_rate)
        .unwrap_or(fallback_success_rate);
    let stable_qps = stable_stage
        .map(|stage| stage.qps)
        .unwrap_or(recommended as f64);

    CapacityRecommendation {
        recommended_concurrency: recommended,
        max_stable_concurrency: max_stable,
        p95_latency_ms: p95,
        success_rate,
        stable_qps,
    }
}

pub fn build_recommendation_text(model_name: &str, capacity: &CapacityRecommendation) -> String {
    format!(
        "在当前模拟数据集和 SLA 条件下，{} 建议生产并发为 {}，最大稳定并发为 {}；超过稳定水位后需重点观察 TTFT、P95 和错误率。",
        model_name, capacity.recommended_concurrency, capacity.max_stable_concurrency
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_status_respects_sla() {
        assert_eq!(stage_status(3000, 99.5, 5000, 99.0), "stable");
        assert_eq!(stage_status(5800, 98.5, 5000, 99.0), "watch");
        assert_eq!(stage_status(7000, 96.0, 5000, 99.0), "failed");
    }

    #[test]
    fn capacity_prefers_highest_stable_stage() {
        let stages = vec![
            ReportStageSummary {
                stage_index: 1,
                concurrency: 16,
                sample_rounds: 3,
                warmup_rounds: 1,
                request_count: 48,
                success_count: 48,
                failure_count: 0,
                qps: 10.0,
                p95_latency_ms: 1200,
                ttft_ms: 300,
                tps: 50.0,
                success_rate: 99.9,
                error_rate: 0.1,
                input_tokens: 100,
                output_tokens: 50,
                total_tokens: 150,
                batch_size: 0,
                text_count: 0,
                documents_per_query: 0,
                pair_count: 0,
                image_count: 0,
                sla_passed: true,
                stop_reason: None,
                status: "stable".to_string(),
            },
            ReportStageSummary {
                stage_index: 2,
                concurrency: 32,
                sample_rounds: 3,
                warmup_rounds: 1,
                request_count: 96,
                success_count: 96,
                failure_count: 0,
                qps: 18.0,
                p95_latency_ms: 2100,
                ttft_ms: 500,
                tps: 42.0,
                success_rate: 99.6,
                error_rate: 0.4,
                input_tokens: 180,
                output_tokens: 42,
                total_tokens: 222,
                batch_size: 0,
                text_count: 0,
                documents_per_query: 0,
                pair_count: 0,
                image_count: 0,
                sla_passed: true,
                stop_reason: None,
                status: "stable".to_string(),
            },
        ];
        let capacity = capacity_from_stages(&stages, 8, 1000, 99.0);
        assert_eq!(capacity.max_stable_concurrency, 32);
        assert_eq!(capacity.recommended_concurrency, 22);
        assert_eq!(capacity.p95_latency_ms, 2100);
    }
}
