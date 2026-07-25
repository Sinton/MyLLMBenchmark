use crate::domain::workload::WorkloadConfig;

use super::super::VisionSample;

pub(crate) fn messages_body(
    model: &str,
    prompt: &str,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "max_tokens": workload.max_output_tokens.max(1),
        "temperature": 0.7,
        "messages": [{"role": "user", "content": prompt}]
    })
}

pub(crate) fn vision_messages_body(
    model: &str,
    sample: &VisionSample,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    let mut content = vec![serde_json::json!({
        "type": "text",
        "text": sample.prompt
    })];
    for image_url in &sample.image_urls {
        content.push(serde_json::json!({
            "type": "image",
            "source": {
                "type": "url",
                "url": image_url
            }
        }));
    }
    serde_json::json!({
        "model": model,
        "max_tokens": workload.max_output_tokens.max(1),
        "temperature": 0.2,
        "messages": [{"role": "user", "content": content}]
    })
}

pub(crate) fn extract_text(payload: &serde_json::Value) -> String {
    payload
        .get("content")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("text").and_then(|value| value.as_str()))
        .collect::<Vec<_>>()
        .join("")
}

pub(crate) fn diagnostic_prompt() -> &'static str {
    "请用一句中文回复：MyLLMBenchmark Anthropic 诊断成功。"
}
