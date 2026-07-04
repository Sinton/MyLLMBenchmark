import { ReportKpi } from "./ReportKpi";

type ReportKpiItem = {
  label: string;
  helpKey?: string;
  value: string | number;
  unit?: string;
  hint?: string;
};

type ReportKpiGridProps = {
  kpis: ReportKpiItem[];
};

export function ReportKpiGrid({ kpis }: ReportKpiGridProps) {
  return (
    <div className="report-kpi-grid">
      {kpis.map((metric) => (
        <ReportKpi
          key={metric.label}
          label={metric.label}
          helpKey={metric.helpKey}
          value={metric.value}
          unit={metric.unit ?? ""}
          hint={metric.hint ?? ""}
        />
      ))}
    </div>
  );
}
