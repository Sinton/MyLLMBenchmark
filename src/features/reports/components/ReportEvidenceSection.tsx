import { Badge } from "../../../components/common/Badge";
import { Card } from "../../../components/common/Card";
import { AlertTriangle, ShieldCheck } from "../../../components/common/icons";
import type { ReportDetail } from "../../../types/api";
import { InfoPill } from "./InfoPill";

type ReportEvidenceSectionProps = {
  detail: ReportDetail;
};

export function ReportEvidenceSection({ detail }: ReportEvidenceSectionProps) {
  const diagnostics = detail.diagnostics_snapshot;
  const datasetQuality = detail.dataset_quality;
  const preflight = detail.preflight_result;

  if (!diagnostics && !datasetQuality && !preflight) {
    return null;
  }

  return (
    <Card title="证据链">
      <div className="report-evidence-grid">
        {preflight && (
          <section className="report-evidence-card">
            <div className="report-evidence-card-header">
              <h4>启动前校验</h4>
              <Badge tone={preflight.status === "passed" ? "success" : "warning"}>
                {String(preflight.status)}
              </Badge>
            </div>
            <p className="report-section-copy">
              {Array.isArray((preflight as { warnings?: unknown }).warnings) &&
              ((preflight as { warnings?: string[] }).warnings ?? []).length > 0
                ? ((preflight as { warnings?: string[] }).warnings ?? []).join("；")
                : "未发现明显阻断项。"}
            </p>
          </section>
        )}

        {datasetQuality && (
          <section className="report-evidence-card">
            <div className="report-evidence-card-header">
              <h4>数据集质量</h4>
              <Badge tone={datasetQuality.status === "passed" ? "success" : "warning"}>
                {datasetQuality.status}
              </Badge>
            </div>
            <p className="report-section-copy">
              {datasetQuality.recommendations[0] ?? "已完成样本质量检查。"}
            </p>
            <div className="report-evidence-metrics">
              <InfoPill icon={<ShieldCheck size={14} />} label="样本数" value={`${datasetQuality.sample_count}`} />
              <InfoPill
                icon={<AlertTriangle size={14} />}
                label="问题类型"
                value={`${datasetQuality.issues.length} 类`}
              />
            </div>
          </section>
        )}

        {diagnostics && (
          <section className="report-evidence-card">
            <div className="report-evidence-card-header">
              <h4>兼容性诊断</h4>
              <Badge tone={diagnostics.status === "passed" ? "success" : "warning"}>
                {diagnostics.status}
              </Badge>
            </div>
            <p className="report-section-copy">
              {diagnostics.recommendations[0] ?? "已完成端点诊断。"}
            </p>
            <div className="report-evidence-metrics">
              <InfoPill
                icon={<ShieldCheck size={14} />}
                label="端点数"
                value={`${diagnostics.endpoints.length}`}
              />
              <InfoPill
                icon={<AlertTriangle size={14} />}
                label="告警项"
                value={`${diagnostics.endpoints.filter((endpoint) => !endpoint.ok).length}`}
              />
            </div>
          </section>
        )}
      </div>
    </Card>
  );
}
