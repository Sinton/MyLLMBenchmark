use crate::models::ReportExportResult;

pub(crate) fn export_file_meta(
    report_id: &str,
    format: &str,
    file_name: String,
    file_path: String,
) -> ReportExportResult {
    let mime_type = match format {
        "html" => "text/html; charset=utf-8",
        "pdf" => "application/pdf",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "json" => "application/json",
        _ => "application/octet-stream",
    };

    ReportExportResult {
        report_id: report_id.to_string(),
        format: format.to_string(),
        file_name: file_name.clone(),
        file_path,
        mime_type: mime_type.to_string(),
        message: format!("报告已导出为 {file_name}"),
    }
}
