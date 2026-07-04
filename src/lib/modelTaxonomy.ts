import type { ModelCapability, ModelSummary, ModelType } from "../types/api";

export const MODEL_TYPE_LABELS: Record<ModelType, string> = {
  text_generation: "文本生成",
  embedding: "向量嵌入",
  multimodal: "视觉多模态",
  rerank: "重排序",
};

export const MODEL_TYPE_DESCRIPTIONS: Record<ModelType, string> = {
  text_generation: "关注 TTFT、输出 TPS、上下文长度和端到端延迟",
  embedding: "关注批量吞吐、QPS、tokens/s 和向量生成延迟",
  multimodal: "关注图片输入、图文推理延迟和上传处理开销",
  rerank: "关注 query + 候选文档组的 docs/s 和排序延迟",
};

export const MODEL_CAPABILITY_LABELS: Record<ModelCapability, string> = {
  streaming: "Streaming",
  reasoning: "Reasoning",
  tool_calling: "Tool Calling",
  json_schema: "JSON Schema",
  image_input: "Image Input",
  batch: "Batch",
};

const LEGACY_MODEL_TYPES: Record<string, ModelType> = {
  Chat: "text_generation",
  Text: "text_generation",
  Embedding: "embedding",
  Vision: "multimodal",
  Multimodal: "multimodal",
  Reranker: "rerank",
  Rerank: "rerank",
  reranker: "rerank",
};

export function normalizeModelType(value: string): ModelType {
  if (value in MODEL_TYPE_LABELS) {
    return value as ModelType;
  }
  return LEGACY_MODEL_TYPES[value] ?? "text_generation";
}

export function getModelTypeLabel(value: string) {
  return MODEL_TYPE_LABELS[normalizeModelType(value)];
}

export function getModelTypeDescription(value: string) {
  return MODEL_TYPE_DESCRIPTIONS[normalizeModelType(value)];
}

export function getModelCapabilities(model: ModelSummary): ModelCapability[] {
  const capabilities = new Set<ModelCapability>();

  for (const capability of model.capabilities) {
    if (capability in MODEL_CAPABILITY_LABELS) {
      capabilities.add(capability as ModelCapability);
    }
  }

  if (model.supports_streaming) {
    capabilities.add("streaming");
  }

  if (normalizeModelType(model.model_type) === "multimodal") {
    capabilities.add("image_input");
  }

  return Array.from(capabilities);
}
