use super::client::RealProviderClient;
use super::helpers::{api_url, classify_model, parse_vision_sample};
use super::outcome::{RequestOutcome, RequestUnits, TokenUsage};
use super::protocol::RealProviderProtocol;
use super::providers::{anthropic, gemini, openai_responses as responses};
use crate::config::BenchmarkEngineMode;
use crate::domain::{model_type::ModelType, workload::WorkloadConfig};
use crate::models::{ProviderConnectionConfig, ProviderDiagnosticsInput};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn joins_api_url_without_duplicate_slashes() {
    assert_eq!(
        api_url("https://example.com", "models"),
        "https://example.com/v1/models"
    );
    assert_eq!(
        api_url("https://example.com/", "models"),
        "https://example.com/v1/models"
    );
    assert_eq!(
        api_url("https://example.com/v1/", "/models"),
        "https://example.com/v1/models"
    );
    assert_eq!(
        api_url("https://example.com/openai/v1/", "/models"),
        "https://example.com/openai/v1/models"
    );
}

#[test]
fn classifies_common_model_names() {
    assert_eq!(classify_model("bge-m3").model_type, "embedding");
    assert_eq!(classify_model("bce-reranker").model_type, "reranker");
    assert_eq!(classify_model("qwen-vl").model_type, "multimodal");
    assert_eq!(classify_model("deepseek-r1").model_type, "text_generation");
}

#[test]
fn maps_real_provider_protocols_from_interface_type() {
    assert_eq!(
        RealProviderProtocol::from_interface_type("OpenAI-Response"),
        Some(RealProviderProtocol::OpenAIResponses)
    );
    assert_eq!(
        RealProviderProtocol::from_interface_type("Anthropic"),
        Some(RealProviderProtocol::Anthropic)
    );
    assert_eq!(
        RealProviderProtocol::from_interface_type("Gemini"),
        Some(RealProviderProtocol::Gemini)
    );
    assert_eq!(
        RealProviderProtocol::from_interface_type("OpenAI"),
        Some(RealProviderProtocol::OpenAICompatible)
    );
}

#[test]
fn extracts_text_from_real_protocol_payloads() {
    let responses_payload = serde_json::json!({
        "output": [{
            "content": [{"type": "output_text", "text": "Responses OK"}]
        }]
    });
    let anthropic_payload = serde_json::json!({
        "content": [{"type": "text", "text": "Anthropic OK"}],
        "usage": {"input_tokens": 10, "output_tokens": 2}
    });
    let gemini_payload = serde_json::json!({
        "candidates": [{
            "content": {"parts": [{"text": "Gemini OK"}]}
        }],
        "usageMetadata": {
            "promptTokenCount": 12,
            "candidatesTokenCount": 3,
            "totalTokenCount": 15
        }
    });

    assert_eq!(
        responses::extract_output_text(&responses_payload),
        "Responses OK"
    );
    assert_eq!(anthropic::extract_text(&anthropic_payload), "Anthropic OK");
    assert_eq!(gemini::extract_text(&gemini_payload), "Gemini OK");
    assert_eq!(gemini::usage_from_value(&gemini_payload), Some((12, 3, 15)));
}

#[test]
fn parses_vision_json_samples_with_image_limit() {
    let sample = parse_vision_sample(
        r#"{"prompt":"描述图片","image_urls":["https://a.test/1.png","https://a.test/2.png"]}"#,
        1,
    );

    assert_eq!(sample.prompt, "描述图片");
    assert_eq!(sample.image_urls, vec!["https://a.test/1.png"]);
}

#[test]
fn embedding_tick_uses_input_token_throughput_and_batch_units() {
    let workload = WorkloadConfig::for_model_type("embedding");
    let results = vec![
        RequestOutcome::success_with_units(
            Duration::from_millis(80),
            Duration::ZERO,
            TokenUsage {
                input_tokens: 160,
                output_tokens: 0,
                total_tokens: 160,
            },
            RequestUnits {
                batch_size: 16,
                text_count: 16,
                ..RequestUnits::default()
            },
        ),
        RequestOutcome::success_with_units(
            Duration::from_millis(90),
            Duration::ZERO,
            TokenUsage {
                input_tokens: 160,
                output_tokens: 0,
                total_tokens: 160,
            },
            RequestUnits {
                batch_size: 16,
                text_count: 16,
                ..RequestUnits::default()
            },
        ),
    ];

    let tick = super::metrics::build_tick_from_results(
        "task",
        1,
        2,
        ModelType::Embedding,
        &workload,
        2.0,
        results,
    );

    assert_eq!(tick.request_count, 2);
    assert_eq!(tick.success_count, 2);
    assert_eq!(tick.batch_size, 16);
    assert_eq!(tick.text_count, 16);
    assert_eq!(tick.tps, 160.0);
    assert_eq!(tick.ttft_ms, 0);
}

