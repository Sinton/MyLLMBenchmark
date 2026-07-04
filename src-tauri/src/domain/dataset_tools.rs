use crate::models::{
    DatasetExportResult, DatasetSample, DatasetSummary, DatasetValidationIssue,
    DatasetValidationResult,
};
use chrono::Utc;
use std::collections::{HashMap, HashSet};

pub struct DatasetExportPayload {
    pub format: String,
    pub file_extension: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

pub fn normalize_dataset_export_format(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "csv" => "csv".to_string(),
        "txt" => "txt".to_string(),
        _ => "jsonl".to_string(),
    }
}

pub fn render_dataset_export(samples: &[DatasetSample], format: &str) -> DatasetExportPayload {
    let format = normalize_dataset_export_format(format);
    match format.as_str() {
        "csv" => DatasetExportPayload {
            format,
            file_extension: "csv".to_string(),
            mime_type: "text/csv;charset=utf-8".to_string(),
            bytes: render_csv(samples).into_bytes(),
        },
        "txt" => DatasetExportPayload {
            format,
            file_extension: "txt".to_string(),
            mime_type: "text/plain;charset=utf-8".to_string(),
            bytes: samples
                .iter()
                .map(|sample| sample.prompt.as_str())
                .collect::<Vec<_>>()
                .join("\n")
                .into_bytes(),
        },
        _ => DatasetExportPayload {
            format,
            file_extension: "jsonl".to_string(),
            mime_type: "application/x-ndjson;charset=utf-8".to_string(),
            bytes: samples
                .iter()
                .map(|sample| serde_json::json!({ "prompt": sample.prompt }).to_string())
                .collect::<Vec<_>>()
                .join("\n")
                .into_bytes(),
        },
    }
}

pub fn dataset_export_result(
    dataset: &DatasetSummary,
    payload: &DatasetExportPayload,
    file_name: String,
    file_path: String,
) -> DatasetExportResult {
    DatasetExportResult {
        dataset_id: dataset.id.clone(),
        format: payload.format.clone(),
        file_name,
        file_path,
        mime_type: payload.mime_type.clone(),
        sample_count: dataset.sample_count,
        message: format!(
            "已导出 {} 条样本为 {} 文件。",
            dataset.sample_count, payload.format
        ),
    }
}

pub fn validate_dataset_samples(
    dataset_id: &str,
    dataset_type: &str,
    samples: &[DatasetSample],
) -> DatasetValidationResult {
    let mut issues = Vec::new();
    let empty_indexes = samples
        .iter()
        .filter(|sample| sample.prompt.trim().is_empty())
        .map(|sample| sample.sample_index + 1)
        .take(8)
        .collect::<Vec<_>>();
    if !empty_indexes.is_empty() {
        issues.push(issue(
            "empty_prompt",
            "空 Prompt",
            empty_indexes.len() as i64,
            empty_indexes,
        ));
    }

    let mut seen = HashMap::<String, Vec<i64>>::new();
    for sample in samples {
        let key = sample.prompt.trim().to_lowercase();
        if !key.is_empty() {
            seen.entry(key).or_default().push(sample.sample_index + 1);
        }
    }
    let duplicate_indexes = seen
        .values()
        .filter(|indexes| indexes.len() > 1)
        .flat_map(|indexes| indexes.iter().copied())
        .take(8)
        .collect::<Vec<_>>();
    let duplicate_count = seen.values().filter(|indexes| indexes.len() > 1).count() as i64;
    if duplicate_count > 0 {
        issues.push(issue(
            "duplicate_prompt",
            "重复 Prompt",
            duplicate_count,
            duplicate_indexes,
        ));
    }

    let long_indexes = samples
        .iter()
        .filter(|sample| sample.prompt.chars().count() > 8_000)
        .map(|sample| sample.sample_index + 1)
        .take(8)
        .collect::<Vec<_>>();
    if !long_indexes.is_empty() {
        issues.push(issue(
            "overlong_prompt",
            "超长 Prompt",
            long_indexes.len() as i64,
            long_indexes,
        ));
    }

    match dataset_type {
        "Vision" | "Multimodal" | "multimodal" => {
            let missing = samples
                .iter()
                .filter(|sample| !has_image_input(&sample.prompt))
                .map(|sample| sample.sample_index + 1)
                .take(8)
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                issues.push(issue(
                    "vision_missing_image",
                    "Vision 样本缺少图片 URL",
                    missing.len() as i64,
                    missing,
                ));
            }
        }
        "Reranker" | "Rerank" | "rerank" => {
            if samples.len() < 2 {
                issues.push(issue(
                    "rerank_not_enough_documents",
                    "Rerank 候选文档不足",
                    1,
                    Vec::new(),
                ));
            }
        }
        _ => {}
    }

    let status = if issues.is_empty() {
        "passed"
    } else if issues.iter().any(|item| {
        matches!(
            item.kind.as_str(),
            "empty_prompt" | "vision_missing_image" | "rerank_not_enough_documents"
        )
    }) {
        "failed"
    } else {
        "warning"
    };

    DatasetValidationResult {
        dataset_id: dataset_id.to_string(),
        status: status.to_string(),
        checked_at: Utc::now().to_rfc3339(),
        sample_count: samples.len() as i64,
        recommendations: recommendations(status, &issues),
        issues,
    }
}

