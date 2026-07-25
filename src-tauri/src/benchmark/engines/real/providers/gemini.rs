use crate::domain::workload::WorkloadConfig;

use super::super::VisionSample;

pub(crate) fn generate_content_body(prompt: &str, workload: &WorkloadConfig) -> serde_json::Value {
    serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{"text": prompt}]
        }],
        "generationConfig": {
            "maxOutputTokens": workload.max_output_tokens.max(1),
            "temperature": 0.7
        }
    })
}

pub(crate) fn vision_generate_content_body(
    sample: &VisionSample,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    let mut parts = vec![serde_json::json!({"text": sample.prompt})];
    for image_url in &sample.image_urls {
        parts.push(serde_json::json!({
            "fileData": {
                "fileUri": image_url
            }
        }));
    }
    serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": parts
        }],
        "generationConfig": {
            "maxOutputTokens": workload.max_output_tokens.max(1),
            "temperature": 0.2
        }
    })
}

pub(crate) fn extract_text(payload: &serde_json::Value) -> String {
    payload
        .get("candidates")
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .flat_map(|candidate| {
            candidate
                .get("content")
                .and_then(|content| content.get("parts"))
                .and_then(|value| value.as_array())
                .into_iter()
                .flatten()
        })
        .filter_map(|part| part.get("text").and_then(|value| value.as_str()))
        .collect::<Vec<_>>()
        .join("")
}

pub(crate) fn usage_from_value(value: &serde_json::Value) -> Option<(i64, i64, i64)> {
    let usage = value.get("usageMetadata")?;
    let input_tokens = usage
        .get("promptTokenCount")
        .and_then(|item| item.as_i64())
        .unwrap_or(0);
    let output_tokens = usage
        .get("candidatesTokenCount")
        .and_then(|item| item.as_i64())
        .unwrap_or(0);
    let total_tokens = usage
        .get("totalTokenCount")
        .and_then(|item| item.as_i64())
        .unwrap_or(input_tokens + output_tokens);
    Some((input_tokens, output_tokens, total_tokens))
}

pub(crate) fn diagnostic_prompt() -> &'static str {
    "请用一句中文回复：MyLLMBenchmark Gemini 诊断成功。"
}
