use super::now;
use super::types::MockData;
use crate::domain::dataset_import::estimate_tokens;
use crate::domain::demo_samples::{
    build_chat_prompts, build_embedding_prompts, build_rerank_prompts, build_vision_prompts,
};
use crate::domain::model_catalog::{model_summaries_for_interface, CatalogFlavor};
use crate::models::{DatasetSample, DatasetSummary, ProviderSummary};
use std::collections::HashMap;

pub(in crate::mock) fn seed_mock_data() -> MockData {
    let now = now();
    let provider_id = "mock-provider-openai".to_string();
    let chat_samples = build_samples("mock-dataset-chat", build_chat_prompts());
    let embedding_samples = build_samples("mock-dataset-embedding", build_embedding_prompts());
    let rerank_samples = build_samples("mock-dataset-rerank", build_rerank_prompts());
    let vision_samples = build_samples("mock-dataset-vision", build_vision_prompts());
    let providers = vec![ProviderSummary {
        id: provider_id.clone(),
        name: "OpenAI Compatible Mock".to_string(),
        base_url_masked: "mock://openai-compatible/v1".to_string(),
        api_key_masked: "****************mock".to_string(),
        interface_type: "OpenAI".to_string(),
        status: "online".to_string(),
        model_count: 4,
        last_checked_at: Some(now.clone()),
        created_at: now.clone(),
    }];
    let models = model_summaries_for_interface(&provider_id, "OpenAI", CatalogFlavor::Mock);
    let datasets = vec![
        dataset_summary(
            "mock-dataset-chat",
            "Chat Prompt Mock Set",
            "Chat",
            &chat_samples,
            &now,
        ),
        dataset_summary(
            "mock-dataset-embedding",
            "Embedding Corpus Mock Set",
            "Embedding",
            &embedding_samples,
            &now,
        ),
        dataset_summary(
            "mock-dataset-rerank",
            "Reranker Query-Doc Mock Set",
            "Reranker",
            &rerank_samples,
            &now,
        ),
        dataset_summary(
            "mock-dataset-vision",
            "Vision Multimodal Mock Set",
            "Vision",
            &vision_samples,
            &now,
        ),
    ];

    MockData {
        providers,
        provider_base_urls: HashMap::from([(
            "mock-provider-openai".to_string(),
            "mock://openai-compatible/v1".to_string(),
        )]),
        provider_api_keys: HashMap::from([(
            "mock-provider-openai".to_string(),
            "mock-key".to_string(),
        )]),
        models,
        datasets,
        dataset_samples: HashMap::from([
            ("mock-dataset-chat".to_string(), chat_samples),
            ("mock-dataset-embedding".to_string(), embedding_samples),
            ("mock-dataset-rerank".to_string(), rerank_samples),
            ("mock-dataset-vision".to_string(), vision_samples),
        ]),
        ..Default::default()
    }
}

fn build_samples(dataset_id: &str, prompts: Vec<String>) -> Vec<DatasetSample> {
    prompts
        .into_iter()
        .enumerate()
        .map(|(index, prompt)| DatasetSample {
            id: format!("{dataset_id}-sample-{}", index + 1),
            dataset_id: dataset_id.to_string(),
            sample_index: index as i64,
            estimated_tokens: estimate_tokens(&prompt),
            prompt,
        })
        .collect()
}

fn dataset_summary(
    id: &str,
    name: &str,
    dataset_type: &str,
    samples: &[DatasetSample],
    updated_at: &str,
) -> DatasetSummary {
    DatasetSummary {
        id: id.to_string(),
        name: name.to_string(),
        dataset_type: dataset_type.to_string(),
        sample_count: samples.len() as i64,
        average_tokens: average_tokens(samples),
        updated_at: updated_at.to_string(),
    }
}

fn average_tokens(samples: &[DatasetSample]) -> i64 {
    if samples.is_empty() {
        return 0;
    }
    samples
        .iter()
        .map(|sample| sample.estimated_tokens)
        .sum::<i64>()
        / samples.len() as i64
}
