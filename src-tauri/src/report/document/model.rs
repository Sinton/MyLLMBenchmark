use crate::models::ReportDetail;

use super::html::build_trend_svg;
use super::utils::{
    format_metric_value, join_numbers, request_log_appendix_text, request_log_meta_text,
    ttft_source_text,
};

pub(crate) struct ReportDocument {
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) source_label: String,
    pub(crate) template: ReportTemplate,
    pub(crate) summary: Vec<(String, String)>,
    pub(crate) sections: Vec<DocumentSection>,
    pub(crate) trend_svg: Option<String>,
}

#[derive(Clone)]
pub(crate) struct DocumentSection {
    pub(crate) title: String,
    pub(crate) paragraphs: Vec<String>,
    pub(crate) rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReportTemplate {
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
                "TTFT 口径".to_string(),
                ttft_source_text(&detail.ttft_source).to_string(),
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
                    vec![
                        "TTFT 口径".to_string(),
                        ttft_source_text(&detail.ttft_source).to_string(),
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
                    "共记录 {} 个实时指标点，末次稳定 QPS {:.2}，TTFT {}ms（{}），输出/专项吞吐 {:.2}。",
                    detail.trends.len(),
                    detail.stable_qps,
                    detail.ttft_ms,
                    ttft_source_text(&detail.ttft_source),
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
                        "TTFT 口径".to_string(),
                        ttft_source_text(&detail.ttft_source).to_string(),
                    ],
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

    pub(crate) fn as_plain_lines(&self) -> Vec<String> {
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
    pub(crate) fn from_label(value: Option<&str>) -> Self {
        match value.unwrap_or("").trim() {
            "运维容量版" => Self::OperationsCapacity,
            "详细审计版" => Self::DetailedAudit,
            _ => Self::DeliverySummary,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::DeliverySummary => "交付摘要版",
            Self::OperationsCapacity => "运维容量版",
            Self::DetailedAudit => "详细审计版",
        }
    }

    pub(crate) fn filter_sections(self, sections: Vec<DocumentSection>) -> Vec<DocumentSection> {
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

pub(crate) fn summary_rows(summary: &[(String, String)]) -> Vec<Vec<String>> {
    std::iter::once(vec!["摘要项".to_string(), "值".to_string()])
        .chain(
            summary
                .iter()
                .map(|(label, value)| vec![label.clone(), value.clone()]),
        )
        .collect()
}
