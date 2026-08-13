use crate::domain::workload::WorkloadConfig;

use super::super::helpers::VisionSample;

pub(crate) fn response_body(
    model: &str,
    prompt: &str,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "input": prompt,
        "max_output_tokens": workload.max_output_tokens.max(1),
        "temperature": workload.temperature
    })
}

pub(crate) fn streaming_response_body(
    model: &str,
    prompt: &str,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    let mut body = response_body(model, prompt, workload);
    body["stream"] = serde_json::json!(true);
    body
}

pub(crate) fn vision_response_body(
    model: &str,
    sample: &VisionSample,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    let mut content = vec![serde_json::json!({
        "type": "input_text",
        "text": sample.prompt
    })];
    for image_url in &sample.image_urls {
        content.push(serde_json::json!({
            "type": "input_image",
            "image_url": image_url
        }));
    }
    serde_json::json!({
        "model": model,
        "input": [{"role": "user", "content": content}],
        "max_output_tokens": workload.max_output_tokens.max(1),
        "temperature": workload.temperature
    })
}

pub(crate) fn streaming_vision_response_body(
    model: &str,
    sample: &VisionSample,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    let mut body = vision_response_body(model, sample, workload);
    body["stream"] = serde_json::json!(true);
    body
}

pub(crate) fn extract_output_text(payload: &serde_json::Value) -> String {
    if let Some(text) = payload.get("output_text").and_then(|value| value.as_str()) {
        return text.to_string();
    }

    payload
        .get("output")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .flat_map(|item| {
            item.get("content")
                .and_then(|value| value.as_array())
                .into_iter()
                .flatten()
        })
        .filter_map(|content| {
            content
                .get("text")
                .or_else(|| content.get("output_text"))
                .and_then(|value| value.as_str())
        })
        .collect::<Vec<_>>()
        .join("")
}

pub(crate) fn diagnostic_prompt() -> &'static str {
    "请用一句中文回复：MyLLMBenchmark Responses 诊断成功。"
}
