import type { BenchmarkWorkloadConfig, MetricsTick } from "./benchmark";
import type { DatasetValidationResult } from "./dataset";
import type { ModelType } from "./model";
import type { ProviderDiagnosticsResult } from "./provider";

export type ReportSummary = {
  id: string;
  task_id: string;
  model_name: string;
  provider_name: string;
  recommendation: string;
  recommended_concurrency: number;
  max_stable_concurrency: number;
  p95_latency_ms: number;
  success_rate: number;
  created_at: string;
};

export type ReportStageSummary = {
  stage_index: number;
  concurrency: number;
  sample_rounds: number;
  warmup_rounds: number;
  request_count: number;
  success_count: number;
  failure_count: number;
  qps: number;
  p95_latency_ms: number;
  ttft_ms: number;
  tps: number;
  success_rate: number;
  error_rate: number;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  batch_size: number;
  text_count: number;
  documents_per_query: number;
  pair_count: number;
  image_count: number;
  sla_passed: boolean;
  stop_reason?: string | null;
  status: "stable" | "watch" | "failed";
};

export type ReportErrorBucket = {
  label: string;
  value: number;
  percent: number;
};

export type ReportRequestLogMeta = {
  enabled: boolean;
  total_records: number;
  body_records: number;
  body_available: boolean;
};

export type ReportSpecialtyMetric = {
  label: string;
  value: string | number;
  unit?: string;
  hint: string;
};

export type ReportSpecialtySection = {
  title: string;
  description: string;
  metrics: ReportSpecialtyMetric[];
  guidance: string[];
};

export type ReportDetail = {
  summary: ReportSummary;
  source: "measured" | "estimated" | "mock";
  model_type: ModelType | string;
  task_name: string;
  dataset_name: string;
  mode: string;
  duration_seconds: number;
  planned_stages: number[];
  executed_stages: number[];
  stage_sample_rounds: number;
  warmup_rounds: number;
  request_timeout_seconds: number;
  sla_stop_policy: "continue_full_staircase" | "stop_on_failure" | string;
  early_stop_reason?: string | null;
  sla_p95_ms: number;
  min_success_rate: number;
  verdict: "pass" | "watch" | "fail";
  verdict_label: string;
  bottleneck: string;
  capacity_conclusion: string;
  stable_qps: number;
  ttft_ms: number;
  ttft_source:
    | "streaming_real"
    | "non_streaming_approximation"
    | "historical_estimated"
    | "not_applicable"
    | string;
  tps: number;
  token_throughput: number;
  input_tokens: number;
  output_tokens: number;
  stages: ReportStageSummary[];
  trends: MetricsTick[];
  errors: ReportErrorBucket[];
  specialty: ReportSpecialtySection;
  recommendations: string[];
  workload_config: BenchmarkWorkloadConfig;
  preflight_result:
    | {
        status?: string;
        warnings?: string[];
        model_type?: string;
        dataset_quality?: DatasetValidationResult;
        checked_at?: string;
      }
    | Record<string, unknown>
    | null;
  diagnostics_snapshot: ProviderDiagnosticsResult | null;
  dataset_quality: DatasetValidationResult | null;
  request_log_meta: ReportRequestLogMeta;
};

export type ReportExportInput = {
  report_id: string;
  format: string;
  template?: string;
};

export type ReportExportResult = {
  report_id: string;
  format: string;
  file_name: string;
  file_path: string;
  mime_type: string;
  message: string;
};
