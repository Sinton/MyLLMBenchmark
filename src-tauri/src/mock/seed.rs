use super::now;
use super::types::MockData;
use crate::domain::dataset_import::estimate_tokens;
use crate::domain::demo_samples::build_chat_prompts;
use crate::domain::model_catalog::{model_summaries_for_interface, CatalogFlavor};
use crate::models::{DatasetSample, DatasetSummary, ProviderSummary};
use std::collections::HashMap;

pub(in crate::mock) fn seed_mock_data() -> MockData {
    let now = now();
    let provider_id = "mock-provider-openai".to_string();
    let chat_samples = build_chat_samples();
    let chat_average_tokens = average_tokens(&chat_samples);
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
        DatasetSummary {
            id: "mock-dataset-chat".to_string(),
            name: "Chat Prompt Mock Set".to_string(),
            dataset_type: "Chat".to_string(),
            sample_count: chat_samples.len() as i64,
            average_tokens: chat_average_tokens,
            updated_at: now.clone(),
        },
        DatasetSummary {
            id: "mock-dataset-embedding".to_string(),
            name: "Embedding Corpus Mock Set".to_string(),
            dataset_type: "Embedding".to_string(),
            sample_count: 2048,
            average_tokens: 180,
            updated_at: now.clone(),
        },
        DatasetSummary {
            id: "mock-dataset-rerank".to_string(),
            name: "Reranker Query-Doc Mock Set".to_string(),
            dataset_type: "Reranker".to_string(),
            sample_count: 512,
            average_tokens: 760,
            updated_at: now.clone(),
        },
        DatasetSummary {
            id: "mock-dataset-vision".to_string(),
            name: "Vision Multimodal Mock Set".to_string(),
            dataset_type: "Vision".to_string(),
            sample_count: 96,
            average_tokens: 120,
            updated_at: now,
        },
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
        dataset_samples: HashMap::from([("mock-dataset-chat".to_string(), chat_samples)]),
        ..Default::default()
    }
}

fn build_chat_samples() -> Vec<DatasetSample> {
    build_chat_prompts()
        .into_iter()
        .enumerate()
        .map(|(index, prompt)| DatasetSample {
            id: format!("mock-sample-chat-{}", index + 1),
            dataset_id: "mock-dataset-chat".to_string(),
            sample_index: index as i64,
            estimated_tokens: estimate_tokens(&prompt),
            prompt,
        })
        .collect()
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
