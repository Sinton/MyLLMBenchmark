use crate::error::{AppError, AppResult};
use crate::models::DatasetImportInput;
use base64::prelude::*;
use calamine::{Reader, Xlsx};
use std::io::Cursor;

const MAX_IMPORT_BYTES: usize = 10 * 1024 * 1024;
const MAX_SAMPLES: usize = 10_000;

#[derive(Debug, Clone)]
pub struct ParsedDatasetImport {
    pub name: String,
    pub dataset_type: String,
    pub prompts: Vec<String>,
    pub average_tokens: i64,
}

pub fn parse_dataset_import(input: DatasetImportInput) -> AppResult<ParsedDatasetImport> {
    let name = normalize_name(&input.name, &input.file_name);
    let dataset_type = normalize_dataset_type(&input.dataset_type);
    let bytes = BASE64_STANDARD
        .decode(input.content_base64.as_bytes())
        .map_err(|_| AppError::validation("数据集文件内容不是有效的 base64"))?;
    if bytes.len() > MAX_IMPORT_BYTES {
        return Err(AppError::validation("数据集文件不能超过 10MB"));
    }

    let prompts = match input.format.trim().to_ascii_lowercase().as_str() {
        "jsonl" => parse_jsonl(&bytes)?,
        "csv" => parse_csv(&bytes)?,
        "txt" => parse_txt(&bytes),
        "excel" | "xlsx" => parse_xlsx(&bytes)?,
        _ => return Err(AppError::validation("暂不支持该数据集格式")),
    };
    let prompts = normalize_prompts(prompts)?;
    let average_tokens = if prompts.is_empty() {
        0
    } else {
        prompts
            .iter()
            .map(|prompt| estimate_tokens(prompt))
            .sum::<i64>()
            / prompts.len() as i64
    };

    Ok(ParsedDatasetImport {
        name,
        dataset_type,
        prompts,
        average_tokens,
    })
}

pub fn estimate_tokens(prompt: &str) -> i64 {
    ((prompt.chars().count() as f64) / 4.0).ceil().max(1.0) as i64
}

fn parse_jsonl(bytes: &[u8]) -> AppResult<Vec<String>> {
    let content = String::from_utf8_lossy(bytes);
    let mut prompts = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line).map_err(|error| {
            AppError::validation(format!("JSONL 第 {} 行解析失败：{}", index + 1, error))
        })?;
        if let Some(prompt) = value.get("prompt").and_then(|item| item.as_str()) {
            prompts.push(prompt.to_string());
        } else if let Some(prompt) = value.get("input").and_then(|item| item.as_str()) {
            prompts.push(prompt.to_string());
        } else if let Some(messages) = value.get("messages").and_then(|item| item.as_array()) {
            prompts.push(messages_to_prompt(messages));
        }
    }
    Ok(prompts)
}

fn parse_csv(bytes: &[u8]) -> AppResult<Vec<String>> {
    let mut reader = csv::ReaderBuilder::new().flexible(true).from_reader(bytes);
    let headers = reader
        .headers()
        .map_err(|error| AppError::validation(format!("CSV 表头解析失败：{error}")))?
        .clone();
    let prompt_index = headers
        .iter()
        .position(|header| {
            let header = header.trim().to_ascii_lowercase();
            header == "prompt" || header == "input"
        })
        .unwrap_or(0);

    let mut prompts = Vec::new();
    for row in reader.records() {
        let row = row.map_err(|error| AppError::validation(format!("CSV 行解析失败：{error}")))?;
        if let Some(value) = row.get(prompt_index) {
            prompts.push(value.to_string());
        }
    }
    Ok(prompts)
}

fn parse_txt(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .map(ToString::to_string)
        .collect()
}

fn parse_xlsx(bytes: &[u8]) -> AppResult<Vec<String>> {
    let cursor = Cursor::new(bytes.to_vec());
    let mut workbook = Xlsx::new(cursor)
        .map_err(|error| AppError::validation(format!("Excel 解析失败：{error}")))?;
    let range = workbook
        .worksheet_range_at(0)
        .ok_or_else(|| AppError::validation("Excel 文件没有可读取的工作表"))?
        .map_err(|error| AppError::validation(format!("Excel 工作表解析失败：{error}")))?;

    let mut rows = range.rows();
    let first = rows
        .next()
        .ok_or_else(|| AppError::validation("Excel 工作表没有数据"))?;
    let prompt_index = first
        .iter()
        .position(|cell| {
            let text = cell.to_string().trim().to_ascii_lowercase();
            text == "prompt" || text == "input"
        })
        .unwrap_or(0);

    let first_is_header = first.iter().any(|cell| {
        let text = cell.to_string().trim().to_ascii_lowercase();
        text == "prompt" || text == "input"
    });

    let mut prompts = Vec::new();
    if !first_is_header {
        if let Some(cell) = first.get(prompt_index) {
            prompts.push(cell.to_string());
        }
    }
    for row in rows {
        if let Some(cell) = row.get(prompt_index) {
            prompts.push(cell.to_string());
        }
    }
    Ok(prompts)
}

fn normalize_prompts(prompts: Vec<String>) -> AppResult<Vec<String>> {
    let prompts: Vec<String> = prompts
        .into_iter()
        .map(|prompt| prompt.trim().to_string())
        .filter(|prompt| !prompt.is_empty())
        .take(MAX_SAMPLES)
        .collect();
    if prompts.is_empty() {
        return Err(AppError::validation("数据集没有可用 Prompt 样本"));
    }
    Ok(prompts)
}

fn messages_to_prompt(messages: &[serde_json::Value]) -> String {
    messages
        .iter()
        .filter_map(|message| {
            let role = message
                .get("role")
                .and_then(|item| item.as_str())
                .unwrap_or("user");
            let content = message.get("content").and_then(|item| item.as_str())?;
            Some(format!("{role}: {content}"))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_name(name: &str, file_name: &str) -> String {
    let name = name.trim();
    if !name.is_empty() {
        return name.to_string();
    }
    file_name
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(file_name)
        .trim()
        .to_string()
}

fn normalize_dataset_type(dataset_type: &str) -> String {
    let dataset_type = dataset_type.trim();
    if dataset_type.is_empty() {
        "Chat".to_string()
    } else {
        dataset_type.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::parse_dataset_import;
    use crate::models::DatasetImportInput;
    use base64::prelude::*;

    #[test]
    fn parses_jsonl_prompt_samples() {
        let content =
            BASE64_STANDARD.encode("{\"prompt\":\"介绍杭州\"}\n{\"input\":\"解释 Transformer\"}");
        let parsed = parse_dataset_import(DatasetImportInput {
            name: "Chat Set".to_string(),
            dataset_type: "Chat".to_string(),
            format: "JSONL".to_string(),
            file_name: "chat.jsonl".to_string(),
            content_base64: content,
        })
        .unwrap();
        assert_eq!(parsed.prompts.len(), 2);
        assert!(parsed.average_tokens > 0);
    }
}
