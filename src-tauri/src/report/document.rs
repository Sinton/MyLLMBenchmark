use crate::models::{ReportDetail, ReportExportResult};

pub(crate) struct ReportDocument {
    title: String,
    subtitle: String,
    source_label: String,
    template: ReportTemplate,
    summary: Vec<(String, String)>,
    sections: Vec<DocumentSection>,
    trend_svg: Option<String>,
}

#[derive(Clone)]
struct DocumentSection {
    title: String,
    paragraphs: Vec<String>,
    rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReportTemplate {
    DeliverySummary,
    OperationsCapacity,
    DetailedAudit,
}

impl ReportDocument {
    pub(crate) fn from_detail(detail: &ReportDetail, template: Option<&str>) -> Self {
        let template = ReportTemplate::from_label(template);
        let source_label = match detail.source.as_str() {
            "measured" => "真实接口实测数据",
            "mock" => "Mock 数据",
            "estimated" => "历史估算数据",
            _ => "未知数据来源",
        };
        let summary = vec![
            ("测试对象".to_string(), detail.summary.model_name.clone()),
            ("服务商".to_string(), detail.summary.provider_name.clone()),
            ("数据集".to_string(), detail.dataset_name.clone()),
            ("数据来源".to_string(), source_label.to_string()),
            (
                "请求明细".to_string(),
                request_log_meta_text(
                    detail.request_log_meta.total_records,
                    detail.request_log_meta.body_records,
                ),
            ),
            (
                "推荐生产并发".to_string(),
                format!("{} 路", detail.summary.recommended_concurrency),
            ),
            (
                "最大稳定并发".to_string(),
                format!("{} 路", detail.summary.max_stable_concurrency),
            ),
            (
                "SLA 结论".to_string(),
                format!(
                    "{}，P95 {}ms，成功率 {:.2}%",
                    detail.verdict_label,
                    detail.summary.p95_latency_ms,
                    detail.summary.success_rate
                ),
            ),
            ("主要瓶颈".to_string(), detail.bottleneck.clone()),
        ];

        let mut sections = vec![
            DocumentSection {
                title: "执行摘要".to_string(),
                paragraphs: vec![
                    detail.capacity_conclusion.clone(),
                    format!(
                        "计划阶梯：{}；实际执行：{}；每阶段请求轮次：{}；请求超时：{}s；SLA 策略：{}。",
                        join_numbers(&detail.planned_stages),
                        join_numbers(&detail.executed_stages),
                        detail.stage_sample_rounds,
                        detail.request_timeout_seconds,
                        detail.sla_stop_policy
                    ),
                ],
                rows: Vec::new(),
            },
            DocumentSection {
                title: "测试配置".to_string(),
                paragraphs: vec![format!(
                    "任务模式：{}；测试时长：{} 秒；SLA：P95 <= {}ms，成功率 >= {:.2}%。",
                    detail.mode,
                    detail.duration_seconds,
                    detail.sla_p95_ms,
                    detail.min_success_rate
                )],
                rows: vec![
                    vec!["配置项".to_string(), "值".to_string()],
                    vec!["任务名称".to_string(), detail.task_name.clone()],
                    vec!["模型类型".to_string(), detail.model_type.clone()],
                    vec!["数据集".to_string(), detail.dataset_name.clone()],
                    vec![
                        "请求明细".to_string(),
                        request_log_meta_text(
                            detail.request_log_meta.total_records,
                            detail.request_log_meta.body_records,
                        ),
                    ],
                    vec!["计划阶梯".to_string(), join_numbers(&detail.planned_stages)],
                    vec![
                        "实际阶梯".to_string(),
                        join_numbers(&detail.executed_stages),
                    ],
                    vec![
                        "请求超时".to_string(),
                        format!("{}s", detail.request_timeout_seconds),
                    ],
                    vec!["SLA 策略".to_string(), detail.sla_stop_policy.clone()],
                ],
            },
            DocumentSection {
                title: "阶段证据".to_string(),
                paragraphs: detail
                    .early_stop_reason
                    .clone()
                    .map(|reason| vec![reason])
                    .unwrap_or_default(),
                rows: std::iter::once(vec![
                    "阶段".to_string(),
                    "并发".to_string(),
                    "请求数".to_string(),
                    "成功/失败".to_string(),
                    "QPS".to_string(),
                    "P95".to_string(),
                    "成功率".to_string(),
                    "SLA".to_string(),
                ])
                .chain(detail.stages.iter().map(|stage| {
                    vec![
                        format!("#{}", stage.stage_index),
                        stage.concurrency.to_string(),
                        stage.request_count.to_string(),
                        format!("{}/{}", stage.success_count, stage.failure_count),
                        format!("{:.2}", stage.qps),
                        format!("{}ms", stage.p95_latency_ms),
                        format!("{:.2}%", stage.success_rate),
                        if stage.sla_passed { "通过" } else { "未达标" }.to_string(),
                    ]
                }))
                .collect(),
            },
            DocumentSection {
                title: "趋势摘要".to_string(),
                paragraphs: vec![format!(
                    "共记录 {} 个实时指标点，末次稳定 QPS {:.2}，TTFT {}ms，输出/专项吞吐 {:.2}。",
                    detail.trends.len(),
                    detail.stable_qps,
                    detail.ttft_ms,
                    detail.tps
                )],
                rows: trend_rows(detail),
            },
            DocumentSection {
                title: detail.specialty.title.clone(),
                paragraphs: std::iter::once(detail.specialty.description.clone())
                    .chain(detail.specialty.guidance.iter().cloned())
                    .collect(),
                rows: std::iter::once(vec![
                    "指标".to_string(),
                    "数值".to_string(),
                    "说明".to_string(),
                ])
                .chain(detail.specialty.metrics.iter().map(|metric| {
                    vec![
                        metric.label.clone(),
                        format_metric_value(&metric.value, metric.unit.as_deref()),
                        metric.hint.clone(),
                    ]
                }))
                .collect(),
            },
            DocumentSection {
                title: "错误分布".to_string(),
                paragraphs: if detail.errors.is_empty() {
                    vec!["本次压测未记录错误。".to_string()]
                } else {
                    Vec::new()
                },
                rows: if detail.errors.is_empty() {
                    Vec::new()
                } else {
                    std::iter::once(vec![
                        "类型".to_string(),
                        "数量".to_string(),
                        "占比".to_string(),
                    ])
                    .chain(detail.errors.iter().map(|bucket| {
                        vec![
                            bucket.label.clone(),
                            bucket.value.to_string(),
                            format!("{}%", bucket.percent),
                        ]
                    }))
                    .collect()
                },
            },
            DocumentSection {
                title: "上线建议".to_string(),
                paragraphs: detail.recommendations.clone(),
                rows: Vec::new(),
            },
            DocumentSection {
                title: "附录".to_string(),
                paragraphs: vec![
                    request_log_appendix_text(detail),
                    "本报告不包含 API Key；PDF/DOCX 默认不导出完整请求正文或模型响应正文。".to_string(),
                    "容量结论基于持久化阶段指标和秒级 tick；历史缺失字段仅在兼容模式下估算。".to_string(),
                ],
                rows: vec![
                    vec!["字段".to_string(), "值".to_string()],
                    vec!["Report ID".to_string(), detail.summary.id.clone()],
                    vec!["Task ID".to_string(), detail.summary.task_id.clone()],
                    vec!["数据来源".to_string(), source_label.to_string()],
                    vec![
                        "请求明细索引".to_string(),
                        format!("{} 条", detail.request_log_meta.total_records),
                    ],
                    vec![
                        "正文可用记录".to_string(),
                        format!("{} 条", detail.request_log_meta.body_records),
                    ],
                    vec!["导出模板".to_string(), template.label().to_string()],
                ],
            },
        ];

        if detail.preflight_result.is_some() || detail.dataset_quality.is_some() {
            sections.push(preflight_section(detail));
        }
        if detail.diagnostics_snapshot.is_some() {
            sections.push(diagnostics_section(detail));
        }

        if sections[0]
            .paragraphs
            .iter()
            .all(|item| item.trim().is_empty())
        {
            sections[0]
                .paragraphs
                .push(detail.summary.recommendation.clone());
        }

        Self {
            title: format!("MyLLMBenchmark 压测报告 - {}", detail.summary.model_name),
            subtitle: format!(
                "{} / {} / {}",
                detail.task_name, detail.summary.provider_name, source_label
            ),
            source_label: source_label.to_string(),
            template,
            summary,
            sections: template.filter_sections(sections),
            trend_svg: build_trend_svg(detail),
        }
    }

