import type {
  AppConfig,
  BenchmarkRequestLogDetail,
  BenchmarkRequestLogPage,
  BenchmarkRequestLogPageInput,
  BenchmarkStartInput,
  BenchmarkTaskSummary,
  ConfigUpdateResult,
  CreateProviderInput,
  DashboardSummary,
  DatasetAppendInput,
  DatasetExportInput,
  DatasetExportResult,
  DatasetImportInput,
  DatasetSampleBatchDeleteInput,
  DatasetSampleCreateInput,
  DatasetSamplePage,
  DatasetSamplePageInput,
  DatasetSamplePreview,
  DatasetSampleUpdateInput,
  DatasetSummary,
  DatasetUpdateInput,
  DatasetValidationResult,
  DeleteResult,
  EndpointProbeBatchDetail,
  EndpointProbeBatchSummary,
  EndpointProbeHistoryPage,
  EndpointProbeHistoryPageInput,
  EndpointProbeModelScanInput,
  EndpointProbeModelScanResult,
  EndpointProbePromotionInput,
  EndpointProbePromotionResult,
  EndpointProbeRunDetail,
  EndpointProbeStartInput,
  EndpointProbeStopResult,
  ModelSummary,
  MetricsTick,
  ProviderConnectionResult,
  ProviderDiagnosticsInput,
  ProviderDiagnosticsResult,
  ProviderImportInput,
  ProviderImportResult,
  ProviderModelScanResult,
  ProviderSummary,
  ReportDetail,
  ReportExportInput,
  ReportExportResult,
  ReportSummary,
  StopResult,
  UpdateProviderInput,
} from "../types/api";

export type AppApi = {
  getAppConfig: () => Promise<AppConfig>;
  updateAppConfig: (config: AppConfig) => Promise<ConfigUpdateResult>;
  getDashboardSummary: () => Promise<DashboardSummary>;
  listProviders: () => Promise<ProviderSummary[]>;
  createProvider: (input: CreateProviderInput) => Promise<ProviderSummary>;
  importProviders: (input: ProviderImportInput) => Promise<ProviderImportResult>;
  updateProvider: (providerId: string, input: UpdateProviderInput) => Promise<ProviderSummary>;
  deleteProvider: (providerId: string) => Promise<DeleteResult>;
  testProviderConnection: (providerId: string) => Promise<ProviderConnectionResult>;
  diagnoseProvider: (input: ProviderDiagnosticsInput) => Promise<ProviderDiagnosticsResult>;
  getProviderDiagnostics: (providerId: string) => Promise<ProviderDiagnosticsResult | null>;
  listProviderModels: (providerId: string) => Promise<ModelSummary[]>;
  scanProviderModels: (providerId: string) => Promise<ProviderModelScanResult>;
  listDatasets: () => Promise<DatasetSummary[]>;
  importDataset: (input: DatasetImportInput) => Promise<DatasetSummary>;
  updateDataset: (input: DatasetUpdateInput) => Promise<DatasetSummary>;
  deleteDataset: (datasetId: string) => Promise<DeleteResult>;
  previewDatasetSamples: (
    datasetId: string,
    limit?: number,
  ) => Promise<DatasetSamplePreview[]>;
  listDatasetSamplesPage: (input: DatasetSamplePageInput) => Promise<DatasetSamplePage>;
  createDatasetSample: (input: DatasetSampleCreateInput) => Promise<DatasetSamplePreview>;
  updateDatasetSample: (input: DatasetSampleUpdateInput) => Promise<DatasetSamplePreview>;
  deleteDatasetSample: (sampleId: string) => Promise<DeleteResult>;
  appendDatasetSamples: (input: DatasetAppendInput) => Promise<DatasetSummary>;
  deleteDatasetSamplesBatch: (input: DatasetSampleBatchDeleteInput) => Promise<DeleteResult>;
  exportDataset: (input: DatasetExportInput) => Promise<DatasetExportResult>;
  validateDatasetSamples: (datasetId: string) => Promise<DatasetValidationResult>;
  startBenchmark: (input: BenchmarkStartInput) => Promise<BenchmarkTaskSummary>;
  stopBenchmark: (taskId: string) => Promise<StopResult>;
  getBenchmarkTask: (taskId: string) => Promise<BenchmarkTaskSummary>;
  listBenchmarkTicks: (taskId: string) => Promise<MetricsTick[]>;
  listBenchmarkRequestLogsPage: (
    input: BenchmarkRequestLogPageInput,
  ) => Promise<BenchmarkRequestLogPage>;
  getBenchmarkRequestLogDetail: (
    requestId: string,
  ) => Promise<BenchmarkRequestLogDetail>;
  deleteBenchmarkRequestLogs: (taskId: string) => Promise<DeleteResult>;
  generateReport: (taskId: string) => Promise<ReportSummary>;
  listReports: () => Promise<ReportSummary[]>;
  getReportDetail: (reportId: string) => Promise<ReportDetail>;
  exportReport: (input: ReportExportInput) => Promise<ReportExportResult>;
  startEndpointProbe: (
    input: EndpointProbeStartInput,
  ) => Promise<EndpointProbeBatchSummary>;
  stopEndpointProbe: (batchId: string) => Promise<EndpointProbeStopResult>;
  scanEndpointProbeModels: (
    input: EndpointProbeModelScanInput,
  ) => Promise<EndpointProbeModelScanResult>;
  promoteEndpointProbeTarget: (
    input: EndpointProbePromotionInput,
  ) => Promise<EndpointProbePromotionResult>;
  listEndpointProbeBatchesPage: (
    input: EndpointProbeHistoryPageInput,
  ) => Promise<EndpointProbeHistoryPage>;
  getEndpointProbeBatchDetail: (
    batchId: string,
  ) => Promise<EndpointProbeBatchDetail>;
  getEndpointProbeRunDetail: (runId: string) => Promise<EndpointProbeRunDetail>;
  deleteEndpointProbeBatch: (batchId: string) => Promise<DeleteResult>;
};
