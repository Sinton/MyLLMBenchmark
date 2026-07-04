use crate::models::BenchmarkStartInput;

pub const SLA_STOP_CONTINUE: &str = "continue_full_staircase";
pub const SLA_STOP_ON_FAILURE: &str = "stop_on_failure";
const DEFAULT_STAGE_SAMPLE_ROUNDS: i64 = 4;
const DEFAULT_WARMUP_ROUNDS: i64 = 1;
const DEFAULT_REQUEST_TIMEOUT_SECONDS: i64 = 120;

#[derive(Debug, Clone)]
pub struct BenchmarkPlan {
    pub is_staircase: bool,
    pub stages: Vec<i64>,
    pub stage_sample_rounds: i64,
    pub stage_duration_seconds: i64,
    pub warmup_rounds: i64,
    pub warmup_seconds: i64,
    pub request_timeout_seconds: i64,
    pub sla_p95_ms: i64,
    pub min_success_rate: f64,
    pub sla_stop_policy: String,
}

impl BenchmarkPlan {
    pub fn from_input(input: &BenchmarkStartInput) -> Self {
        let is_staircase = input.mode == "阶梯加压";
        let request_timeout_seconds = input
            .request_timeout_seconds
            .unwrap_or(DEFAULT_REQUEST_TIMEOUT_SECONDS)
            .clamp(5, 600);
        let sla_stop_policy = normalize_sla_stop_policy(input.sla_stop_policy.as_deref());
        if !is_staircase {
            let stage_sample_rounds = input
                .stage_sample_rounds
                .or(input.stage_duration_seconds)
                .unwrap_or(input.duration_seconds)
                .clamp(1, 300);
            return Self {
                is_staircase,
                stages: vec![input.concurrency.max(1)],
                stage_sample_rounds,
                stage_duration_seconds: stage_sample_rounds,
                warmup_rounds: 0,
                warmup_seconds: 0,
                request_timeout_seconds,
                sla_p95_ms: input.sla_p95_ms.unwrap_or(5000),
                min_success_rate: input.min_success_rate.unwrap_or(99.0),
                sla_stop_policy,
            };
        }

        let start = input.start_concurrency.unwrap_or(1).max(1);
        let end = input
            .end_concurrency
            .unwrap_or(input.concurrency.max(start))
            .max(start);
        let strategy = input.step_strategy.as_deref().unwrap_or("double");
        let step_value = input
            .step_value
            .unwrap_or(if strategy == "linear" { 8 } else { 2 })
            .max(1);
        let mut stages = Vec::new();
        let mut current = start;

        while current <= end && stages.len() < 16 {
            stages.push(current);
            current = if strategy == "linear" {
                current + step_value
            } else {
                let next = current * step_value.max(2);
                if next == current {
                    current + 1
                } else {
                    next
                }
            };
        }

        if !stages.contains(&end) {
            stages.push(end);
        }

        let stage_sample_rounds = input
            .stage_sample_rounds
            .or(input.stage_duration_seconds)
            .unwrap_or(DEFAULT_STAGE_SAMPLE_ROUNDS)
            .clamp(1, 300);
        let warmup_rounds = input
            .warmup_rounds
            .or(input.warmup_seconds)
            .unwrap_or(DEFAULT_WARMUP_ROUNDS)
            .clamp(0, 120);

        Self {
            is_staircase,
            stages,
            stage_sample_rounds,
            stage_duration_seconds: stage_sample_rounds,
            warmup_rounds,
            warmup_seconds: warmup_rounds,
            request_timeout_seconds,
            sla_p95_ms: input.sla_p95_ms.unwrap_or(5000),
            min_success_rate: input.min_success_rate.unwrap_or(99.0),
            sla_stop_policy,
        }
    }

    pub fn should_stop_on_sla_failure(&self) -> bool {
        self.sla_stop_policy == SLA_STOP_ON_FAILURE
    }
}

fn normalize_sla_stop_policy(value: Option<&str>) -> String {
    match value {
        Some(SLA_STOP_ON_FAILURE) => SLA_STOP_ON_FAILURE.to_string(),
        _ => SLA_STOP_CONTINUE.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> BenchmarkStartInput {
        BenchmarkStartInput {
            provider_id: "provider".to_string(),
            model_id: None,
            dataset_id: "dataset".to_string(),
            mode: "阶梯加压".to_string(),
            concurrency: 64,
            duration_seconds: 30,
            start_concurrency: Some(1),
            end_concurrency: Some(64),
            step_strategy: Some("double".to_string()),
            step_value: Some(2),
            stage_sample_rounds: Some(4),
            stage_duration_seconds: Some(4),
            warmup_rounds: Some(1),
            warmup_seconds: Some(1),
            request_timeout_seconds: Some(120),
            sla_p95_ms: Some(5000),
            min_success_rate: Some(99.0),
            sla_stop_policy: Some(SLA_STOP_CONTINUE.to_string()),
            workload_config: None,
        }
    }

    #[test]
    fn builds_doubling_staircase() {
        let plan = BenchmarkPlan::from_input(&base_input());
        assert_eq!(plan.stages, vec![1, 2, 4, 8, 16, 32, 64]);
        assert!(plan.is_staircase);
        assert_eq!(plan.stage_sample_rounds, 4);
        assert!(!plan.should_stop_on_sla_failure());
    }

    #[test]
    fn old_duration_fields_still_work() {
        let mut input = base_input();
        input.stage_sample_rounds = None;
        input.warmup_rounds = None;
        input.stage_duration_seconds = Some(6);
        input.warmup_seconds = Some(2);
        let plan = BenchmarkPlan::from_input(&input);
        assert_eq!(plan.stage_sample_rounds, 6);
        assert_eq!(plan.warmup_rounds, 2);
    }

    #[test]
    fn can_enable_protective_sla_stop() {
        let mut input = base_input();
        input.sla_stop_policy = Some(SLA_STOP_ON_FAILURE.to_string());
        let plan = BenchmarkPlan::from_input(&input);
        assert!(plan.should_stop_on_sla_failure());
    }
}
