import type { ChartMetric } from "../domain/modelMetrics";

export const chartMetricMeta: Record<
  ChartMetric,
  { name: string; color: string; unit: string }
> = {
  latency: { name: "Latency P95", color: "#91593C", unit: "ms" },
  ttft: { name: "TTFT", color: "#8B6F5A", unit: "ms" },
  qps: { name: "QPS", color: "#6D9E95", unit: "" },
  tps: { name: "TPS", color: "#D98A5B", unit: "" },
  success: { name: "Success", color: "#6D9E95", unit: "%" },
  errors: { name: "Errors", color: "#B85C5C", unit: "" },
};

export const chartTheme = {
  axis: "#D6CAC0",
  label: "#8C8075",
  splitLine: "#E6E0D9",
  sla: "#D98A5B",
};
