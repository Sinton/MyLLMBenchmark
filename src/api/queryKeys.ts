export const queryKeys = {
  appConfig: () => ["app-config"] as const,
  dashboard: () => ["dashboard"] as const,
  providers: () => ["providers"] as const,
  providerModels: (providerId: string) => ["provider-models", providerId] as const,
  providerDiagnostics: (providerId: string) => ["provider-diagnostics", providerId] as const,
  datasets: () => ["datasets"] as const,
  datasetSamplesRoot: (datasetId: string) => ["dataset-samples", datasetId] as const,
  datasetSamples: (
    datasetId: string,
    page: number,
    pageSize: number,
    keyword: string,
  ) => ["dataset-samples", datasetId, page, pageSize, keyword] as const,
  reports: () => ["reports"] as const,
  reportDetail: (reportId: string) => ["report-detail", reportId] as const,
  benchmarkTask: (taskId: string) => ["benchmark-task", taskId] as const,
  benchmarkTicks: (taskId: string) => ["benchmark-ticks", taskId] as const,
  benchmarkRequestLogs: (
    taskId: string,
    page: number,
    pageSize: number,
    stageIndex: number | undefined,
    status: string,
    keyword: string,
  ) => ["benchmark-request-logs", taskId, page, pageSize, stageIndex ?? "all", status, keyword] as const,
  benchmarkRequestLogDetail: (requestId: string) =>
    ["benchmark-request-log-detail", requestId] as const,
  siteProbeRuns: (page: number, pageSize: number, status: string, keyword: string) =>
    ["site-probe-runs", page, pageSize, status, keyword] as const,
  siteProbeRunDetail: (runId: string) => ["site-probe-run-detail", runId] as const,
};
