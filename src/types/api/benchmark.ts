import type { ModelType } from "./model";

export type BenchmarkStartInput = {
  provider_id: string;
  model_id?: string | null;
  dataset_id: string;
  mode: string;
  concurrency: number;
  duration_seconds: number;
  start_concurrency?: number;
  end_concurrency?: number;
  step_strategy?: "double" | "linear";
  step_value?: number;
  stage_sample_rounds?: number;
  stage_duration_seconds?: number;
  warmup_rounds?: number;
  warmup_seconds?: number;
  request_timeout_seconds?: number;
  sla_p95_ms?: number;
  min_success_rate?: number;
  sla_stop_policy?: "continue_full_staircase" | "stop_on_failure";
  workload_config?: BenchmarkWorkloadConfig;
  request_log_config?: RequestLogConfig;
};

export type RequestLogConfig = {
  enabled: boolean;
  capture_body: boolean;
  max_records_per_stage: number;
};

export type BenchmarkWorkloadConfig = {
  streaming?: boolean;
  max_output_tokens?: number;
  prompt_profile?: "short" | "mixed" | "long";
  batch_size?: number;
  text_count_per_request?: number;
  documents_per_query?: number;
  top_k?: number;
  image_profile?: "small" | "medium" | "large";
  image_count?: number;
};

export type BenchmarkTaskSummary = {
  id: string;
  name: string;
  status: string;
  model_type: ModelType | string;
  model_name: string;
  provider_name: string;
  dataset_name: string;
  concurrency: number;
  success_rate: number;
  p95_latency_ms: number;
  goodput_qps: number;
  created_at: string;
};

export type StopResult = {
  task_id: string;
  stopped: boolean;
};

export type MetricsTick = {
  task_id: string;
  elapsed_seconds: number;
  qps: number;
  latency_ms: number;
  ttft_ms: number;
  tps: number;
  success_rate: number;
  errors: number;
  in_flight: number;
  request_count: number;
  success_count: number;
  failure_count: number;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  batch_size: number;
  text_count: number;
  documents_per_query: number;
  pair_count: number;
  image_count: number;
};

export type StageChangedEvent = {
  task_id: string;
  stage: string;
  message: string;
  stage_index?: number | null;
  stage_total?: number | null;
  concurrency?: number | null;
};

export type BenchmarkRequestLogPageInput = {
  task_id: string;
  page: number;
  page_size: number;
  stage_index?: number;
  status?: string;
  keyword?: string;
};

export type BenchmarkRequestLogSummary = {
  id: string;
  task_id: string;
  stage_index: number;
  request_index: number;
  sample_index: number;
  status: string;
  latency_ms: number;
  ttft_ms: number;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  error_kind?: string | null;
  prompt_preview?: string | null;
  response_preview?: string | null;
  created_at: string;
};

export type BenchmarkRequestLogDetail = BenchmarkRequestLogSummary & {
  prompt?: string | null;
  response_text?: string | null;
  raw_error?: string | null;
  raw_usage?: unknown;
  body_available: boolean;
};

export type BenchmarkRequestLogPage = {
  items: BenchmarkRequestLogSummary[];
  total: number;
  page: number;
  page_size: number;
};
