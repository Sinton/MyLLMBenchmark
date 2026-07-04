export type { ChartMetric } from "../../domain/modelMetrics";

export type WorkbenchForm = {
  provider_id: string;
  model_id: string;
  dataset_id: string;
  mode: string;
  concurrency: number;
  duration_seconds: number;
  start_concurrency: number;
  end_concurrency: number;
  step_strategy: string;
  step_value: number;
  stage_sample_rounds: number;
  stage_duration_seconds: number;
  warmup_rounds: number;
  warmup_seconds: number;
  request_timeout_seconds: number;
  sla_p95_ms: number;
  min_success_rate: number;
  sla_stop_policy: "continue_full_staircase" | "stop_on_failure";
  streaming: boolean;
  max_output_tokens: number;
  prompt_profile: string;
  embedding_batch_size: number;
  embedding_text_count_per_request: number;
  rerank_documents_per_query: number;
  rerank_top_k: number;
  vision_image_profile: string;
  vision_image_count: number;
};

export type StartNotice = {
  tone: "info" | "success" | "danger";
  title: string;
  message: string;
};

export const defaultWorkbenchForm: WorkbenchForm = {
  provider_id: "",
  model_id: "",
  dataset_id: "",
  mode: "阶梯加压",
  concurrency: 32,
  duration_seconds: 24,
  start_concurrency: 1,
  end_concurrency: 64,
  step_strategy: "double",
  step_value: 2,
  stage_sample_rounds: 4,
  stage_duration_seconds: 4,
  warmup_rounds: 1,
  warmup_seconds: 1,
  request_timeout_seconds: 120,
  sla_p95_ms: 5000,
  min_success_rate: 99,
  sla_stop_policy: "continue_full_staircase",
  streaming: true,
  max_output_tokens: 512,
  prompt_profile: "mixed",
  embedding_batch_size: 16,
  embedding_text_count_per_request: 16,
  rerank_documents_per_query: 30,
  rerank_top_k: 10,
  vision_image_profile: "medium",
  vision_image_count: 1,
};
