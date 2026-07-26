use crate::models::{DatasetSample, DiscoveredModel};
use std::time::Duration;

pub fn api_url(base_url: &str, path: &str) -> String {
    format!(
        "{}/{}",
        normalized_api_base_url(base_url),
        path.trim_start_matches('/')
    )
}

fn normalized_api_base_url(base_url: &str) -> String {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return trimmed.to_string();
    }

    let without_scheme = trimmed
        .split_once("://")
        .map(|(_, rest)| rest)
        .unwrap_or(trimmed);
    let path = without_scheme
        .split_once('/')
        .map(|(_, path)| path)
        .unwrap_or("");

    if path.is_empty() {
        format!("{trimmed}/v1")
    } else {
        trimmed.to_string()
    }
}

pub fn classify_model(model_name: &str) -> DiscoveredModel {
    let lower = model_name.to_ascii_lowercase();
    let model_type = if lower.contains("rerank") || lower.contains("reranker") {
        "reranker"
    } else if lower.contains("embedding")
        || lower.contains("embed")
        || lower.contains("bge")
        || lower.contains("e5")
    {
        "embedding"
    } else if lower.contains("vision")
        || lower.contains("qwen-vl")
        || lower.contains("-vl")
        || lower.contains("_vl")
        || lower.contains("llava")
    {
        "multimodal"
    } else {
        "text_generation"
    };

    let capabilities = match model_type {
        "embedding" => vec!["embedding".to_string()],
        "reranker" => vec!["rerank".to_string()],
        "multimodal" => vec!["streaming".to_string(), "image_input".to_string()],
        _ => vec!["streaming".to_string(), "chat".to_string()],
    };

    DiscoveredModel {
        name: model_name.to_string(),
        model_type: model_type.to_string(),
        capabilities,
        supports_streaming: matches!(model_type, "text_generation" | "multimodal"),
        recommended_concurrency: None,
    }
}

pub(crate) fn preview_text(text: &str) -> String {
    const MAX_CHARS: usize = 120;
    let mut preview = text.chars().take(MAX_CHARS).collect::<String>();
    if text.chars().count() > MAX_CHARS {
        preview.push('…');
    }
    preview
}

pub(crate) fn collect_embedding_inputs(
    samples: &[DatasetSample],
    start_index: usize,
    count: usize,
) -> Vec<String> {
    if samples.is_empty() {
        return Vec::new();
    }
    (0..count.max(1))
        .map(|offset| {
            samples[(start_index + offset) % samples.len()]
                .prompt
                .clone()
        })
        .collect()
}

pub(crate) fn collect_rerank_inputs(
    samples: &[DatasetSample],
    start_index: usize,
    documents_per_query: usize,
) -> (String, Vec<String>) {
    if samples.is_empty() {
        return ("".to_string(), Vec::new());
    }
    let query = samples[start_index % samples.len()].prompt.clone();
    let documents = (0..documents_per_query.max(1))
        .map(|offset| {
            samples[(start_index + offset + 1) % samples.len()]
                .prompt
                .clone()
        })
        .collect();
    (query, documents)
}

pub(crate) fn rerank_prompt_for_log(query: &str, documents: &[String]) -> String {
    let docs = documents
        .iter()
        .enumerate()
        .map(|(index, document)| format!("{}. {}", index + 1, document))
        .collect::<Vec<_>>()
        .join("\n");
    format!("Query:\n{query}\n\nDocuments:\n{docs}")
}

pub(crate) struct VisionSample {
    pub(crate) prompt: String,
    pub(crate) image_urls: Vec<String>,
}

pub(crate) fn parse_vision_sample(raw: &str, image_limit: i64) -> VisionSample {
    let limit = image_limit.clamp(1, 8) as usize;
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) {
        let prompt = value
            .get("prompt")
            .or_else(|| value.get("text"))
            .or_else(|| value.get("input"))
            .and_then(|item| item.as_str())
            .unwrap_or("请分析这张图片。")
            .to_string();
        let mut image_urls = Vec::new();
        if let Some(image_url) = value
            .get("image_url")
            .or_else(|| value.get("image"))
            .and_then(|item| item.as_str())
        {
            image_urls.push(image_url.to_string());
        }
        if let Some(items) = value
            .get("image_urls")
            .or_else(|| value.get("images"))
            .and_then(|item| item.as_array())
        {
            image_urls.extend(
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .map(ToString::to_string),
            );
        }
        image_urls.truncate(limit);
        return VisionSample { prompt, image_urls };
    }

    let trimmed = raw.trim();
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("data:image/")
    {
        return VisionSample {
            prompt: "请分析这张图片。".to_string(),
            image_urls: vec![trimmed.to_string()],
        };
    }

    VisionSample {
        prompt: trimmed.to_string(),
        image_urls: Vec::new(),
    }
}

pub(crate) fn duration_ms(duration: Duration) -> i64 {
    duration.as_millis().min(i64::MAX as u128) as i64
}

pub(crate) fn request_timeout(seconds: i64) -> Duration {
    Duration::from_secs(seconds.clamp(5, 600) as u64)
}
