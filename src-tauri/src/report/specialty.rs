use crate::domain::model_type::ModelType;
use crate::domain::workload::json_i64;
use crate::models::{ReportSpecialtyMetric, ReportSpecialtySection};
use crate::report::formatting::round2;

pub(crate) struct SpecialtyInput<'a> {
    pub model_type: &'a str,
    pub workload_config: &'a serde_json::Value,
    pub ttft_ms: i64,
    pub tps: f64,
    pub token_throughput: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub stable_qps: f64,
    pub p95_latency_ms: i64,
    pub success_rate: f64,
}

pub(crate) fn build_specialty_section(input: SpecialtyInput<'_>) -> ReportSpecialtySection {
    let SpecialtyInput {
        model_type,
        workload_config,
        ttft_ms,
        tps,
        token_throughput,
        input_tokens,
        output_tokens,
        stable_qps,
        p95_latency_ms,
        success_rate,
    } = input;

    match ModelType::normalize(model_type) {
        ModelType::Embedding => ReportSpecialtySection {
            title: "Embedding 向量化专项".to_string(),
            description: "关注批处理吞吐、输入 token/s 和 P95 延迟，适合知识库入库与召回链路容量评估。"
                .to_string(),
            metrics: vec![
                metric(
                    "向量吞吐",
                    stable_qps,
                    Some("req/s"),
                    "按当前 batch 配置估算的稳定请求吞吐",
                ),
                metric(
                    "Batch Size",
                    json_i64(workload_config, "batch_size"),
                    Some("条/批"),
                    "每个请求携带的文本条数",
                ),
                metric(
                    "Text/s",
                    (stable_qps * json_i64(workload_config, "text_count_per_request") as f64)
                        .round() as i64,
                    Some("条/s"),
                    "每秒处理的文本条数",
                ),
                metric(
                    "Input Token TPS",
                    input_tokens,
                    Some("token/s"),
                    "每秒处理的文本 token 数",
                ),
                metric(
                    "P95 延迟",
                    p95_latency_ms,
                    Some("ms"),
                    "批量嵌入请求的尾延迟",
                ),
                metric(
                    "成功率",
                    success_rate,
                    Some("%"),
                    "向量化接口成功完成比例",
                ),
            ],
            guidance: vec![
                "优先按 batch size 分组复测，观察吞吐是否线性增长。".to_string(),
                "知识库入库场景建议额外做长稳测试，检查队列堆积。".to_string(),
            ],
        },
        ModelType::Rerank => ReportSpecialtySection {
            title: "Rerank 重排序专项".to_string(),
            description: "关注 query-doc 对规模、排序吞吐和 P95 延迟，适合 RAG 精排链路容量评估。"
                .to_string(),
            metrics: vec![
                metric(
                    "排序吞吐",
                    stable_qps,
                    Some("query/s"),
                    "稳定水位下每秒 query 数",
                ),
                metric(
                    "Docs/Query",
                    json_i64(workload_config, "documents_per_query"),
                    Some("docs"),
                    "每个 query 的候选文档数量",
                ),
                metric(
                    "Pair/s",
                    (stable_qps * json_i64(workload_config, "documents_per_query") as f64)
                        .round() as i64,
                    Some("pair/s"),
                    "每秒处理的 query-doc 对",
                ),
                metric(
                    "TopK",
                    json_i64(workload_config, "top_k"),
                    Some("docs"),
                    "返回的候选结果数量",
                ),
                metric(
                    "候选处理量",
                    input_tokens,
                    Some("token/s"),
                    "候选文档输入 token 吞吐",
                ),
                metric(
                    "P95 延迟",
                    p95_latency_ms,
                    Some("ms"),
                    "精排链路尾延迟",
                ),
                metric(
                    "成功率",
                    success_rate,
                    Some("%"),
                    "排序请求成功完成比例",
                ),
            ],
            guidance: vec![
                "按 TopK 和候选文档数量拆分数据集，例如 10、30、50、100。".to_string(),
                "线上应限制单次 rerank 的候选文档数量，避免精排拖慢问答链路。"
                    .to_string(),
            ],
        },
        ModelType::Multimodal => ReportSpecialtySection {
            title: "Vision / 多模态专项".to_string(),
            description: "关注图片尺寸、图文输入编码开销、TTFT 和尾延迟，适合视觉识别与图文问答容量评估。"
                .to_string(),
            metrics: vec![
                metric(
                    "图文 P95",
                    p95_latency_ms,
                    Some("ms"),
                    "图文请求整体尾延迟",
                ),
                metric(
                    "Image Count",
                    json_i64(workload_config, "image_count"),
                    Some("张/请求"),
                    "每个请求包含的图片数量",
                ),
                metric(
                    "Image Profile",
                    workload_config
                        .get("image_profile")
                        .and_then(|value| value.as_str())
                        .unwrap_or("medium"),
                    None,
                    "图片尺寸档位",
                ),
                metric(
                    "TTFT",
                    ttft_ms,
                    Some("ms"),
                    "首 token 或首段响应等待时间",
                ),
                metric(
                    "Token Throughput",
                    token_throughput,
                    Some("token/s"),
                    "图文输入与输出综合 token 吞吐",
                ),
                metric(
                    "成功率",
                    success_rate,
                    Some("%"),
                    "视觉请求成功完成比例",
                ),
            ],
            guidance: vec![
                "按图片分辨率分层复测，避免用小图样本高估生产容量。".to_string(),
                "建议限制单请求图片数量和尺寸，并对超大图做预处理。".to_string(),
            ],
        },
        ModelType::TextGeneration => ReportSpecialtySection {
            title: "文本生成专项".to_string(),
            description: "关注 TTFT、输出 TPS、TPOT 和 token 吞吐，适合 Chat、Reasoning、Streaming 模型容量评估。"
                .to_string(),
            metrics: vec![
                metric(
                    "TTFT",
                    ttft_ms,
                    Some("ms"),
                    "首 token 延迟，反映排队和首段推理等待",
                ),
                metric(
                    "Output TPS",
                    tps,
                    Some("token/s"),
                    "稳定水位下的输出 token 速度",
                ),
                metric(
                    "TPOT",
                    if tps > 0.0 {
                        round2(1000.0 / tps)
                    } else {
                        0.0
                    },
                    Some("ms/token"),
                    "平均每个输出 token 的生成耗时",
                ),
                metric(
                    "Max Output",
                    json_i64(workload_config, "max_output_tokens"),
                    Some("token"),
                    "单次请求最大输出长度",
                ),
                metric(
                    "Token Throughput",
                    token_throughput,
                    Some("token/s"),
                    "输入与输出合计 token 吞吐",
                ),
                metric(
                    "Input Tokens",
                    input_tokens,
                    Some("token/s"),
                    "每秒输入 token 处理量",
                ),
                metric(
                    "Output Tokens",
                    output_tokens,
                    Some("token/s"),
                    "每秒输出 token 生成量",
                ),
            ],
            guidance: vec![
                "Streaming 场景优先看 TTFT 和 TPOT，不要只看总延迟。".to_string(),
                "建议生产限制 Prompt < 300 Token、Output < 512 Token，再针对长上下文单独复测。"
                    .to_string(),
            ],
        },
    }
}

fn metric<T: serde::Serialize>(
    label: &str,
    value: T,
    unit: Option<&str>,
    hint: &str,
) -> ReportSpecialtyMetric {
    ReportSpecialtyMetric {
        label: label.to_string(),
        value: serde_json::to_value(value).unwrap_or(serde_json::Value::Null),
        unit: unit.map(|value| value.to_string()),
        hint: hint.to_string(),
    }
}