fn render_csv(samples: &[DatasetSample]) -> String {
    let mut rows = vec!["sample_index,prompt,estimated_tokens".to_string()];
    rows.extend(samples.iter().map(|sample| {
        format!(
            "{},{},{}",
            sample.sample_index + 1,
            csv_escape(&sample.prompt),
            sample.estimated_tokens
        )
    }));
    rows.join("\n")
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn has_image_input(prompt: &str) -> bool {
    let trimmed = prompt.trim();
    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("data:image/")
    {
        return true;
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return false;
    };
    if value
        .get("image_url")
        .or_else(|| value.get("image"))
        .and_then(|item| item.as_str())
        .filter(|item| !item.trim().is_empty())
        .is_some()
    {
        return true;
    }
    value
        .get("image_urls")
        .or_else(|| value.get("images"))
        .and_then(|item| item.as_array())
        .map(|items| items.iter().any(|item| item.as_str().is_some()))
        .unwrap_or(false)
}

fn issue(kind: &str, label: &str, count: i64, sample_indexes: Vec<i64>) -> DatasetValidationIssue {
    DatasetValidationIssue {
        kind: kind.to_string(),
        label: label.to_string(),
        count,
        sample_indexes: unique_indexes(sample_indexes),
    }
}

fn unique_indexes(indexes: Vec<i64>) -> Vec<i64> {
    let mut seen = HashSet::new();
    indexes
        .into_iter()
        .filter(|index| seen.insert(*index))
        .collect()
}

fn recommendations(status: &str, issues: &[DatasetValidationIssue]) -> Vec<String> {
    if status == "passed" {
        return vec!["样本质量检查通过，可以用于真实压测。".to_string()];
    }
    issues
        .iter()
        .map(|issue| match issue.kind.as_str() {
            "duplicate_prompt" => {
                "建议去重后再做容量评估，避免少量 Prompt 放大缓存收益。".to_string()
            }
            "overlong_prompt" => "建议拆分超长 Prompt，或单独建立长上下文压测任务。".to_string(),
            "vision_missing_image" => {
                "Vision 数据集需要 image_url / image_urls / images 字段或图片 URL 行。".to_string()
            }
            "rerank_not_enough_documents" => {
                "Rerank 数据集至少需要 query 和候选文档样本，建议导入更完整的候选集合。".to_string()
            }
            _ => "建议清理无效样本后重新检查。".to_string(),
        })
        .collect()
}
