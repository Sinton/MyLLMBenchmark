export const MODEL_TYPES = [
  "text_generation",
  "embedding",
  "multimodal",
  "rerank",
] as const;

export type ModelType = (typeof MODEL_TYPES)[number];

export const MODEL_CAPABILITIES = [
  "streaming",
  "reasoning",
  "tool_calling",
  "json_schema",
  "image_input",
  "batch",
] as const;

export type ModelCapability = (typeof MODEL_CAPABILITIES)[number];

export type ModelSummary = {
  id: string;
  provider_id: string;
  name: string;
  model_type: ModelType | string;
  capabilities: ModelCapability[] | string[];
  supports_streaming: boolean;
  recommended_concurrency: number | null;
};

export type ProviderModelScanResult = {
  provider_id: string;
  models: ModelSummary[];
  message: string;
  scanned_at: string;
};
