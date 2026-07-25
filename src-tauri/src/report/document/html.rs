use super::model::ReportDocument;
use super::utils::escape_html;

pub(crate) fn render_html(document: &ReportDocument) -> Vec<u8> {
    let mut html = String::new();
    html.push_str("<!doctype html><html><head><meta charset=\"utf-8\"><title>");
    html.push_str(&escape_html(&document.title));
    html.push_str("</title><style>");
    html.push_str(HTML_STYLE);
    html.push_str("</style></head><body><main class=\"report\"><div class=\"watermark\">");
    html.push_str(&escape_html(&document.source_label));
    html.push_str("</div><header class=\"cover\"><span>MyLLMBenchmark</span><h1>");
    html.push_str(&escape_html(&document.title));
    html.push_str("</h1><p>");
    html.push_str(&escape_html(&document.subtitle));
    html.push_str("</p><div class=\"cover-meta\"><b>");
    html.push_str(&escape_html(document.template.label()));
    html.push_str("</b><b>");
    html.push_str(&escape_html(&document.source_label));
    html.push_str("</b></div></header><section class=\"summary\">");
    for (label, value) in &document.summary {
        html.push_str("<div><span>");
        html.push_str(&escape_html(label));
        html.push_str("</span><strong>");
        html.push_str(&escape_html(value));
        html.push_str("</strong></div>");
    }
    html.push_str("</section>");
    for section in &document.sections {
        html.push_str("<section><h2>");
        html.push_str(&escape_html(&section.title));
        html.push_str("</h2>");
        for paragraph in &section.paragraphs {
            html.push_str("<p>");
            html.push_str(&escape_html(paragraph));
            html.push_str("</p>");
        }
        if section.title == "趋势摘要" {
            if let Some(svg) = &document.trend_svg {
                html.push_str(svg);
            }
        }
        if !section.rows.is_empty() {
            html.push_str("<table>");
            for (index, row) in section.rows.iter().enumerate() {
                html.push_str("<tr>");
                for cell in row {
                    if index == 0 {
                        html.push_str("<th>");
                        html.push_str(&escape_html(cell));
                        html.push_str("</th>");
                    } else {
                        html.push_str("<td>");
                        html.push_str(&escape_html(cell));
                        html.push_str("</td>");
                    }
                }
                html.push_str("</tr>");
            }
            html.push_str("</table>");
        }
        html.push_str("</section>");
    }
    html.push_str("</main></body></html>");
    html.into_bytes()
}

const HTML_STYLE: &str = r#"
@page{size:A4;margin:16mm}
body{margin:0;background:#f9f5f2;color:#2d231d;font-family:"Segoe UI","Microsoft YaHei UI",sans-serif}
.report{position:relative;max-width:1080px;margin:0 auto;padding:36px}
.watermark{position:fixed;right:28px;top:22px;color:#b99a88;font-size:12px;font-weight:800;letter-spacing:.08em;z-index:0}
.cover{position:relative;background:#fff;border:1px solid #e6ddd6;border-radius:18px;padding:30px 32px;margin-bottom:18px;box-shadow:0 18px 42px rgba(82,58,42,.08)}
.cover span{color:#91593c;font-weight:900;font-size:12px}
h1{font-size:30px;line-height:1.25;margin:10px 0;color:#2d231d}
h2{font-size:18px;margin:0 0 12px;color:#2d231d}
p{color:#5f534b;line-height:1.75}
.cover-meta{display:flex;gap:10px;flex-wrap:wrap;margin-top:16px}
.cover-meta b{border:1px solid #e6ddd6;border-radius:999px;background:#f9f5f2;padding:7px 11px;color:#6d4c3b;font-size:12px}
section{position:relative;background:#fff;border:1px solid #e6ddd6;border-radius:14px;padding:18px;margin:14px 0;break-inside:avoid}
.summary{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));gap:10px;background:transparent;border:0;padding:0}
.summary div{background:#fff;border:1px solid #e6ddd6;border-radius:12px;padding:12px;break-inside:avoid}
.summary span{display:block;color:#8c8075;font-size:12px}
.summary strong{display:block;margin-top:6px;font-size:15px}
table{width:100%;border-collapse:separate;border-spacing:0;margin-top:10px;font-size:13px;border:1px solid #eee7e1;border-radius:10px;overflow:hidden}
th,td{border-bottom:1px solid #eee7e1;text-align:left;padding:9px 8px;vertical-align:top}
th{background:#f2ebe5;color:#6d5e53;font-weight:800}
tr:last-child td{border-bottom:0}
.trend-chart{background:#fbf7f4;border:1px solid #eee7e1;border-radius:12px;margin:12px 0;padding:10px}
.trend-chart svg{display:block;width:100%;height:auto}
.trend-chart .axis{fill:none;stroke:#d8cec6;stroke-width:1}
.trend-chart .line{fill:none;stroke-width:3;stroke-linecap:round;stroke-linejoin:round}
.trend-chart .qps{stroke:#91593c}
.trend-chart .latency{stroke:#6f8d7a}
.trend-chart text{fill:#7b6d62;font-size:12px;font-weight:700}
@media print{
  body{background:#fff}
  .report{padding:0}
  .cover,section,.summary div{box-shadow:none}
  .watermark{position:fixed}
}
"#;

use crate::models::ReportDetail;

pub(crate) fn build_trend_svg(detail: &ReportDetail) -> Option<String> {
    if detail.trends.len() < 2 {
        return None;
    }
    let width = 920.0;
    let height = 240.0;
    let padding = 28.0;
    let max_qps = detail
        .trends
        .iter()
        .map(|tick| tick.qps)
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let max_latency = detail
        .trends
        .iter()
        .map(|tick| tick.latency_ms as f64)
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let last = (detail.trends.len() - 1) as f64;
    let qps_points = detail
        .trends
        .iter()
        .enumerate()
        .map(|(index, tick)| {
            let x = padding + (index as f64 / last) * (width - padding * 2.0);
            let y = height - padding - (tick.qps / max_qps) * (height - padding * 2.0);
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    let latency_points = detail
        .trends
        .iter()
        .enumerate()
        .map(|(index, tick)| {
            let x = padding + (index as f64 / last) * (width - padding * 2.0);
            let y = height
                - padding
                - (tick.latency_ms as f64 / max_latency) * (height - padding * 2.0);
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    Some(format!(
        r#"<div class="trend-chart"><svg viewBox="0 0 {width:.0} {height:.0}" role="img" aria-label="QPS 与 P95 趋势"><path d="M {padding:.0} {baseline:.0} H {right:.0}" class="axis"/><path d="M {padding:.0} {padding:.0} V {baseline:.0}" class="axis"/><polyline points="{qps_points}" class="line qps"/><polyline points="{latency_points}" class="line latency"/><text x="{padding:.0}" y="18">QPS</text><text x="{right_minus:.0}" y="18">P95 Latency</text></svg></div>"#,
        baseline = height - padding,
        right = width - padding,
        right_minus = width - padding - 120.0,
    ))
}
