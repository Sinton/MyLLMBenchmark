pub(crate) fn embeddings_body(model: &str, inputs: Vec<String>) -> serde_json::Value {
    serde_json::json!({
        "model": model,
        "input": inputs
    })
}

pub(crate) fn diagnostic_inputs() -> Vec<String> {
    vec![
        "MyLLMBenchmark embedding diagnostic sample one".to_string(),
        "MyLLMBenchmark embedding diagnostic sample two".to_string(),
    ]
}
