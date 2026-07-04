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
};
