use crate::models::ReportDetail;

pub(crate) fn format_metric_value(value: &serde_json::Value, unit: Option<&str>) -> String {
    let raw = value
        .as_str()
        .map(ToString::to_string)
        .or_else(|| value.as_i64().map(|value| value.to_string()))
        .or_else(|| value.as_f64().map(|value| format!("{value:.2}")))
        .unwrap_or_else(|| "-".to_string());
    if let Some(unit) = unit.filter(|unit| !unit.is_empty()) {
        format!("{raw} {unit}")
    } else {
        raw
    }
}

pub(crate) fn join_numbers(values: &[i64]) -> String {
    if values.is_empty() {
        return "-".to_string();
    }
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" -> ")
}

pub(crate) fn request_log_meta_text(total_records: i64, body_records: i64) -> String {
    if total_records <= 0 {
        return "未采集".to_string();
    }
    if body_records > 0 {
        format!("{total_records} 条索引，{body_records} 条正文可用")
    } else {
        format!("{total_records} 条索引，未保存正文")
    }
}

pub(crate) fn request_log_appendix_text(detail: &ReportDetail) -> String {
    if detail.request_log_meta.total_records <= 0 {
        return "本次报告未记录请求级明细。".to_string();
    }
    format!(
        "本次报告记录请求级明细索引 {} 条，其中正文可用 {} 条；单条详情请在 MyLLMBenchmark 客户端内查看。",
        detail.request_log_meta.total_records, detail.request_log_meta.body_records
    )
}

pub(crate) fn ttft_source_text(source: &str) -> &'static str {
    match source {
        "streaming_real" => "真实流式首 token 延迟",
        "non_streaming_approximation" => "非流式完整响应耗时近似",
        "historical_estimated" => "历史兼容估算",
        "not_applicable" => "不适用",
        _ => "未标注",
    }
}

pub(crate) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub(crate) fn escape_xml(value: &str) -> String {
    escape_html(value).replace('\'', "&apos;")
}
