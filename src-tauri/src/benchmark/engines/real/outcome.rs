use super::helpers::duration_ms;
use reqwest::StatusCode;
use std::time::Duration;

#[derive(Debug, Clone)]
pub(crate) struct RequestOutcome {
    pub(crate) ok: bool,
    pub(crate) request_index: i64,
    pub(crate) sample_index: i64,
    pub(crate) latency_ms: i64,
    pub(crate) ttft_ms: i64,
    pub(crate) usage: TokenUsage,
    pub(crate) units: RequestUnits,
    pub(crate) error_kind: Option<&'static str>,
    pub(crate) error_message: Option<String>,
    pub(crate) prompt: Option<String>,
    pub(crate) response_text: Option<String>,
    pub(crate) raw_usage: Option<serde_json::Value>,
}

impl RequestOutcome {
    pub(crate) fn success(latency: Duration, ttft: Duration, usage: TokenUsage) -> Self {
        Self::success_with_units(latency, ttft, usage, RequestUnits::default())
    }

    pub(crate) fn success_with_units(
        latency: Duration,
        ttft: Duration,
        usage: TokenUsage,
        units: RequestUnits,
    ) -> Self {
        Self {
            ok: true,
            request_index: 0,
            sample_index: 0,
            latency_ms: duration_ms(latency),
            ttft_ms: duration_ms(ttft),
            usage,
            units,
            error_kind: None,
            error_message: None,
            prompt: None,
            response_text: None,
            raw_usage: None,
        }
    }

    pub(crate) fn failure(kind: &'static str, message: &str, latency: Duration) -> Self {
        Self {
            ok: false,
            request_index: 0,
            sample_index: 0,
            latency_ms: duration_ms(latency),
            ttft_ms: 0,
            usage: TokenUsage::default(),
            units: RequestUnits::default(),
            error_kind: Some(kind),
            error_message: Some(message.chars().take(180).collect()),
            prompt: None,
            response_text: None,
            raw_usage: None,
        }
    }

    pub(crate) fn with_metadata(mut self, request_index: i64, sample_index: i64) -> Self {
        self.request_index = request_index;
        self.sample_index = sample_index;
        self
    }

    pub(crate) fn with_body(
        mut self,
        prompt: Option<String>,
        response_text: Option<String>,
        raw_usage: Option<serde_json::Value>,
    ) -> Self {
        self.prompt = prompt;
        self.response_text = response_text;
        self.raw_usage = raw_usage;
        self
    }

    pub(crate) fn from_status(status: StatusCode, latency: Duration) -> Self {
        let kind = if status.is_client_error() {
            "http_4xx"
        } else if status.is_server_error() {
            "http_5xx"
        } else {
            "http"
        };
        Self::failure(kind, &format!("HTTP {status}"), latency)
    }

    pub(crate) fn from_reqwest_error(error: reqwest::Error, latency: Duration) -> Self {
        let kind = if error.is_timeout() {
            "timeout"
        } else if error.is_connect() {
            "connection"
        } else {
            "request"
        };
        Self::failure(kind, &error.to_string(), latency)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RequestUnits {
    pub(crate) batch_size: i64,
    pub(crate) text_count: i64,
    pub(crate) documents_per_query: i64,
    pub(crate) pair_count: i64,
    pub(crate) image_count: i64,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TokenUsage {
    pub(crate) input_tokens: i64,
    pub(crate) output_tokens: i64,
    pub(crate) total_tokens: i64,
}

impl TokenUsage {
    pub(crate) fn estimated(prompt: &str, output: &str) -> Self {
        let input_tokens = estimate_tokens(prompt);
        let output_tokens = estimate_tokens(output);
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }
}

pub(crate) fn usage_from_value(value: &serde_json::Value) -> Option<TokenUsage> {
    let usage = value.get("usage")?;
    usage_from_usage_value(usage)
}

pub(crate) fn usage_from_usage_value(usage: &serde_json::Value) -> Option<TokenUsage> {
    let input_tokens = usage
        .get("prompt_tokens")
        .or_else(|| usage.get("input_tokens"))
        .and_then(|item| item.as_i64())
        .unwrap_or(0);
    let output_tokens = usage
        .get("completion_tokens")
        .or_else(|| usage.get("output_tokens"))
        .and_then(|item| item.as_i64())
        .unwrap_or(0);
    let total_tokens = usage
        .get("total_tokens")
        .and_then(|item| item.as_i64())
        .unwrap_or(input_tokens + output_tokens);
    Some(TokenUsage {
        input_tokens,
        output_tokens,
        total_tokens,
    })
}

pub(crate) fn raw_usage_from_value(value: &serde_json::Value) -> Option<serde_json::Value> {
    value.get("usage").cloned()
}

pub(crate) fn estimate_tokens(text: &str) -> i64 {
    ((text.chars().count() as f64) / 4.0).ceil().max(1.0) as i64
}
