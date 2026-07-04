use crate::domain::model_type::{default_capabilities, ModelType};
use crate::models::ModelSummary;

#[derive(Debug, Clone, Copy)]
pub enum CatalogFlavor {
    Demo,
    Mock,
}

#[derive(Debug, Clone)]
pub struct ModelTemplate {
    pub name: String,
    pub model_type: String,
    pub capabilities: Vec<String>,
    pub supports_streaming: bool,
    pub recommended_concurrency: Option<i64>,
}

pub fn model_templates_for_interface(
    interface_type: &str,
    flavor: CatalogFlavor,
) -> Vec<ModelTemplate> {
    let suffix = match flavor {
        CatalogFlavor::Demo => "demo",
        CatalogFlavor::Mock => "mock",
    };
    let display_suffix = match flavor {
        CatalogFlavor::Demo => "Demo",
        CatalogFlavor::Mock => "Mock",
    };

    let specs = match interface_type {
        "Jina Rerank" => vec![(
            format!("jina-reranker-v2-base-multilingual-{suffix}"),
            ModelType::Rerank,
            false,
            Some(64),
            vec!["batch"],
        )],
        "Gemini" => vec![
            (
                format!("gemini-2.5-pro-{suffix}"),
                ModelType::TextGeneration,
                true,
                Some(24),
                vec!["streaming", "reasoning", "json_schema"],
            ),
            (
                format!("gemini-2.5-flash-vision-{suffix}"),
                ModelType::Multimodal,
                true,
                Some(16),
                vec!["streaming", "image_input"],
            ),
            (
                format!("text-embedding-004-{suffix}"),
                ModelType::Embedding,
                false,
                Some(96),
                vec!["batch"],
            ),
        ],
        "Anthropic" => vec![
            (
                format!("claude-sonnet-{suffix}"),
                ModelType::TextGeneration,
                true,
                Some(24),
                vec!["streaming", "reasoning", "tool_calling"],
            ),
            (
                format!("claude-vision-{suffix}"),
                ModelType::Multimodal,
                true,
                Some(16),
                vec!["streaming", "image_input"],
            ),
        ],
        "OpenAI-Response" => vec![
            (
                format!("gpt-4.1-responses-{suffix}"),
                ModelType::TextGeneration,
                true,
                Some(32),
                vec!["streaming", "reasoning", "tool_calling", "json_schema"],
            ),
            (
                format!("text-embedding-3-large-{suffix}"),
                ModelType::Embedding,
                false,
                Some(128),
                vec!["batch"],
            ),
        ],
        _ => vec![
            (
                format!("DeepSeek-R1-{display_suffix}"),
                ModelType::TextGeneration,
                true,
                Some(32),
                vec!["streaming", "reasoning"],
            ),
            (
                format!("BGE-M3-{display_suffix}"),
                ModelType::Embedding,
                false,
                Some(128),
                vec!["batch"],
            ),
            (
                format!("BCE-Reranker-{display_suffix}"),
                ModelType::Rerank,
                false,
                Some(64),
                vec!["batch"],
            ),
            (
                format!("Qwen-VL-{display_suffix}"),
                ModelType::Multimodal,
                true,
                Some(12),
                vec!["streaming", "image_input"],
            ),
        ],
    };

    specs
        .into_iter()
        .map(
            |(name, model_type, supports_streaming, recommended_concurrency, capabilities)| {
                ModelTemplate {
                    name,
                    model_type: model_type.as_str().to_string(),
                    capabilities: capabilities
                        .into_iter()
                        .map(|capability| capability.to_string())
                        .collect(),
                    supports_streaming,
                    recommended_concurrency,
                }
            },
        )
        .collect()
}

pub fn model_summaries_for_interface(
    provider_id: &str,
    interface_type: &str,
    flavor: CatalogFlavor,
) -> Vec<ModelSummary> {
    model_templates_for_interface(interface_type, flavor)
        .into_iter()
        .map(|template| ModelSummary {
            id: format!(
                "{}-{}",
                provider_id,
                template.name.to_lowercase().replace(' ', "-")
            ),
            provider_id: provider_id.to_string(),
            name: template.name,
            model_type: template.model_type.clone(),
            capabilities: if template.capabilities.is_empty() {
                default_capabilities(&template.model_type)
            } else {
                template.capabilities
            },
            supports_streaming: template.supports_streaming,
            recommended_concurrency: template.recommended_concurrency,
        })
        .collect()
}
