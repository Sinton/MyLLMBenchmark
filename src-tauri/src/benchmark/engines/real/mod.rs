mod client;
pub mod diagnostics;
mod helpers;
mod metrics;
mod outcome;
mod protocol;
mod providers;
mod request_logs;
mod runtime;
mod streaming;

pub use client::RealProviderClient;
pub use helpers::{api_url, classify_model};
pub(crate) use outcome::RequestOutcome;
pub use protocol::RealProviderProtocol;
pub use runtime::RealBenchmarkRuntime;
pub(crate) use streaming::StreamDeltaObserver;

use crate::domain::workload::WorkloadConfig;

pub(crate) fn build_text_generation_request_body(
    protocol: RealProviderProtocol,
    model: &str,
    prompt: &str,
    workload: &WorkloadConfig,
) -> serde_json::Value {
    match protocol {
        RealProviderProtocol::OpenAICompatible => {
            if workload.streaming {
                providers::openai_compatible::streaming_completion_body(model, prompt, workload)
            } else {
                providers::openai_compatible::completion_body(model, prompt, workload, false)
            }
        }
        RealProviderProtocol::OpenAIResponses => {
            if workload.streaming {
                providers::openai_responses::streaming_response_body(model, prompt, workload)
            } else {
                providers::openai_responses::response_body(model, prompt, workload)
            }
        }
        RealProviderProtocol::Anthropic => {
            if workload.streaming {
                providers::anthropic::streaming_messages_body(model, prompt, workload)
            } else {
                providers::anthropic::messages_body(model, prompt, workload)
            }
        }
        RealProviderProtocol::Gemini => providers::gemini::generate_content_body(prompt, workload),
    }
}

#[cfg(test)]
mod tests;
