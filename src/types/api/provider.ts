export const PROVIDER_INTERFACE_TYPES = [
  "OpenAI",
  "OpenAI-Response",
  "Anthropic",
  "Gemini",
  "Jina Rerank",
] as const;

export type ProviderInterfaceType = (typeof PROVIDER_INTERFACE_TYPES)[number];

export type ProviderSummary = {
  id: string;
  name: string;
  base_url_masked: string;
  api_key_masked: string;
  interface_type: string;
  status: string;
  model_count: number;
  last_checked_at: string | null;
  created_at: string;
};

export type CreateProviderInput = {
  name: string;
  base_url: string;
  api_key?: string;
  interface_type: ProviderInterfaceType;
};

export type UpdateProviderInput = {
  name: string;
  base_url: string;
  api_key?: string;
  interface_type: ProviderInterfaceType;
};

export type ProviderImportItem = {
  name: string;
  base_url: string;
  api_key?: string;
  interface_type: ProviderInterfaceType;
  models?: string[];
};

export type ProviderImportInput = {
  items: ProviderImportItem[];
};

export type ProviderImportItemResult = {
  index: number;
  status: "created" | "skipped" | "failed" | string;
  provider_id: string | null;
  message: string;
};

export type ProviderImportResult = {
  created: number;
  skipped: number;
  failed: number;
  items: ProviderImportItemResult[];
};

export type DeleteResult = {
  id: string;
  deleted: boolean;
};

export type ProviderConnectionResult = {
  provider_id: string;
  ok: boolean;
  status: string;
  message: string;
  checked_at: string;
};

export type ProviderDiagnosticsInput = {
  provider_id: string;
  model_id?: string;
  dataset_id?: string;
};

export type DiagnosticEndpoint = {
  name: string;
  method: string;
  path: string;
  ok: boolean;
  latency_ms: number | null;
  http_status: number | null;
  message: string;
  error_kind: string | null;
};

export type ProviderDiagnosticsResult = {
  provider_id: string;
  status: "passed" | "warning" | "failed" | "unsupported" | string;
  checked_at: string;
  engine_mode: string;
  endpoints: DiagnosticEndpoint[];
  recommendations: string[];
};
