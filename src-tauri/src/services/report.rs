use crate::benchmark::events;
use crate::error::AppResult;
use crate::models::{ReportDetail, ReportExportInput, ReportExportResult, ReportSummary};
use crate::report::document::{export_file_meta, ReportDocument};
use crate::state::AppState;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter, Manager};

pub async fn generate_report(
    app: AppHandle,
    state: &AppState,
    task_id: &str,
) -> AppResult<ReportSummary> {
    let report = state.generate_report(task_id).await?;
    let _ = app.emit(events::REPORT_READY, report.clone());
    Ok(report)
}

pub async fn list_reports(state: &AppState) -> AppResult<Vec<ReportSummary>> {
    Ok(state.list_reports().await?)
}

pub async fn get_report_detail(state: &AppState, report_id: &str) -> AppResult<ReportDetail> {
    Ok(state.get_report_detail(report_id).await?)
}

pub async fn export_report(
    app: AppHandle,
    state: &AppState,
    input: ReportExportInput,
) -> AppResult<ReportExportResult> {
    let detail = state.get_report_detail(&input.report_id).await?;
    let format = normalize_export_format(&input.format);
    let document = ReportDocument::from_detail(&detail, input.template.as_deref());
    let bytes = match format.as_str() {
        "html" => document.render_html(),
        "pdf" => document.render_pdf(),
        "docx" => document.render_docx(),
        "json" => serde_json::to_vec_pretty(&detail).map_err(anyhow::Error::from)?,
        _ => document.render_html(),
    };
    let file_name = build_export_file_name(&detail, &format, input.template.as_deref());
    let export_dir = app
        .path()
        .app_data_dir()
        .map_err(anyhow::Error::from)?
        .join("exports")
        .join("reports");
    tokio::fs::create_dir_all(&export_dir)
        .await
        .map_err(anyhow::Error::from)?;
    let file_path = export_dir.join(&file_name);
    tokio::fs::write(&file_path, bytes)
        .await
        .map_err(anyhow::Error::from)?;

    Ok(export_file_meta(
        &input.report_id,
        &format,
        file_name,
        path_to_string(file_path),
    ))
}

fn normalize_export_format(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "word" | "docx" => "docx".to_string(),
        "pdf" => "pdf".to_string(),
        "json" => "json".to_string(),
        _ => "html".to_string(),
    }
}

fn build_export_file_name(detail: &ReportDetail, format: &str, template: Option<&str>) -> String {
    let template = template
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("交付报告");
    format!(
        "{}-{}-{}.{}",
        sanitize_file_part(&detail.summary.model_name),
        sanitize_file_part(template),
        chrono::Local::now().format("%Y%m%d%H%M%S"),
        format
    )
}

fn sanitize_file_part(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| match ch {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '-',
            _ => ch,
        })
        .collect::<String>();
    let sanitized = sanitized.trim().trim_matches('-');
    if sanitized.is_empty() {
        "LLMBench".to_string()
    } else {
        sanitized.chars().take(48).collect()
    }
}

fn path_to_string(path: PathBuf) -> String {
    path.to_string_lossy().to_string()
}
