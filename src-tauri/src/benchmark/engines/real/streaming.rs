use super::outcome::{
    usage_from_usage_value, usage_from_value, RequestOutcome, RequestUnits, TokenUsage,
};
use super::protocol::RealProviderProtocol;
use super::providers::gemini;
use futures_util::StreamExt;
use tokio::time::{Duration, Instant};

pub(super) async fn collect_streaming_response(
    response: reqwest::Response,
    protocol: RealProviderProtocol,
    prompt: &str,
    started: Instant,
    units: RequestUnits,
) -> RequestOutcome {
    let mut stream = response.bytes_stream();
    let mut parser = SseBuffer::default();
    let mut output = String::new();
    let mut first_token_at = None;
    let mut usage = None;
    let mut raw_usage = None;

    while let Some(chunk) = stream.next().await {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(error) => {
                return RequestOutcome::from_reqwest_error(error, started.elapsed()).with_body(
                    Some(prompt.to_string()),
                    Some(output),
                    raw_usage,
                )
            }
        };
        let text = String::from_utf8_lossy(&chunk);
        for data in parser.push(&text) {
            match handle_stream_event(protocol, &data, &mut output, &mut usage, &mut raw_usage) {
                Ok(EventProgress::Text) if first_token_at.is_none() => {
                    first_token_at = Some(started.elapsed());
                }
                Ok(_) => {}
                Err(message) => {
                    return RequestOutcome::failure("parse", &message, started.elapsed()).with_body(
                        Some(prompt.to_string()),
                        Some(output),
                        raw_usage,
                    )
                }
            }
        }
    }

    for data in parser.finish() {
        match handle_stream_event(protocol, &data, &mut output, &mut usage, &mut raw_usage) {
            Ok(EventProgress::Text) if first_token_at.is_none() => {
                first_token_at = Some(started.elapsed());
            }
            Ok(_) => {}
            Err(message) => {
                return RequestOutcome::failure("parse", &message, started.elapsed()).with_body(
                    Some(prompt.to_string()),
                    Some(output),
                    raw_usage,
                )
            }
        }
    }

    if output.is_empty() {
        return RequestOutcome::failure(
            "stream_broken",
            "streaming response ended without output text",
            started.elapsed(),
        )
        .with_body(Some(prompt.to_string()), None, raw_usage);
    }

    RequestOutcome::success_with_units(
        started.elapsed(),
        first_token_at.unwrap_or_else(|| Duration::from_millis(0)),
        usage.unwrap_or_else(|| TokenUsage::estimated(prompt, &output)),
        units,
    )
    .with_body(Some(prompt.to_string()), Some(output), raw_usage)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EventProgress {
    Text,
    Metadata,
    Done,
}

fn handle_stream_event(
    protocol: RealProviderProtocol,
    data: &str,
    output: &mut String,
    usage: &mut Option<TokenUsage>,
    raw_usage: &mut Option<serde_json::Value>,
) -> Result<EventProgress, String> {
    let trimmed = data.trim();
    if trimmed.is_empty() || trimmed == "[DONE]" {
        return Ok(EventProgress::Done);
    }

    let value = serde_json::from_str::<serde_json::Value>(trimmed)
        .map_err(|error| format!("invalid streaming event JSON: {error}"))?;

    if let Some((found_usage, raw)) = usage_from_stream_value(protocol, &value) {
        *usage = Some(found_usage);
        *raw_usage = Some(raw);
    }

    if let Some(delta) = output_delta(protocol, &value) {
        if !delta.is_empty() {
            output.push_str(&delta);
            return Ok(EventProgress::Text);
        }
    }

    Ok(EventProgress::Metadata)
}

fn output_delta(protocol: RealProviderProtocol, value: &serde_json::Value) -> Option<String> {
    match protocol {
        RealProviderProtocol::OpenAICompatible => value
            .pointer("/choices/0/delta/content")
            .and_then(|value| value.as_str())
            .map(ToString::to_string),
        RealProviderProtocol::OpenAIResponses => {
            if value
                .get("type")
                .and_then(|item| item.as_str())
                .is_some_and(|event_type| event_type == "response.output_text.delta")
            {
                return value
                    .get("delta")
                    .and_then(|item| item.as_str())
                    .map(ToString::to_string);
            }
            value
                .get("delta")
                .and_then(|item| item.as_str())
                .or_else(|| value.get("output_text").and_then(|item| item.as_str()))
                .map(ToString::to_string)
        }
        RealProviderProtocol::Anthropic => value
            .pointer("/delta/text")
            .and_then(|item| item.as_str())
            .or_else(|| {
                value
                    .pointer("/content_block/text")
                    .and_then(|item| item.as_str())
            })
            .map(ToString::to_string),
        RealProviderProtocol::Gemini => {
            let text = gemini::extract_text(value);
            (!text.is_empty()).then_some(text)
        }
    }
}

