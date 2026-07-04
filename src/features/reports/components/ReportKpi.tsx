type ReportKpiProps = {
  label: string;
  value: string | number;
  unit: string;
  hint: string;
};

export function ReportKpi({ label, value, unit, hint }: ReportKpiProps) {
  return (
    <div className="report-kpi">
      <span>{label}</span>
      <strong>
        {value}
        {unit && <em>{unit}</em>}
      </strong>
      <small>{hint}</small>
    </div>
  );
}
