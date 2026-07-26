use crate::domain::workload::WorkloadConfig;

use super::super::helpers::VisionSample;

pub(crate) fn vision_completion_body(
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
            "type": "image_url",
            "image_url": {"url": image_url}
        }));
    }
    serde_json::json!({
        "model": model,
        "messages": [{"role": "user", "content": content}],
        "stream": false,
        "max_tokens": workload.max_output_tokens.max(1),
        "temperature": 0.2
    })
}