fn usage_from_stream_value(
    protocol: RealProviderProtocol,
    value: &serde_json::Value,
) -> Option<(TokenUsage, serde_json::Value)> {
    match protocol {
        RealProviderProtocol::Gemini => {
            let raw = value.get("usageMetadata")?.clone();
            let (input_tokens, output_tokens, total_tokens) = gemini::usage_from_value(value)?;
            Some((
                TokenUsage {
                    input_tokens,
                    output_tokens,
                    total_tokens,
                },
                raw,
            ))
        }
        RealProviderProtocol::OpenAIResponses => {
            if let Some(raw) = value.get("usage") {
                return usage_from_usage_value(raw).map(|usage| (usage, raw.clone()));
            }
            let response = value.get("response")?;
            let raw = response.get("usage")?;
            usage_from_usage_value(raw).map(|usage| (usage, raw.clone()))
        }
        RealProviderProtocol::Anthropic => {
            if let Some(raw) = value.get("usage") {
                return usage_from_usage_value(raw).map(|usage| (usage, raw.clone()));
            }
            let message = value.get("message")?;
            let raw = message.get("usage")?;
            usage_from_usage_value(raw).map(|usage| (usage, raw.clone()))
        }
        RealProviderProtocol::OpenAICompatible => {
            let raw = value.get("usage")?.clone();
            usage_from_value(value).map(|usage| (usage, raw))
        }
    }
}

#[derive(Default)]
struct SseBuffer {
    buffer: String,
}

impl SseBuffer {
    fn push(&mut self, chunk: &str) -> Vec<String> {
        self.buffer.push_str(chunk);
        let mut events = Vec::new();
        while let Some((index, separator_len)) = find_event_boundary(&self.buffer) {
            let event = self.buffer[..index].to_string();
            self.buffer.drain(..index + separator_len);
            if let Some(data) = event_data(&event) {
                events.push(data);
            }
        }
        events
    }

    fn finish(&mut self) -> Vec<String> {
        if self.buffer.trim().is_empty() {
            self.buffer.clear();
            return Vec::new();
        }
        let event = std::mem::take(&mut self.buffer);
        event_data(&event).into_iter().collect()
    }
}

fn find_event_boundary(buffer: &str) -> Option<(usize, usize)> {
    let lf = buffer.find("\n\n").map(|index| (index, 2));
    let crlf = buffer.find("\r\n\r\n").map(|index| (index, 4));
    match (lf, crlf) {
        (Some(left), Some(right)) => Some(left.min(right)),
        (Some(found), None) | (None, Some(found)) => Some(found),
        (None, None) => None,
    }
}

fn event_data(event: &str) -> Option<String> {
    let data = event
        .lines()
        .filter_map(|line| {
            line.trim_end_matches('\r')
                .trim_start()
                .strip_prefix("data:")
                .map(str::trim)
        })
        .collect::<Vec<_>>();
    if data.is_empty() {
        None
    } else {
        Some(data.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::{handle_stream_event, SseBuffer};
    use crate::benchmark::engines::real::protocol::RealProviderProtocol;

    #[test]
    fn sse_buffer_handles_chunk_boundaries() {
        let mut buffer = SseBuffer::default();
        assert!(buffer.push("data: {\"delta\":\"Hel").is_empty());
        let events = buffer.push("lo\"}\n\n");
        assert_eq!(events, vec![r#"{"delta":"Hello"}"#]);
    }

    #[test]
    fn parses_protocol_specific_deltas() {
        let cases = [
            (
                RealProviderProtocol::OpenAICompatible,
                r#"{"choices":[{"delta":{"content":"OpenAI"}}]}"#,
                "OpenAI",
            ),
            (
                RealProviderProtocol::OpenAIResponses,
                r#"{"type":"response.output_text.delta","delta":"Responses"}"#,
                "Responses",
            ),
            (
                RealProviderProtocol::Anthropic,
                r#"{"type":"content_block_delta","delta":{"type":"text_delta","text":"Anthropic"}}"#,
                "Anthropic",
            ),
            (
                RealProviderProtocol::Gemini,
                r#"{"candidates":[{"content":{"parts":[{"text":"Gemini"}]}}]}"#,
                "Gemini",
            ),
        ];

        for (protocol, payload, expected) in cases {
            let mut output = String::new();
            let mut usage = None;
            let mut raw_usage = None;
            handle_stream_event(protocol, payload, &mut output, &mut usage, &mut raw_usage)
                .unwrap();
            assert_eq!(output, expected);
        }
    }
}
