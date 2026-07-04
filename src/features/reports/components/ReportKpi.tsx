import { MetricHelp } from "../../../components/common/MetricHelp";

type ReportKpiProps = {
  label: string;
  helpKey?: string;
  value: string | number;
  unit: string;
  hint: string;
};

export function ReportKpi({ label, helpKey, value, unit, hint }: ReportKpiProps) {
  return (
    <div className="report-kpi">
      <span>
        <MetricHelp helpKey={helpKey}>{label}</MetricHelp>
      </span>
      <strong>
        {value}
        {unit && <em>{unit}</em>}
      </strong>
      <small>{hint}</small>
    </div>
  );
}
