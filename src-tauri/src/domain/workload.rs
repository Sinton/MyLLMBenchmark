use crate::domain::model_type::ModelType;

#[derive(Debug, Clone)]
pub struct WorkloadConfig {
    pub streaming: bool,
    pub temperature: f64,
    pub max_output_tokens: i64,
    pub prompt_profile: String,
    pub batch_size: i64,
    pub text_count_per_request: i64,
    pub documents_per_query: i64,
    pub top_k: i64,
    pub image_profile: String,
    pub image_count: i64,
}

impl WorkloadConfig {
    pub fn for_model_type(model_type: &str) -> Self {
        match ModelType::normalize(model_type) {
            ModelType::Embedding => Self {
                streaming: false,
                temperature: 0.0,
                max_output_tokens: 0,
                prompt_profile: "mixed".to_string(),
                batch_size: 16,
                text_count_per_request: 16,
                documents_per_query: 0,
                top_k: 0,
                image_profile: "medium".to_string(),
                image_count: 0,
            },
            ModelType::Rerank => Self {
                streaming: false,
                temperature: 0.0,
                max_output_tokens: 0,
                prompt_profile: "mixed".to_string(),
                batch_size: 0,
                text_count_per_request: 0,
                documents_per_query: 30,
                top_k: 10,
                image_profile: "medium".to_string(),
                image_count: 0,
            },
            ModelType::Multimodal => Self {
                streaming: true,
                temperature: 0.2,
                max_output_tokens: 512,
                prompt_profile: "mixed".to_string(),
                batch_size: 0,
                text_count_per_request: 0,
                documents_per_query: 0,
                top_k: 0,
                image_profile: "medium".to_string(),
                image_count: 1,
            },
            ModelType::TextGeneration => Self {
                streaming: true,
                temperature: 0.7,
                max_output_tokens: 512,
                prompt_profile: "mixed".to_string(),
                batch_size: 0,
                text_count_per_request: 0,
                documents_per_query: 0,
                top_k: 0,
                image_profile: "medium".to_string(),
                image_count: 0,
            },
        }
    }

    pub fn from_value(model_type: &str, value: Option<&serde_json::Value>) -> Self {
        let mut config = Self::for_model_type(model_type);
        if let Some(value) = value {
            config.streaming = json_bool(value, "streaming", config.streaming);
            config.temperature = json_f64_or(value, "temperature", config.temperature);
            config.max_output_tokens =
                json_i64_or(value, "max_output_tokens", config.max_output_tokens);
            config.prompt_profile = json_string(value, "prompt_profile", &config.prompt_profile);
            config.batch_size = json_i64_or(value, "batch_size", config.batch_size);
            config.text_count_per_request = json_i64_or(
                value,
                "text_count_per_request",
                config.text_count_per_request,
            );
            config.documents_per_query =
                json_i64_or(value, "documents_per_query", config.documents_per_query);
            config.top_k = json_i64_or(value, "top_k", config.top_k);
            config.image_profile = json_string(value, "image_profile", &config.image_profile);
            config.image_count = json_i64_or(value, "image_count", config.image_count);
        }
        config
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "streaming": self.streaming,
            "temperature": self.temperature,
            "max_output_tokens": self.max_output_tokens,
            "prompt_profile": self.prompt_profile,
            "batch_size": self.batch_size,
            "text_count_per_request": self.text_count_per_request,
            "documents_per_query": self.documents_per_query,
            "top_k": self.top_k,
            "image_profile": self.image_profile,
            "image_count": self.image_count
        })
    }
}

pub fn default_workload_config(model_type: &str) -> serde_json::Value {
    WorkloadConfig::for_model_type(model_type).to_json()
}

pub fn merge_workload_config(model_type: &str, value: serde_json::Value) -> serde_json::Value {
    let mut base = default_workload_config(model_type);
    if let (Some(base_map), Some(input_map)) = (base.as_object_mut(), value.as_object()) {
        for (key, value) in input_map {
            base_map.insert(key.clone(), value.clone());
        }
    }
    base
}

pub fn parse_workload_config(value: String, model_type: &str) -> serde_json::Value {
    let parsed =
        serde_json::from_str::<serde_json::Value>(&value).unwrap_or_else(|_| serde_json::json!({}));
    merge_workload_config(model_type, parsed)
}

pub fn json_i64(value: &serde_json::Value, key: &str) -> i64 {
    value.get(key).and_then(|item| item.as_i64()).unwrap_or(0)
}

fn json_i64_or(value: &serde_json::Value, key: &str, fallback: i64) -> i64 {
    value
        .get(key)
        .and_then(|item| item.as_i64())
        .unwrap_or(fallback)
}

fn json_f64_or(value: &serde_json::Value, key: &str, fallback: f64) -> f64 {
    value
        .get(key)
        .and_then(|item| item.as_f64())
        .unwrap_or(fallback)
}

fn json_bool(value: &serde_json::Value, key: &str, fallback: bool) -> bool {
    value
        .get(key)
        .and_then(|item| item.as_bool())
        .unwrap_or(fallback)
}

fn json_string(value: &serde_json::Value, key: &str, fallback: &str) -> String {
    value
        .get(key)
        .and_then(|item| item.as_str())
        .unwrap_or(fallback)
        .to_string()
}
