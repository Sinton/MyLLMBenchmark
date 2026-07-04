type MetricCardProps = {
  label: string;
  value: string | number;
  unit?: string;
  hint?: string;
};

export function MetricCard({ label, value, unit, hint }: MetricCardProps) {
  return (
    <div className="metric-card">
      <div className="metric-label">{label}</div>
      <div className="metric-value">
        {value}
        {unit && <span>{unit}</span>}
      </div>
      {hint && <div className="metric-hint">{hint}</div>}
    </div>
  );
}

