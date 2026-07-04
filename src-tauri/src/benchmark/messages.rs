pub fn runtime_failed(error: &anyhow::Error) -> String {
    format!("压测运行失败：{error}")
}

pub fn warmup_started(is_staircase: bool, stages: &[i64]) -> String {
    if is_staircase {
        format!(
            "阶梯加压开始：共 {} 个阶段，并发序列 {}",
            stages.len(),
            stages
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(" -> ")
        )
    } else {
        "开始预热并建立模拟请求窗口".to_string()
    }
}

pub fn stage_running(
    stage_number: i64,
    stage_total: usize,
    concurrency: i64,
    warmup_rounds: i64,
    sample_rounds: i64,
) -> String {
    format!(
        "阶段 {stage_number}/{stage_total}：并发 {concurrency}，预热 {warmup_rounds} 轮，请求采样 {sample_rounds} 轮"
    )
}

pub fn threshold_reached(stage_number: i64, p95_latency_ms: i64, success_rate: f64) -> String {
    format!("阶段 {stage_number} 未达 SLA：P95 {p95_latency_ms}ms / 成功率 {success_rate}%")
}

pub fn threshold_reached_and_continue(
    stage_number: i64,
    p95_latency_ms: i64,
    success_rate: f64,
) -> String {
    format!(
        "阶段 {stage_number} 未达 SLA：P95 {p95_latency_ms}ms / 成功率 {success_rate}%，按当前策略继续执行后续阶梯"
    )
}

pub fn threshold_reached_and_stop(
    stage_number: i64,
    p95_latency_ms: i64,
    success_rate: f64,
) -> String {
    format!(
        "阶段 {stage_number} 未达 SLA：P95 {p95_latency_ms}ms / 成功率 {success_rate}%，已触发保护性停止"
    )
}

pub fn stage_completed(
    stage_number: i64,
    qps: f64,
    p95_latency_ms: i64,
    success_rate: f64,
) -> String {
    format!("阶段 {stage_number} 完成：QPS {qps}，P95 {p95_latency_ms}ms，成功率 {success_rate}%")
}

#[cfg(test)]
mod tests {
    use super::{stage_completed, stage_running, warmup_started};

    fn assert_no_mojibake(message: &str) {
        for marker in ["锛", "鍚", "鎴", "妯", "�"] {
            assert!(
                !message.contains(marker),
                "message contains mojibake marker {marker}: {message}"
            );
        }
    }

    #[test]
    fn runtime_messages_are_readable_chinese() {
        let messages = [
            warmup_started(true, &[1, 2, 4]),
            stage_running(1, 3, 8, 2, 10),
            stage_completed(1, 12.5, 1800, 99.9),
        ];

        for message in messages {
            assert_no_mojibake(&message);
            assert!(message.contains("阶段") || message.contains("阶梯"));
        }
    }
}
