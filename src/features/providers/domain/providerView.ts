import {
  getModelCapabilities,
  normalizeModelType,
} from "../../../lib/modelTaxonomy";
import {
  PROVIDER_INTERFACE_TYPES,
  type ModelSummary,
  type ProviderInterfaceType,
} from "../../../types/api";

export const DEFAULT_INTERFACE_TYPE: ProviderInterfaceType = "OpenAI";

export const providerTypeOptions = PROVIDER_INTERFACE_TYPES.map((type) => ({
  value: type,
  label: type,
  description:
    type === "Jina Rerank"
      ? "重排序接口"
      : type === "OpenAI-Response"
        ? "Responses API"
        : "模型推理接口",
}));

export function countCapabilities(models: ModelSummary[]) {
  return models.reduce(
    (result, model) => {
      result[normalizeModelType(model.model_type)] += 1;
      return result;
    },
    {
      text_generation: 0,
      embedding: 0,
      multimodal: 0,
      rerank: 0,
    },
  );
}

export function formatDate(value: string) {
  return new Date(value).toLocaleString("zh-CN", { hour12: false });
}

export function getInitials(name: string) {
  const trimmed = name.trim();
  if (!trimmed) return "AI";
  if (/^[\u4e00-\u9fa5]/.test(trimmed)) {
    return trimmed.slice(0, 2);
  }
  return trimmed
    .split(/\s+/)
    .map((part) => part[0])
    .join("")
    .slice(0, 2)
    .toUpperCase();
}

export function getErrorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  return typeof error === "string" ? error : "请求未成功，请查看运行日志。";
}

export function capabilityNames(model: ModelSummary) {
  return getModelCapabilities(model);
}
