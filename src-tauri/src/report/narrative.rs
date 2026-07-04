use crate::domain::model_type::ModelType;
use crate::models::ReportSummary;
use crate::report::formatting::trim_float;

pub(crate) fn capacity_conclusion(
    summary: &ReportSummary,
    sla_p95_ms: i64,
    min_success_rate: f64,
) -> String {
    format!(
        "{} 推荐生产并发 {}，最大稳定并发 {}；SLA 条件为 P95 <= {}ms 且成功率 >= {}%。",
        summary.model_name,
        summary.recommended_concurrency,
        summary.max_stable_concurrency,
        sla_p95_ms,
        trim_float(min_success_rate)
    )
}

pub(crate) fn verdict_for(
    p95: i64,
    success_rate: f64,
    sla_p95_ms: i64,
    min_success_rate: f64,
) -> (String, String) {
    if p95 > sla_p95_ms || success_rate < min_success_rate - 2.0 {
        ("fail".to_string(), "不建议上线".to_string())
    } else if p95 > (sla_p95_ms as f64 * 0.75) as i64 || success_rate < min_success_rate {
        ("watch".to_string(), "需关注".to_string())
    } else {
        ("pass".to_string(), "可上线".to_string())
    }
}

pub(crate) fn bottleneck_for(
    model_type: &str,
    p95: i64,
    ttft_ms: i64,
    tps: f64,
    success_rate: f64,
) -> String {
    if success_rate < 99.0 {
        return "成功率低于 SLA，优先关注超时、限流和服务端错误。".to_string();
    }

    match ModelType::normalize(model_type) {
        ModelType::Embedding => {
            if p95 > 1200 {
                "批量向量化延迟偏高，瓶颈可能来自 batch size、输入长度或队列调度。".to_string()
            } else {
                "向量吞吐保持稳定，当前瓶颈不明显，可继续通过 batch size 做容量探索。".to_string()
            }
        }
        ModelType::Rerank => {
            "重排序耗时主要受候选文档数量影响，建议按 query-doc 对规模分层复测。".to_string()
        }
        ModelType::Multimodal => {
            "视觉多模态请求受图片尺寸和编码开销影响，P95 与输入尺寸高度相关。".to_string()
        }
        ModelType::TextGeneration => {
            if ttft_ms > 1500 {
                "TTFT 上升明显，服务端开始排队，首 token 响应是主要瓶颈。".to_string()
            } else if tps < 25.0 {
                "输出 TPS 下降，推理资源趋于饱和，长输出请求会放大延迟。".to_string()
            } else if p95 > 3000 {
                "总延迟偏高，建议继续拆分 TTFT 与输出阶段耗时。".to_string()
            } else {
                "首 token 与输出吞吐均处于健康区间，当前容量水位可作为上线基线。".to_string()
            }
        }
    }
}

pub(crate) fn build_recommendations(
    model_type: &str,
    summary: &ReportSummary,
    sla_p95_ms: i64,
) -> Vec<String> {
    let mut items = vec![
        format!(
            "生产限流建议先配置为 {} 并发，观察 24 小时后再按 10%-15% 逐步上调。",
            summary.recommended_concurrency
        ),
        format!(
            "建议告警阈值：P95 > {}ms 或成功率 < 99%。",
            ((sla_p95_ms as f64) * 0.9).round() as i64
        ),
    ];

    items.push(match ModelType::normalize(model_type) {
        ModelType::Embedding => {
            "Embedding 场景优先调优 batch size，并为离线入库和在线召回分别建立压测任务。"
                .to_string()
        }
        ModelType::Rerank => "Rerank 场景必须限制候选文档数量，并按 TopK 档位复测。".to_string(),
        ModelType::Multimodal => {
            "Vision 场景建议限制图片尺寸和单请求图片数量，超大图先做预处理。".to_string()
        }
        ModelType::TextGeneration => {
            "文本生成场景建议开启 Streaming，并按输出长度分层评估容量。".to_string()
        }
    });
    items.push("真实接口接入后补充长稳压测，验证连续运行下的错误率和延迟漂移。".to_string());
    items
}
