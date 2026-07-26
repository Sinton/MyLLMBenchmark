use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealProviderProtocol {
    OpenAICompatible,
    OpenAIResponses,
    Anthropic,
    Gemini,
}

impl RealProviderProtocol {
    pub fn from_interface_type(value: &str) -> Option<Self> {
        match value {
            "OpenAI-Response" => Some(Self::OpenAIResponses),
            "Anthropic" => Some(Self::Anthropic),
            "Gemini" => Some(Self::Gemini),
            "OpenAI" | "OpenAI Compatible" | "Jina Rerank" => Some(Self::OpenAICompatible),
            _ => Some(Self::OpenAICompatible),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::OpenAICompatible => "OpenAI Compatible",
            Self::OpenAIResponses => "OpenAI Responses",
            Self::Anthropic => "Anthropic",
            Self::Gemini => "Gemini",
        }
    }

    pub fn engine_mode(self) -> &'static str {
        match self {
            Self::OpenAICompatible => "openai_compatible",
            Self::OpenAIResponses => "openai_responses",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
        }
    }
}

pub(crate) fn ensure_success(status: StatusCode, operation: &str) -> anyhow::Result<()> {
    if status.is_success() {
        return Ok(());
    }

    let message = if status == StatusCode::NOT_FOUND {
        format!(
            "{operation} failed with HTTP {status}; check Base URL (OpenAI Compatible usually ends with /v1), API Key and model permissions"
        )
    } else if status.is_client_error() {
        format!(
            "{operation} failed with HTTP {status}; check Base URL, API Key and model permissions"
        )
    } else if status.is_server_error() {
        format!("{operation} failed with HTTP {status}; provider service returned an error")
    } else {
        format!("{operation} failed with HTTP {status}")
    };
    Err(anyhow::anyhow!(message))
}

pub(crate) fn map_reqwest_error(error: reqwest::Error) -> anyhow::Error {
    if error.is_timeout() {
        anyhow::anyhow!("request timeout while connecting provider")
    } else if error.is_connect() {
        anyhow::anyhow!("failed to connect provider endpoint")
    } else {
        anyhow::anyhow!("provider request failed: {}", error)
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ModelsResponse {
    pub(crate) data: Vec<ModelItem>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ModelItem {
    pub(crate) id: String,
}
