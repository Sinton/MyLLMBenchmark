import type { ReportStageSummary } from "../../types/api";
export type { ChartMetric } from "../../domain/modelMetrics";

export type StageColumn = {
  key: string;
  label: string;
  render: (stage: ReportStageSummary) => string | number;
};
