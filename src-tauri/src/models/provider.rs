use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ProviderSummary {
    pub id: String,
    pub name: String,
    pub base_url_masked: String,
    pub api_key_masked: String,
    pub interface_type: String,
    pub status: String,
    pub model_count: i64,
    pub last_checked_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProviderInput {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub interface_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProviderInput {
    pub name: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub interface_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelSummary {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub model_type: String,
    pub capabilities: Vec<String>,
    pub supports_streaming: bool,
    pub recommended_concurrency: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderModelScanResult {
    pub provider_id: String,
    pub models: Vec<ModelSummary>,
    pub message: String,
    pub scanned_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteResult {
    pub id: String,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderConnectionResult {
    pub provider_id: String,
    pub ok: bool,
    pub status: String,
    pub message: String,
    pub checked_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProviderDiagnosticsInput {
    pub provider_id: String,
    pub model_id: Option<String>,
    pub dataset_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderDiagnosticsResult {
    pub provider_id: String,
    pub status: String,
    pub checked_at: String,
    pub engine_mode: String,
    pub endpoints: Vec<DiagnosticEndpoint>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiagnosticEndpoint {
    pub name: String,
    pub method: String,
    pub path: String,
    pub ok: bool,
    pub latency_ms: Option<i64>,
    pub http_status: Option<i64>,
    pub message: String,
    pub error_kind: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProviderConnectionConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key_plaintext: String,
    pub interface_type: String,
}

#[derive(Debug, Clone)]
pub struct DiscoveredModel {
    pub name: String,
    pub model_type: String,
    pub capabilities: Vec<String>,
    pub supports_streaming: bool,
    pub recommended_concurrency: Option<i64>,
}
