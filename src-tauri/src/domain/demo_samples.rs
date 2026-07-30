pub fn build_chat_prompts() -> Vec<String> {
    let themes = [
        "政务服务大厅智能问答",
        "银行网点客户经理助手",
        "运营商客服知识库",
        "央企设备巡检报告",
        "合同条款风险识别",
        "招投标文件摘要",
        "内部制度问答",
        "数据治理方案评审",
    ];
    let tasks = [
        "请用面向业务负责人的语言总结核心结论，并给出三条落地建议。",
        "请识别潜在风险点，按高、中、低三个等级输出。",
        "请把复杂技术内容改写成售前汇报口径，要求稳健、专业。",
        "请生成一段适合客户会议使用的答复，不要夸大模型能力。",
        "请列出需要补充确认的信息，并说明这些信息会影响哪些判断。",
        "请把输入内容整理成结构化清单，保留关键数字和约束条件。",
        "请给出容量评估口径，说明推荐并发、稳定并发和风险边界。",
        "请生成一份复测建议，覆盖样本长度、并发阶梯和错误观察点。",
        "请用简洁中文解释 Transformer、Embedding、Rerank 在该场景中的区别。",
        "请根据国企内网部署约束，给出上线前检查项。",
        "请把下面内容改写为运维交接文档，强调可观测性和回滚方案。",
        "请模拟客户质疑并给出稳妥回复，避免承诺无法验证的性能指标。",
        "请提取关键实体、时间、责任部门和后续动作。",
        "请生成面向领导汇报的一页纸摘要，包含背景、结论、风险和建议。",
        "请判断当前需求更适合文本生成、向量检索、重排序还是视觉模型。",
        "请根据压测报告指标解释 TTFT、P95、TPS、成功率之间的关系。",
    ];

    themes
        .iter()
        .flat_map(|theme| tasks.iter().map(move |task| format!("{theme}：{task}")))
        .collect()
}

pub fn build_embedding_prompts() -> Vec<String> {
    let domains = [
        ("客户服务", "覆盖工单受理、问题定位、升级流转和回访闭环。"),
        ("合同管理", "覆盖条款审查、履约节点、风险提示和归档要求。"),
        ("设备运维", "覆盖巡检计划、告警处置、备件更换和复盘记录。"),
        ("数据治理", "覆盖数据标准、质量规则、血缘关系和责任边界。"),
        ("信息安全", "覆盖账号权限、漏洞响应、审计留痕和应急处置。"),
        ("采购管理", "覆盖需求申报、供应商评估、验收付款和异常处理。"),
        ("人力资源", "覆盖入转调离、培训发展、绩效反馈和制度咨询。"),
        ("项目交付", "覆盖范围确认、里程碑、变更控制和上线验收。"),
    ];
    let topics = [
        ("适用范围", "说明该知识条目适用的组织、角色和业务场景。"),
        ("办理条件", "列出执行前必须满足的资料、权限和系统状态。"),
        (
            "标准流程",
            "按照发起、审核、执行、确认四个阶段描述操作步骤。",
        ),
        ("时限要求", "明确响应时限、完成时限以及超时后的升级路径。"),
        ("异常处理", "给出常见异常、排查顺序和需要保留的证据。"),
        ("责任分工", "区分申请人、审批人、执行人和监督人的职责。"),
        ("质量检查", "提供可复核的检查项、通过条件和整改方式。"),
        ("常见问答", "汇总高频疑问，并给出不超出制度边界的标准答复。"),
    ];

    domains
        .iter()
        .flat_map(|(domain, scope)| {
            topics.iter().map(move |(topic, detail)| {
                format!(
                    "{domain}知识库｜{topic}。{scope}{detail}检索时应同时关注业务对象、当前状态、时间要求和责任角色，信息冲突时以最新生效制度及审批记录为准。"
                )
            })
        })
        .collect()
}

pub fn build_rerank_prompts() -> Vec<String> {
    let scenarios = [
        "客户反馈线上业务无法提交",
        "合同即将到期但尚未完成续签",
        "生产设备出现间歇性告警",
        "数据报表口径与源系统不一致",
        "员工账号存在越权访问风险",
        "供应商交付物未通过验收",
        "项目上线窗口需要临时调整",
        "知识库检索结果与问题无关",
    ];
    let evidence_types = [
        "处理流程",
        "适用条件",
        "排查记录",
        "制度条款",
        "操作手册",
        "历史案例",
        "风险提示",
        "验收标准",
    ];

    scenarios
        .iter()
        .flat_map(|scenario| {
            evidence_types.iter().map(move |evidence_type| {
                format!(
                    "候选资料｜{evidence_type}：针对“{scenario}”，应先确认业务对象、发生时间、系统状态和已有处置记录，再依据现行流程判断责任环节；证据不足时不得直接给出确定性结论。"
                )
            })
        })
        .collect()
}

pub fn build_vision_prompts() -> Vec<String> {
    let image_ids = [0, 10, 20, 28, 42, 48, 60, 96, 180, 201, 250, 366];
    let tasks = [
        "请描述图片中的主要对象、场景布局和可见文字；无法确认的信息请明确标注。",
        "请从业务巡检角度列出图片中的可观察事实、潜在异常和建议复核项。",
    ];

    image_ids
        .iter()
        .flat_map(|image_id| {
            tasks.iter().map(move |prompt| {
                serde_json::json!({
                    "prompt": prompt,
                    "image_url": format!("https://picsum.photos/id/{image_id}/1024/768")
                })
                .to_string()
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{build_embedding_prompts, build_rerank_prompts, build_vision_prompts};

    #[test]
    fn model_specific_demo_samples_are_non_empty_and_well_formed() {
        let embedding = build_embedding_prompts();
        let rerank = build_rerank_prompts();
        let vision = build_vision_prompts();

        assert_eq!(embedding.len(), 64);
        assert_eq!(rerank.len(), 64);
        assert_eq!(vision.len(), 24);
        assert!(embedding.iter().all(|sample| !sample.trim().is_empty()));
        assert!(rerank.iter().all(|sample| !sample.trim().is_empty()));
        assert!(vision.iter().all(|sample| {
            serde_json::from_str::<serde_json::Value>(sample)
                .ok()
                .and_then(|value| value.get("image_url").cloned())
                .and_then(|value| value.as_str().map(str::to_string))
                .is_some_and(|url| url.starts_with("https://"))
        }));
    }
}
