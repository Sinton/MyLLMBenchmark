use crate::domain::workload::WorkloadConfig;

pub(crate) fn completion_body(
    model: &str,
    prompt: &str,
    workload: &WorkloadConfig,
    stream: bool,
) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": prompt}],
        "stream": stream,
        "max_tokens": workload.max_output_tokens.max(1),
        "temperature": workload.temperature
    })
}

pub(crate) fn streaming_completion_body(
    model: &str,
    prompt: &str,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    let mut body = completion_body(model, prompt, workload, true);
    body["stream_options"] = serde_json::json!({"include_usage": true});
    body
}

pub(crate) fn diagnostic_prompt() -> &'static str {
    "请用一句中文回复：MyLLMBenchmark 连接诊断成功。"
}