#[test]
fn rerank_tick_uses_pair_throughput() {
    let workload = WorkloadConfig::for_model_type("rerank");
    let results = vec![RequestOutcome::success_with_units(
        Duration::from_millis(120),
        Duration::ZERO,
        TokenUsage {
            input_tokens: 300,
            output_tokens: 0,
            total_tokens: 300,
        },
        RequestUnits {
            documents_per_query: 30,
            pair_count: 30,
            ..RequestUnits::default()
        },
    )];

    let tick = super::metrics::build_tick_from_results(
        "task",
        1,
        1,
        ModelType::Rerank,
        &workload,
        1.5,
        results,
    );

    assert_eq!(tick.documents_per_query, 30);
    assert_eq!(tick.pair_count, 20);
    assert_eq!(tick.tps, 20.0);
    assert_eq!(tick.ttft_ms, 0);
}

#[test]
fn vision_tick_preserves_image_units_and_output_throughput() {
    let workload = WorkloadConfig::for_model_type("multimodal");
    let results = vec![RequestOutcome::success_with_units(
        Duration::from_millis(500),
        Duration::from_millis(180),
        TokenUsage {
            input_tokens: 120,
            output_tokens: 40,
            total_tokens: 160,
        },
        RequestUnits {
            image_count: 2,
            ..RequestUnits::default()
        },
    )];

    let tick = super::metrics::build_tick_from_results(
        "task",
        1,
        1,
        ModelType::Multimodal,
        &workload,
        2.0,
        results,
    );

    assert_eq!(tick.image_count, 2);
    assert_eq!(tick.ttft_ms, 180);
    assert_eq!(tick.tps, 20.0);
}

#[tokio::test]
async fn diagnostics_probe_models_and_chat_without_exposing_key() {
    let (base_url, requests, handle) = spawn_test_server();
    let config = ProviderConnectionConfig {
        id: "provider-1".to_string(),
        name: "Local Test".to_string(),
        base_url,
        api_key_plaintext: "sk-secret-for-test".to_string(),
        interface_type: "OpenAI".to_string(),
    };
    let client = RealProviderClient::new().unwrap();

    let result = client
        .diagnose_provider(
            &config,
            &ProviderDiagnosticsInput {
                provider_id: config.id.clone(),
                model_id: None,
                dataset_id: None,
            },
            &[],
            &[],
            BenchmarkEngineMode::OpenaiCompatible,
            "2026-07-13T00:00:00Z".to_string(),
        )
        .await;

    handle.join().unwrap();
    assert_eq!(result.status, "passed");
    assert!(result
        .endpoints
        .iter()
        .any(|endpoint| endpoint.path == "/models"));
    assert!(result
        .endpoints
        .iter()
        .any(|endpoint| endpoint.path == "/chat/completions"));
    let serialized = serde_json::to_string(&result).unwrap();
    assert!(!serialized.contains("sk-secret-for-test"));
    assert!(requests.lock().unwrap().iter().any(|request| request
        .to_ascii_lowercase()
        .contains("authorization: bearer sk-secret-for-test")));
}

#[tokio::test]
async fn responses_streaming_request_records_output_usage_and_path() {
    let body = concat!(
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"Res\"}\n\n",
            "data: {\"type\":\"response.output_text.delta\",\"delta\":\"ponses\"}\n\n",
            "data: {\"type\":\"response.completed\",\"response\":{\"usage\":{\"input_tokens\":7,\"output_tokens\":2,\"total_tokens\":9}}}\n\n",
            "data: [DONE]\n\n"
        );
    let (base_url, requests, handle) =
        spawn_single_response_server("200 OK", "text/event-stream", body);
    let config = protocol_config("OpenAI-Response", base_url);
    let workload = WorkloadConfig::for_model_type("text_generation");
    let client = RealProviderClient::new().unwrap();

    let outcome = client
        .text_generation(
            &config,
            RealProviderProtocol::OpenAIResponses,
            "resp-test",
            "hello",
            &workload,
            30,
        )
        .await;

    handle.join().unwrap();
    assert!(outcome.ok, "{:?}", outcome.error_message);
    assert_eq!(outcome.response_text.as_deref(), Some("Responses"));
    assert_eq!(outcome.usage.input_tokens, 7);
    assert_eq!(outcome.usage.output_tokens, 2);
    let request = requests.lock().unwrap().join("\n");
    assert!(request.starts_with("POST /v1/responses "));
    assert!(request.contains("\"stream\":true"));
    assert!(request
        .to_ascii_lowercase()
        .contains("authorization: bearer sk-secret-for-test"));
}

#[tokio::test]
async fn anthropic_streaming_request_records_output_usage_and_headers() {
    let body = concat!(
            "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Anth\"}}\n\n",
            "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"ropic\"}}\n\n",
            "data: {\"type\":\"message_delta\",\"usage\":{\"output_tokens\":2}}\n\n"
        );
    let (base_url, requests, handle) =
        spawn_single_response_server("200 OK", "text/event-stream", body);
    let config = protocol_config("Anthropic", base_url);
    let workload = WorkloadConfig::for_model_type("text_generation");
    let client = RealProviderClient::new().unwrap();

    let outcome = client
        .text_generation(
            &config,
            RealProviderProtocol::Anthropic,
            "claude-test",
            "hello",
            &workload,
            30,
        )
        .await;

    handle.join().unwrap();
    assert!(outcome.ok, "{:?}", outcome.error_message);
    assert_eq!(outcome.response_text.as_deref(), Some("Anthropic"));
    assert_eq!(outcome.usage.output_tokens, 2);
    let request = requests.lock().unwrap().join("\n");
    assert!(request.starts_with("POST /v1/messages "));
    assert!(request.contains("\"stream\":true"));
    assert!(request
        .to_ascii_lowercase()
        .contains("x-api-key: sk-secret-for-test"));
    assert!(request
        .to_ascii_lowercase()
        .contains("anthropic-version: 2023-06-01"));
}

