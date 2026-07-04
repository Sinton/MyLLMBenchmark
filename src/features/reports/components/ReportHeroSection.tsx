import { Badge } from "../../../components/common/Badge";
import { CheckCircle2, Gauge, ShieldCheck } from "../../../components/common/icons";
import type { ReportDetail } from "../../../types/api";
import { getModelTypeLabel } from "../domain/reportDefinitions";
import { InfoPill } from "./InfoPill";

type ReportHeroSectionProps = {
  detail: ReportDetail;
};

export function ReportHeroSection({ detail }: ReportHeroSectionProps) {
  const sourceMeta = getReportSourceMeta(detail.source);

  return (
    <section className="report-hero">
      <div className="report-hero-main">
        <div className="report-title-row">
          <div className="report-doc-icon">
            <ShieldCheck size={22} />
          </div>
          <div>
            <p className="eyebrow">LLM Capacity Assessment</p>
            <h2>{detail.summary.model_name}</h2>
            <div className="report-meta">
              <span>{detail.summary.provider_name}</span>
              <span>{detail.dataset_name}</span>
              <span>{getModelTypeLabel(detail.model_type)}</span>
              <span>{sourceMeta.metaLabel}</span>
            </div>
          </div>
        </div>
        <p className="report-conclusion">{detail.capacity_conclusion}</p>
        <div className="report-verdict-strip">
          <InfoPill icon={<Gauge size={15} />} label="主要瓶颈" value={detail.bottleneck} />
          <InfoPill
            icon={<CheckCircle2 size={15} />}
            label="SLA"
            value={`P95 <= ${detail.sla_p95_ms}ms / 成功率 >= ${detail.min_success_rate}%`}
          />
          <InfoPill
            icon={<Gauge size={15} />}
            label="请求超时"
            value={`${detail.request_timeout_seconds || "-"}s`}
          />
          <InfoPill
            icon={<CheckCircle2 size={15} />}
            label="失败策略"
            value={
              detail.sla_stop_policy === "stop_on_failure"
                ? "保护性停止"
                : "继续完整阶梯"
            }
          />
        </div>
      </div>
      <div className="report-score-card">
        <div className={`verdict-panel verdict-${detail.verdict}`}>
          <strong>{detail.verdict_label}</strong>
          <span>{detail.verdict === "pass" ? "建议按限流上线" : "上线前需要复核"}</span>
        </div>
        <Badge tone={sourceMeta.tone}>
          {sourceMeta.badgeLabel}
        </Badge>
      </div>
    </section>
  );
}

function getReportSourceMeta(source: ReportDetail["source"]) {
  if (source === "measured") {
    return {
      metaLabel: "真实接口实测数据",
      badgeLabel: "真实过程数据",
      tone: "success" as const,
    };
  }

  if (source === "mock") {
    return {
      metaLabel: "Mock 引擎过程数据",
      badgeLabel: "模拟过程数据",
      tone: "info" as const,
    };
  }

  return {
    metaLabel: "摘要估算数据",
    badgeLabel: "历史兼容",
    tone: "warning" as const,
  };
}
