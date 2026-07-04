import { Card } from "../../../components/common/Card";
import { Sparkles } from "../../../components/common/icons";
import type { ReportDetail } from "../../../types/api";
import { InfoPill } from "./InfoPill";
import { ReportKpi } from "./ReportKpi";

type ReportSpecialtySectionProps = {
  detail: ReportDetail;
};

export function ReportSpecialtySection({ detail }: ReportSpecialtySectionProps) {
  return (
    <Card title={detail.specialty.title}>
      <p className="report-section-copy">{detail.specialty.description}</p>
      <div className="specialty-grid">
        {detail.specialty.metrics.map((metric) => (
          <ReportKpi
            key={metric.label}
            label={metric.label}
            value={metric.value}
            unit={metric.unit ?? ""}
            hint={metric.hint}
          />
        ))}
      </div>
      <div className="guidance-list">
        {detail.specialty.guidance.map((item) => (
          <InfoPill key={item} icon={<Sparkles size={15} />} label="专项建议" value={item} />
        ))}
      </div>
    </Card>
  );
}
