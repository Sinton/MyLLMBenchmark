export const ENDPOINT_PROBE_INTERFACE_TYPES = [
  "OpenAI",
  "OpenAI-Response",
  "Anthropic",
] as const;

export type EndpointProbeInterfaceType =
  (typeof ENDPOINT_PROBE_INTERFACE_TYPES)[number];

export type EndpointProbeProviderTargetInput = {
  source: "provider";
  provider_id: string;
  models: string[];
};

export type EndpointProbeTemporaryTargetInput = {
  source: "temporary";
  name?: string;
  base_url: string;
  api_key?: string;
  interface_type: EndpointProbeInterfaceType;
  models: string[];
};

export type EndpointProbeTargetInput =
  | EndpointProbeProviderTargetInput
  | EndpointProbeTemporaryTargetInput;

export type EndpointProbeStartInput = {
  targets: EndpointProbeTargetInput[];
  prompt: string;
  streaming: boolean;
  max_output_tokens?: number;
  timeout_seconds?: number;
  save_body: boolean;
  concurrency?: number;
};

export type EndpointProbeModelScanInput =
  | { source: "provider"; provider_id: string }
  | {
      source: "temporary";
      base_url: string;
      api_key?: string;
      interface_type: EndpointProbeInterfaceType;
    };

export type EndpointProbeModelOption = {
  name: string;
  model_type: string;
  capabilities: string[];
  supports_streaming: boolean;
};

export type EndpointProbeModelScanResult = {
  provider_id: string | null;
  models: EndpointProbeModelOption[];
  message: string;
  scanned_at: string;
};

export type EndpointProbeBatchSummary = {
  id: string;
  name: string;
  status: string;
  total_runs: number;
  pending_runs: number;
  running_runs: number;
  passed_runs: number;
  failed_runs: number;
  cancelled_runs: number;
  streaming: boolean;
  max_output_tokens: number;
  timeout_seconds: number;
  save_body: boolean;
  concurrency: number;
  prompt_preview: string | null;
  created_at: string;
  finished_at: string | null;
};

export type EndpointProbeRunSummary = {
  id: string;
  batch_id: string;
  source_type: "provider" | "temporary" | string;
  provider_id: string | null;
  name: string;
  base_url: string;
  interface_type: string;
  model: string;
  status: string;
  latency_ms: number;
  ttft_ms: number;
  input_tokens: number;
  output_tokens: number;
  total_tokens: number;
  error_kind: string | null;
  error_message: string | null;
  prompt_preview: string | null;
  response_preview: string | null;
  body_available: boolean;
  created_at: string;
  finished_at: string | null;
};

export type EndpointProbeRunDetail = EndpointProbeRunSummary & {
  prompt: string | null;
  response_text: string | null;
  request_payload: unknown | null;
  raw_error: string | null;
  raw_usage: unknown | null;
};

export type EndpointProbeBatchDetail = EndpointProbeBatchSummary & {
  runs: EndpointProbeRunSummary[];
};

export type EndpointProbeHistoryPageInput = {
  page: number;
  page_size: number;
  status?: string;
  keyword?: string;
};

export type EndpointProbeHistoryPage = {
  items: EndpointProbeBatchSummary[];
  total: number;
  page: number;
  page_size: number;
};

export type EndpointProbeStopResult = {
  batch_id: string;
  stopped: boolean;
};

export type EndpointProbePromotionInput = {
  run_id: string;
  name?: string;
  api_key?: string;
  sync_models: boolean;
};

export type EndpointProbePromotionResult = {
  status: "created" | "already_exists" | string;
  provider: import("./provider").ProviderSummary;
  models_synced: boolean;
  warning: string | null;
};

export type EndpointProbeRunStartedEvent = {
  batch_id: string;
  run_id: string;
};

export type EndpointProbeResponseDeltaEvent = {
  batch_id: string;
  run_id: string;
  sequence: number;
  delta: string;
  elapsed_ms: number;
};

export type EndpointProbeRunFinishedEvent = {
  batch_id: string;
  run: EndpointProbeRunDetail;
};
