import type { BenchmarkTaskSummary } from "./benchmark";
import type { ReportSummary } from "./report";

export type DashboardSummary = {
  providers: number;
  models: number;
  tasks: number;
  reports: number;
  recent_tasks: BenchmarkTaskSummary[];
  recent_reports: ReportSummary[];
};
