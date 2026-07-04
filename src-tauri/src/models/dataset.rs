use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct DatasetSummary {
    pub id: String,
    pub name: String,
    pub dataset_type: String,
    pub sample_count: i64,
    pub average_tokens: i64,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetImportInput {
    pub name: String,
    pub dataset_type: String,
    pub format: String,
    pub file_name: String,
    pub content_base64: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetAppendInput {
    pub dataset_id: String,
    pub format: String,
    pub file_name: String,
    pub content_base64: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetUpdateInput {
    pub id: String,
    pub name: String,
    pub dataset_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetSamplePreview {
    pub id: String,
    pub sample_index: i64,
    pub prompt: String,
    pub prompt_preview: String,
    pub estimated_tokens: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetSamplePageInput {
    pub dataset_id: String,
    pub page: i64,
    pub page_size: i64,
    pub keyword: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetSamplePage {
    pub items: Vec<DatasetSamplePreview>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetSampleCreateInput {
    pub dataset_id: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetSampleUpdateInput {
    pub sample_id: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetSampleBatchDeleteInput {
    pub dataset_id: String,
    pub sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetExportInput {
    pub dataset_id: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetExportResult {
    pub dataset_id: String,
    pub format: String,
    pub file_name: String,
    pub file_path: String,
    pub mime_type: String,
    pub sample_count: i64,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatasetValidationIssue {
    pub kind: String,
    pub label: String,
    pub count: i64,
    pub sample_indexes: Vec<i64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatasetValidationResult {
    pub dataset_id: String,
    pub status: String,
    pub checked_at: String,
    pub sample_count: i64,
    pub issues: Vec<DatasetValidationIssue>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DatasetSample {
    pub id: String,
    pub dataset_id: String,
    pub sample_index: i64,
    pub prompt: String,
    pub estimated_tokens: i64,
}
