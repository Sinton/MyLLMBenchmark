export const SITE_PROBE_INTERFACE_TYPES = [
  "OpenAI",
  "OpenAI-Response",
  "Anthropic",
] as const;

export type SiteProbeInterfaceType = (typeof SITE_PROBE_INTERFACE_TYPES)[number];

export type SiteProbeRunInput = {
  name?: string;
  base_url: string;
  api_key?: string;
  interface_type: SiteProbeInterfaceType;
  model: string;
  prompt: string;
  streaming: boolean;
  max_output_tokens?: number;
  timeout_seconds?: number;
  save_body: boolean;
};

export type SiteProbeModelScanInput = {
  base_url: string;
  api_key?: string;
  interface_type: SiteProbeInterfaceType;
};

export type SiteProbeModelOption = {
  name: string;
  model_type: string;
  capabilities: string[];
  supports_streaming: boolean;
};

export type SiteProbeModelScanResult = {
  models: SiteProbeModelOption[];
  message: string;
  scanned_at: string;
};

export type SiteProbeRunSummary = {
  id: string;
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
};

export type SiteProbeRunDetail = SiteProbeRunSummary & {
  prompt: string | null;
  response_text: string | null;
  request_payload: unknown | null;
  raw_error: string | null;
  raw_usage: unknown | null;
};

export type SiteProbeHistoryPageInput = {
  page: number;
  page_size: number;
  status?: string;
  keyword?: string;
};

export type SiteProbeHistoryPage = {
  items: SiteProbeRunSummary[];
  total: number;
  page: number;
  page_size: number;
};
