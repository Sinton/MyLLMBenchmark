pub(super) fn embeddings_body(model: &str, inputs: Vec<String>) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "input": inputs
    })
}

pub(super) fn diagnostic_inputs() -> Vec<String> {
    vec![
        "LLMBench embedding diagnostic sample one".to_string(),
        "LLMBench embedding diagnostic sample two".to_string(),
    ]
}