    pub(crate) fn render_html(&self) -> Vec<u8> {
        let mut html = String::new();
        html.push_str("<!doctype html><html><head><meta charset=\"utf-8\"><title>");
        html.push_str(&escape_html(&self.title));
        html.push_str("</title><style>");
        html.push_str(HTML_STYLE);
        html.push_str("</style></head><body><main class=\"report\"><div class=\"watermark\">");
        html.push_str(&escape_html(&self.source_label));
        html.push_str("</div><header class=\"cover\"><span>MyLLMBenchmark</span><h1>");
        html.push_str(&escape_html(&self.title));
        html.push_str("</h1><p>");
        html.push_str(&escape_html(&self.subtitle));
        html.push_str("</p><div class=\"cover-meta\"><b>");
        html.push_str(&escape_html(self.template.label()));
        html.push_str("</b><b>");
        html.push_str(&escape_html(&self.source_label));
        html.push_str("</b></div></header><section class=\"summary\">");
        for (label, value) in &self.summary {
            html.push_str("<div><span>");
            html.push_str(&escape_html(label));
            html.push_str("</span><strong>");
            html.push_str(&escape_html(value));
            html.push_str("</strong></div>");
        }
        html.push_str("</section>");
        for section in &self.sections {
            html.push_str("<section><h2>");
            html.push_str(&escape_html(&section.title));
            html.push_str("</h2>");
            for paragraph in &section.paragraphs {
                html.push_str("<p>");
                html.push_str(&escape_html(paragraph));
                html.push_str("</p>");
            }
            if section.title == "趋势摘要" {
                if let Some(svg) = &self.trend_svg {
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

    pub(crate) fn render_pdf(&self) -> Vec<u8> {
        let lines = self.as_plain_lines();
        build_simple_pdf(&lines)
    }

    pub(crate) fn render_docx(&self) -> Vec<u8> {
        let document_xml = self.render_docx_document_xml();
        build_docx_package(&document_xml)
    }

    fn render_docx_document_xml(&self) -> String {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>"#);
        xml.push_str(
            r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>"#,
        );
        push_docx_heading(&mut xml, &self.title, "Title");
        push_docx_paragraph(&mut xml, &self.subtitle);
        push_docx_paragraph(&mut xml, &format!("模板：{}", self.template.label()));
        push_docx_paragraph(&mut xml, &format!("数据来源：{}", self.source_label));
        push_docx_table(&mut xml, &summary_rows(&self.summary));
        for section in &self.sections {
            push_docx_heading(&mut xml, &section.title, "Heading1");
            for paragraph in &section.paragraphs {
                push_docx_paragraph(&mut xml, paragraph);
            }
            if !section.rows.is_empty() {
                push_docx_table(&mut xml, &section.rows);
            }
        }
        xml.push_str(r#"<w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr>"#);
        xml.push_str("</w:body></w:document>");
        xml
    }

    fn as_plain_lines(&self) -> Vec<String> {
        let mut lines = vec![self.title.clone(), self.subtitle.clone(), String::new()];
        for (label, value) in &self.summary {
            lines.push(format!("{label}: {value}"));
        }
        for section in &self.sections {
            lines.push(String::new());
            lines.push(section.title.clone());
            lines.extend(section.paragraphs.iter().cloned());
            lines.extend(section.rows.iter().map(|row| row.join(" | ")));
        }
        lines
    }
}

impl ReportTemplate {
    fn from_label(value: Option<&str>) -> Self {
        match value.unwrap_or("").trim() {
            "运维容量版" => Self::OperationsCapacity,
            "详细审计版" => Self::DetailedAudit,
            _ => Self::DeliverySummary,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::DeliverySummary => "交付摘要版",
            Self::OperationsCapacity => "运维容量版",
            Self::DetailedAudit => "详细审计版",
        }
    }

    fn filter_sections(self, sections: Vec<DocumentSection>) -> Vec<DocumentSection> {
        sections
            .into_iter()
            .filter(|section| match self {
                Self::DeliverySummary => {
                    matches!(section.title.as_str(), "执行摘要" | "阶段证据" | "上线建议")
                }
                Self::OperationsCapacity => {
                    !matches!(section.title.as_str(), "附录" | "兼容性诊断附录")
                }
                Self::DetailedAudit => true,
            })
            .collect()
    }
}

fn preflight_section(detail: &ReportDetail) -> DocumentSection {
    let mut paragraphs = Vec::new();
    if let Some(value) = &detail.preflight_result {
        let status = value
            .get("status")
            .and_then(|item| item.as_str())
            .unwrap_or("unknown");
        paragraphs.push(format!("启动前校验状态：{status}。"));
        if let Some(warnings) = value.get("warnings").and_then(|item| item.as_array()) {
            for warning in warnings.iter().filter_map(|item| item.as_str()) {
                paragraphs.push(format!("提示：{warning}"));
            }
        }
    }
    let mut rows = vec![vec![
        "检查项".to_string(),
        "结果".to_string(),
        "示例样本".to_string(),
    ]];
    if let Some(quality) = &detail.dataset_quality {
        rows.push(vec![
            "数据集质量".to_string(),
            format!("{} / {} 条样本", quality.status, quality.sample_count),
            quality
                .recommendations
                .first()
                .cloned()
                .unwrap_or_else(|| "-".to_string()),
        ]);
        rows.extend(quality.issues.iter().map(|issue| {
            vec![
                issue.label.clone(),
                format!("{} 项", issue.count),
                join_numbers(&issue.sample_indexes),
            ]
        }));
    }
    DocumentSection {
        title: "启动前校验".to_string(),
        paragraphs,
        rows,
    }
}

fn diagnostics_section(detail: &ReportDetail) -> DocumentSection {
    let Some(result) = &detail.diagnostics_snapshot else {
        return DocumentSection {
            title: "兼容性诊断附录".to_string(),
            paragraphs: Vec::new(),
            rows: Vec::new(),
        };
    };
    DocumentSection {
        title: "兼容性诊断附录".to_string(),
        paragraphs: result.recommendations.clone(),
        rows: std::iter::once(vec![
            "端点".to_string(),
            "方法".to_string(),
            "结果".to_string(),
            "HTTP".to_string(),
            "说明".to_string(),
        ])
        .chain(result.endpoints.iter().map(|endpoint| {
            vec![
                endpoint.path.clone(),
                endpoint.method.clone(),
                if endpoint.ok { "通过" } else { "失败" }.to_string(),
                endpoint
                    .http_status
                    .map(|status| status.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                endpoint.message.clone(),
            ]
        }))
        .collect(),
    }
}

fn build_trend_svg(detail: &ReportDetail) -> Option<String> {
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

fn trend_rows(detail: &ReportDetail) -> Vec<Vec<String>> {
    let mut rows = vec![vec![
        "时间".to_string(),
        "QPS".to_string(),
        "P95".to_string(),
        "TTFT".to_string(),
        "TPS".to_string(),
        "成功率".to_string(),
    ]];
    let sample_count = detail.trends.len().min(12);
    let skip = if sample_count == 0 {
        1
    } else {
        (detail.trends.len() / sample_count).max(1)
    };
    rows.extend(detail.trends.iter().step_by(skip).take(12).map(|tick| {
        vec![
            format!("{}s", tick.elapsed_seconds),
            format!("{:.2}", tick.qps),
            format!("{}ms", tick.latency_ms),
            format!("{}ms", tick.ttft_ms),
            format!("{:.2}", tick.tps),
            format!("{:.2}%", tick.success_rate),
        ]
    }));
    rows
}

fn summary_rows(summary: &[(String, String)]) -> Vec<Vec<String>> {
    std::iter::once(vec!["摘要项".to_string(), "值".to_string()])
        .chain(
            summary
                .iter()
                .map(|(label, value)| vec![label.clone(), value.clone()]),
        )
        .collect()
}

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

fn format_metric_value(value: &serde_json::Value, unit: Option<&str>) -> String {
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

fn join_numbers(values: &[i64]) -> String {
    if values.is_empty() {
        return "-".to_string();
    }
    values
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" -> ")
}

fn request_log_meta_text(total_records: i64, body_records: i64) -> String {
    if total_records <= 0 {
        return "未采集".to_string();
    }
    if body_records > 0 {
        format!("{total_records} 条索引，{body_records} 条正文可用")
    } else {
        format!("{total_records} 条索引，未保存正文")
    }
}

fn request_log_appendix_text(detail: &ReportDetail) -> String {
    if detail.request_log_meta.total_records <= 0 {
        return "本次报告未记录请求级明细。".to_string();
    }
    format!(
        "本次报告记录请求级明细索引 {} 条，其中正文可用 {} 条；单条详情请在 MyLLMBenchmark 客户端内查看。",
        detail.request_log_meta.total_records, detail.request_log_meta.body_records
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_xml(value: &str) -> String {
    escape_html(value).replace('\'', "&apos;")
}

fn push_docx_heading(xml: &mut String, text: &str, style: &str) {
    xml.push_str(r#"<w:p><w:pPr><w:pStyle w:val=""#);
    xml.push_str(style);
    xml.push_str(r#""/></w:pPr><w:r><w:t>"#);
    xml.push_str(&escape_xml(text));
    xml.push_str("</w:t></w:r></w:p>");
}

fn push_docx_paragraph(xml: &mut String, text: &str) {
    xml.push_str("<w:p><w:r><w:t>");
    xml.push_str(&escape_xml(text));
    xml.push_str("</w:t></w:r></w:p>");
}

fn push_docx_table(xml: &mut String, rows: &[Vec<String>]) {
    if rows.is_empty() {
        return;
    }
    xml.push_str(
        r#"<w:tbl><w:tblPr><w:tblStyle w:val="TableGrid"/><w:tblW w:w="0" w:type="auto"/><w:tblBorders><w:top w:val="single" w:sz="4" w:color="D8CEC6"/><w:left w:val="single" w:sz="4" w:color="D8CEC6"/><w:bottom w:val="single" w:sz="4" w:color="D8CEC6"/><w:right w:val="single" w:sz="4" w:color="D8CEC6"/><w:insideH w:val="single" w:sz="4" w:color="E6DDD6"/><w:insideV w:val="single" w:sz="4" w:color="E6DDD6"/></w:tblBorders></w:tblPr>"#,
    );
    for (row_index, row) in rows.iter().enumerate() {
        xml.push_str("<w:tr>");
        for cell in row {
            xml.push_str("<w:tc><w:tcPr><w:tcW w:w=\"2400\" w:type=\"dxa\"/>");
            if row_index == 0 {
                xml.push_str(r#"<w:shd w:fill="F2EBE5"/>"#);
            }
            xml.push_str("</w:tcPr><w:p><w:r>");
            if row_index == 0 {
                xml.push_str("<w:rPr><w:b/></w:rPr>");
            }
            xml.push_str("<w:t>");
            xml.push_str(&escape_xml(cell));
            xml.push_str("</w:t></w:r></w:p></w:tc>");
        }
        xml.push_str("</w:tr>");
    }
    xml.push_str("</w:tbl>");
}

fn build_simple_pdf(lines: &[String]) -> Vec<u8> {
    let wrapped = wrap_pdf_lines(lines, 56);
    let pages = wrapped
        .chunks(42)
        .map(|chunk| chunk.to_vec())
        .collect::<Vec<_>>();
    let pages = if pages.is_empty() {
        vec![vec!["MyLLMBenchmark 压测报告".to_string()]]
    } else {
        pages
    };
    let page_count = pages.len();
    let font_id = 3 + page_count * 2;
    let descendant_font_id = font_id + 1;
    let page_refs = (0..page_count)
        .map(|index| format!("{} 0 R", 3 + index))
        .collect::<Vec<_>>()
        .join(" ");

    let mut objects = vec![
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        format!("<< /Type /Pages /Kids [{page_refs}] /Count {page_count} >>"),
    ];
    for index in 0..page_count {
        let content_id = 3 + page_count + index;
        objects.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 {font_id} 0 R >> >> /Contents {content_id} 0 R >>"
        ));
    }
    for (index, page_lines) in pages.iter().enumerate() {
        let content = pdf_page_content(page_lines, index + 1, page_count);
        objects.push(format!(
            "<< /Length {} >>\nstream\n{}\nendstream",
            content.len(),
            content
        ));
    }
    objects.push(format!(
        "<< /Type /Font /Subtype /Type0 /BaseFont /STSong-Light /Encoding /UniGB-UCS2-H /DescendantFonts [{descendant_font_id} 0 R] >>"
    ));
    objects.push(
        "<< /Type /Font /Subtype /CIDFontType0 /BaseFont /STSong-Light /CIDSystemInfo << /Registry (Adobe) /Ordering (GB1) /Supplement 2 >> >>"
            .to_string(),
    );

    let mut pdf = b"%PDF-1.4\n".to_vec();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.extend_from_slice(format!("{} 0 obj\n{}\nendobj\n", index + 1, object).as_bytes());
    }
    let xref_offset = pdf.len();
    pdf.extend_from_slice(
        format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes(),
    );
    for offset in offsets {
        pdf.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    pdf.extend_from_slice(
        format!(
            "trailer << /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
            objects.len() + 1,
            xref_offset
        )
        .as_bytes(),
    );
    pdf
}

fn wrap_pdf_lines(lines: &[String], max_chars: usize) -> Vec<String> {
    let mut wrapped = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            wrapped.push(String::new());
            continue;
        }
        let chars = line.chars().collect::<Vec<_>>();
        for chunk in chars.chunks(max_chars) {
            wrapped.push(chunk.iter().collect());
        }
    }
    wrapped
}

fn pdf_page_content(lines: &[String], page: usize, total: usize) -> String {
    let mut content = String::from("BT /F1 9 Tf 50 805 Td 13 TL ");
    content.push('<');
    content.push_str(&utf16be_hex("MyLLMBenchmark 压测报告"));
    content.push_str("> Tj T* ");
    content.push_str("0 -8 Td ");
    for line in lines {
        content.push('<');
        content.push_str(&utf16be_hex(line));
        content.push_str("> Tj T* ");
    }
    content.push_str("ET BT /F1 8 Tf 50 34 Td ");
    content.push('<');
    content.push_str(&utf16be_hex(&format!("第 {page} / {total} 页")));
    content.push_str("> Tj ET");
    content
}

fn utf16be_hex(value: &str) -> String {
    let mut hex = String::from("FEFF");
    for unit in value.encode_utf16() {
        hex.push_str(&format!("{unit:04X}"));
    }
    hex
}

fn build_docx_package(document_xml: &str) -> Vec<u8> {
    let files = vec![
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#.as_bytes().to_vec(),
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#.as_bytes().to_vec(),
        ),
        ("word/document.xml", document_xml.as_bytes().to_vec()),
    ];
    build_store_zip(files)
}

fn build_store_zip(files: Vec<(&str, Vec<u8>)>) -> Vec<u8> {
    let mut zip = Vec::new();
    let mut central = Vec::new();
    for (name, data) in files {
        let offset = zip.len() as u32;
        let crc = crc32(&data);
        let name_bytes = name.as_bytes();
        write_u32(&mut zip, 0x0403_4b50);
        write_u16(&mut zip, 20);
        write_u16(&mut zip, 0);
        write_u16(&mut zip, 0);
        write_u16(&mut zip, 0);
        write_u16(&mut zip, 0);
        write_u32(&mut zip, crc);
        write_u32(&mut zip, data.len() as u32);
        write_u32(&mut zip, data.len() as u32);
        write_u16(&mut zip, name_bytes.len() as u16);
        write_u16(&mut zip, 0);
        zip.extend_from_slice(name_bytes);
        zip.extend_from_slice(&data);

        write_u32(&mut central, 0x0201_4b50);
        write_u16(&mut central, 20);
        write_u16(&mut central, 20);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u32(&mut central, crc);
        write_u32(&mut central, data.len() as u32);
        write_u32(&mut central, data.len() as u32);
        write_u16(&mut central, name_bytes.len() as u16);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u16(&mut central, 0);
        write_u32(&mut central, 0);
        write_u32(&mut central, offset);
        central.extend_from_slice(name_bytes);
    }
    let central_offset = zip.len() as u32;
    let central_size = central.len() as u32;
    let file_count = count_central_entries(&central);
    zip.extend_from_slice(&central);
    write_u32(&mut zip, 0x0605_4b50);
    write_u16(&mut zip, 0);
    write_u16(&mut zip, 0);
    write_u16(&mut zip, file_count);
    write_u16(&mut zip, file_count);
    write_u32(&mut zip, central_size);
    write_u32(&mut zip, central_offset);
    write_u16(&mut zip, 0);
    zip
}

fn count_central_entries(central: &[u8]) -> u16 {
    central
        .windows(4)
        .filter(|window| *window == [0x50, 0x4b, 0x01, 0x02])
        .count() as u16
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn write_u16(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::{build_docx_package, build_simple_pdf, DocumentSection, ReportTemplate};

    #[test]
    fn simple_pdf_has_pdf_header_and_eof_marker() {
        let bytes = build_simple_pdf(&["MyLLMBenchmark 测试报告".to_string()]);

        assert!(bytes.starts_with(b"%PDF-1.4"));
        assert!(bytes.ends_with(b"%%EOF"));
    }

    #[test]
    fn simple_pdf_creates_multiple_page_objects_for_long_reports() {
        let lines = (0..120)
            .map(|index| format!("第 {index} 行容量证据"))
            .collect::<Vec<_>>();
        let bytes = build_simple_pdf(&lines);
        let page_count = bytes
            .windows(b"/Type /Page /Parent".len())
            .filter(|window| *window == b"/Type /Page /Parent")
            .count();

        assert!(page_count >= 2);
    }

    #[test]
    fn docx_package_is_a_zip_with_word_document_part() {
        let bytes = build_docx_package(
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:tbl><w:tr><w:tc><w:p><w:r><w:t>MyLLMBenchmark</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:body></w:document>"#,
        );

        assert!(bytes.starts_with(b"PK\x03\x04"));
        assert!(bytes
            .windows("word/document.xml".len())
            .any(|window| { window == "word/document.xml".as_bytes() }));
        assert!(bytes
            .windows("<w:tbl>".len())
            .any(|window| { window == "<w:tbl>".as_bytes() }));
    }

    #[test]
    fn report_templates_filter_sections_differently() {
        let sections = vec![
            section("执行摘要"),
            section("测试配置"),
            section("阶段证据"),
            section("错误分布"),
            section("上线建议"),
            section("附录"),
        ];

        let summary = ReportTemplate::DeliverySummary.filter_sections(sections.clone());
        let audit = ReportTemplate::DetailedAudit.filter_sections(sections);

        assert_eq!(summary.len(), 3);
        assert_eq!(audit.len(), 6);
    }

    fn section(title: &str) -> DocumentSection {
        DocumentSection {
            title: title.to_string(),
            paragraphs: Vec::new(),
            rows: Vec::new(),
        }
    }
}
