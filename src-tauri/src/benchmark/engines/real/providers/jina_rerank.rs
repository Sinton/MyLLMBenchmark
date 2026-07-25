use crate::domain::workload::WorkloadConfig;

pub(crate) fn rerank_body(
    model: &str,
    query: String,
    documents: Vec<String>,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "query": query,
        "documents": documents,
        "top_n": workload.top_k.max(1)
    })
}

pub(crate) fn diagnostic_query() -> String {
    "MyLLMBenchmark 如何评估大模型接口容量？".to_string()
}

pub(crate) fn diagnostic_documents() -> Vec<String> {
    vec![
        "通过阶梯加压、延迟、成功率和吞吐指标评估容量。".to_string(),
        "只观察页面颜色不能证明接口压测能力。".to_string(),
        "报告应包含请求数、错误分布和上线建议。".to_string(),
    ]
}
