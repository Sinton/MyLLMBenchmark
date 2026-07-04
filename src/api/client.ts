import { invoke } from "@tauri-apps/api/core";
import { listenToEvent } from "./events";
import type { AppApi } from "./types";
import type {
  AppConfig,
  BenchmarkRequestLogDetail,
  BenchmarkRequestLogPage,
  BenchmarkRequestLogPageInput,
  BenchmarkTaskSummary,
  ConfigUpdateResult,
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
  ModelSummary,
  MetricsTick,
  ProviderConnectionResult,
  ProviderDiagnosticsInput,
  ProviderDiagnosticsResult,
  ProviderModelScanResult,
  ProviderSummary,
  ReportDetail,
  ReportExportInput,
  ReportExportResult,
  ReportSummary,
  StopResult,
} from "../types/api";

export const api: AppApi = {
  getAppConfig: () => invoke<AppConfig>("get_app_config"),
  updateAppConfig: (config) =>
    invoke<ConfigUpdateResult>("update_app_config", { config }),
  getDashboardSummary: () => invoke<DashboardSummary>("get_dashboard_summary"),
  listProviders: () => invoke<ProviderSummary[]>("list_providers"),
  createProvider: (input) => invoke<ProviderSummary>("create_provider", { input }),
  updateProvider: (providerId, input) =>
    invoke<ProviderSummary>("update_provider", { providerId, input }),
  deleteProvider: (providerId) =>
    invoke<DeleteResult>("delete_provider", { providerId }),
  testProviderConnection: (providerId) =>
    invoke<ProviderConnectionResult>("test_provider_connection", { providerId }),
  diagnoseProvider: (input: ProviderDiagnosticsInput) =>
    invoke<ProviderDiagnosticsResult>("diagnose_provider", { input }),
  getProviderDiagnostics: (providerId) =>
    invoke<ProviderDiagnosticsResult | null>("get_provider_diagnostics", { providerId }),
  listProviderModels: (providerId) =>
    invoke<ModelSummary[]>("list_provider_models", { providerId }),
  scanProviderModels: (providerId) =>
    invoke<ProviderModelScanResult>("scan_provider_models", { providerId }),
  listDatasets: () => invoke<DatasetSummary[]>("list_datasets"),
  importDataset: (input: DatasetImportInput) =>
    invoke<DatasetSummary>("import_dataset", { input }),
  updateDataset: (input: DatasetUpdateInput) =>
    invoke<DatasetSummary>("update_dataset", { input }),
  deleteDataset: (datasetId) =>
    invoke<DeleteResult>("delete_dataset", { datasetId }),
  previewDatasetSamples: (datasetId, limit) =>
    invoke<DatasetSamplePreview[]>("preview_dataset_samples", { datasetId, limit }),
  listDatasetSamplesPage: (input: DatasetSamplePageInput) =>
    invoke<DatasetSamplePage>("list_dataset_samples_page", { input }),
  createDatasetSample: (input: DatasetSampleCreateInput) =>
    invoke<DatasetSamplePreview>("create_dataset_sample", { input }),
  updateDatasetSample: (input: DatasetSampleUpdateInput) =>
    invoke<DatasetSamplePreview>("update_dataset_sample", { input }),
  deleteDatasetSample: (sampleId) =>
    invoke<DeleteResult>("delete_dataset_sample", { sampleId }),
  appendDatasetSamples: (input: DatasetAppendInput) =>
    invoke<DatasetSummary>("append_dataset_samples", { input }),
  deleteDatasetSamplesBatch: (input: DatasetSampleBatchDeleteInput) =>
    invoke<DeleteResult>("delete_dataset_samples_batch", { input }),
  exportDataset: (input: DatasetExportInput) =>
    invoke<DatasetExportResult>("export_dataset", { input }),
  validateDatasetSamples: (datasetId) =>
    invoke<DatasetValidationResult>("validate_dataset_samples", { datasetId }),
  startBenchmark: (input) =>
    invoke<BenchmarkTaskSummary>("start_benchmark", { input }),
  stopBenchmark: (taskId) => invoke<StopResult>("stop_benchmark", { taskId }),
  getBenchmarkTask: (taskId) =>
    invoke<BenchmarkTaskSummary>("get_benchmark_task", { taskId }),
  listBenchmarkTicks: (taskId) =>
    invoke<MetricsTick[]>("list_benchmark_ticks", { taskId }),
  listBenchmarkRequestLogsPage: (input: BenchmarkRequestLogPageInput) =>
    invoke<BenchmarkRequestLogPage>("list_benchmark_request_logs_page", { input }),
  getBenchmarkRequestLogDetail: (requestId) =>
    invoke<BenchmarkRequestLogDetail>("get_benchmark_request_log_detail", { requestId }),
  deleteBenchmarkRequestLogs: (taskId) =>
    invoke<DeleteResult>("delete_benchmark_request_logs", { taskId }),
  generateReport: (taskId) =>
    invoke<ReportSummary>("generate_report", { taskId }),
  listReports: () => invoke<ReportSummary[]>("list_reports"),
  getReportDetail: (reportId) =>
    invoke<ReportDetail>("get_report_detail", { reportId }),
  exportReport: (input: ReportExportInput) =>
    invoke<ReportExportResult>("export_report", { input }),
};

export { listenToEvent };