#[tokio::test]
async fn gemini_streaming_request_records_output_usage_and_alt_sse() {
    let body = concat!(
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Gem\"}]}}]}\n\n",
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"ini\"}]}}],\"usageMetadata\":{\"promptTokenCount\":5,\"candidatesTokenCount\":2,\"totalTokenCount\":7}}\n\n"
        );
    let (base_url, requests, handle) =
        spawn_single_response_server("200 OK", "text/event-stream", body);
    let config = protocol_config("Gemini", base_url);
    let workload = WorkloadConfig::for_model_type("text_generation");
    let client = RealProviderClient::new().unwrap();

    let outcome = client
        .text_generation(
            &config,
            RealProviderProtocol::Gemini,
            "gemini-test",
            "hello",
            &workload,
            30,
        )
        .await;

    handle.join().unwrap();
    assert!(outcome.ok, "{:?}", outcome.error_message);
    assert_eq!(outcome.response_text.as_deref(), Some("Gemini"));
    assert_eq!(outcome.usage.input_tokens, 5);
    assert_eq!(outcome.usage.output_tokens, 2);
    let request = requests.lock().unwrap().join("\n");
    assert!(request.starts_with("POST /v1/models/gemini-test:streamGenerateContent?"));
    assert!(request.contains("alt=sse"));
    assert!(request.contains("key=sk-secret-for-test"));
}

#[tokio::test]
async fn streaming_plain_json_response_is_not_silently_downgraded() {
    let (base_url, _requests, handle) =
        spawn_single_response_server("200 OK", "application/json", r#"{"output_text":"json"}"#);
    let config = protocol_config("OpenAI-Response", base_url);
    let workload = WorkloadConfig::for_model_type("text_generation");
    let client = RealProviderClient::new().unwrap();

    let outcome = client
        .text_generation(
            &config,
            RealProviderProtocol::OpenAIResponses,
            "resp-test",
            "hello",
            &workload,
            30,
        )
        .await;

    handle.join().unwrap();
    assert!(!outcome.ok);
    assert_eq!(outcome.error_kind, Some("stream_broken"));
}

#[tokio::test]
async fn protocol_http_error_is_classified_without_key_leak() {
    let (base_url, _requests, handle) = spawn_single_response_server(
        "401 Unauthorized",
        "application/json",
        r#"{"error":"bad key"}"#,
    );
    let config = protocol_config("Anthropic", base_url);
    let workload = WorkloadConfig::for_model_type("text_generation");
    let client = RealProviderClient::new().unwrap();

    let outcome = client
        .text_generation(
            &config,
            RealProviderProtocol::Anthropic,
            "claude-test",
            "hello",
            &workload,
            30,
        )
        .await;

    handle.join().unwrap();
    assert!(!outcome.ok);
    assert_eq!(outcome.error_kind, Some("http_4xx"));
    assert!(!outcome
        .error_message
        .unwrap_or_default()
        .contains("sk-secret-for-test"));
}

fn protocol_config(interface_type: &str, base_url: String) -> ProviderConnectionConfig {
    ProviderConnectionConfig {
        id: "provider-1".to_string(),
        name: "Local Test".to_string(),
        base_url,
        api_key_plaintext: "sk-secret-for-test".to_string(),
        interface_type: interface_type.to_string(),
    }
}

fn spawn_single_response_server(
    status: &'static str,
    content_type: &'static str,
    body: &'static str,
) -> (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let requests_for_thread = Arc::clone(&requests);
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut buffer = [0_u8; 8192];
        let size = stream.read(&mut buffer).unwrap();
        let request = String::from_utf8_lossy(&buffer[..size]).to_string();
        requests_for_thread.lock().unwrap().push(request);
        let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
        stream.write_all(response.as_bytes()).unwrap();
    });
    (format!("http://{addr}/v1"), requests, handle)
}

fn spawn_test_server() -> (String, Arc<Mutex<Vec<String>>>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let requests_for_thread = Arc::clone(&requests);
    let handle = thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0_u8; 8192];
            let size = stream.read(&mut buffer).unwrap();
            let request = String::from_utf8_lossy(&buffer[..size]).to_string();
            requests_for_thread.lock().unwrap().push(request.clone());
            let body = if request.starts_with("GET /v1/models") {
                r#"{"data":[{"id":"gpt-test"}]}"#
            } else {
                r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"prompt_tokens":4,"completion_tokens":1,"total_tokens":5}}"#
            };
            let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (format!("http://{addr}/v1"), requests, handle)
}
