import { Badge } from "../../../components/ui/Badge";
import { Card } from "../../../components/ui/Card";
import { FileText } from "../../../components/ui/icons";
import type { ReportSummary } from "../../../types/api";
import { formatDate } from "../domain/reportDefinitions";

type ReportRailProps = {
  reports: ReportSummary[];
  selectedId?: string;
  onSelect: (id: string) => void;
};

export function ReportRail({ reports, selectedId, onSelect }: ReportRailProps) {
  return (
    <Card className="reports-rail" title="报告列表">
      <div className="report-list">
        {reports.map((report) => (
          <button
            className={`report-list-item ${selectedId === report.id ? "active" : ""}`}
            key={report.id}
            onClick={() => onSelect(report.id)}
            type="button"
          >
            <span className="report-list-icon">
              <FileText size={16} />
            </span>
            <span className="report-list-main">
              <strong>{report.model_name}</strong>
              <span>{report.provider_name}</span>
              <em>{formatDate(report.created_at)}</em>
            </span>
            <Badge tone={report.success_rate >= 99 ? "success" : "warning"}>
              {report.success_rate >= 99 ? "稳定" : "关注"}
            </Badge>
          </button>
        ))}
      </div>
    </Card>
  );
}
