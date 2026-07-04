#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelType {
    TextGeneration,
    Embedding,
    Rerank,
    Multimodal,
}

impl ModelType {
    pub fn normalize(value: &str) -> Self {
        match value {
            "Chat" | "Text" | "text_generation" => Self::TextGeneration,
            "Embedding" | "embedding" => Self::Embedding,
            "Vision" | "Multimodal" | "multimodal" => Self::Multimodal,
            "Reranker" | "Rerank" | "rerank" | "reranker" => Self::Rerank,
            _ => Self::TextGeneration,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::TextGeneration => "text_generation",
            Self::Embedding => "embedding",
            Self::Rerank => "rerank",
            Self::Multimodal => "multimodal",
        }
    }

    pub fn has_ttft(self) -> bool {
        matches!(self, Self::TextGeneration | Self::Multimodal)
    }

    pub fn default_capabilities(self) -> Vec<String> {
        match self {
            Self::Embedding | Self::Rerank => vec!["batch".to_string()],
            Self::Multimodal => vec!["streaming".to_string(), "image_input".to_string()],
            Self::TextGeneration => vec!["streaming".to_string(), "reasoning".to_string()],
        }
    }
}

pub fn normalize_model_type(value: &str) -> String {
    ModelType::normalize(value).as_str().to_string()
}

pub fn default_capabilities(model_type: &str) -> Vec<String> {
    ModelType::normalize(model_type).default_capabilities()
}
