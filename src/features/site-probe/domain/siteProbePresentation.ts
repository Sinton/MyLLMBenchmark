import type {
  SiteProbeInterfaceType,
  SiteProbeModelOption,
} from "../../../types/api";

export const siteProbeInterfaceOptions: Array<{
  value: SiteProbeInterfaceType;
  label: string;
  description: string;
}> = [
  {
    value: "OpenAI",
    label: "OpenAI Chat Completions",
    description: "POST /v1/chat/completions",
  },
  {
    value: "OpenAI-Response",
    label: "OpenAI Responses",
    description: "POST /v1/responses",
  },
  {
    value: "Anthropic",
    label: "Anthropic Messages (Claude)",
    description: "POST /v1/messages",
  },
];

export function siteProbeInterfaceLabel(value: string) {
  return siteProbeInterfaceOptions.find((option) => option.value === value)?.label ?? value;
}

export function siteProbeModelDescription(model: SiteProbeModelOption) {
  const typeLabel =
    {
      embedding: "向量模型",
      multimodal: "多模态",
      reranker: "重排序",
      text_generation: "文本生成",
    }[model.model_type] ?? model.model_type;
  return model.supports_streaming ? `${typeLabel} · 支持流式响应` : typeLabel;
}
