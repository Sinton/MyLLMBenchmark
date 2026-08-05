import type {
  EndpointProbePromptTemplate,
  EndpointProbePromptTemplatesConfig,
} from "../../../types/api";

export const DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID = "basic-liveness";
export const DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_NAME = "基础测活";
export const DEFAULT_ENDPOINT_PROBE_PROMPT =
  "请回复1+1 是否等于2，回答是或者不是。";

export const DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATES: EndpointProbePromptTemplatesConfig = {
  selected_id: DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID,
  items: [
    {
      id: DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID,
      name: DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_NAME,
      prompt: DEFAULT_ENDPOINT_PROBE_PROMPT,
    },
  ],
};

export function normalizeEndpointProbePromptTemplates(
  config?: EndpointProbePromptTemplatesConfig | null,
): EndpointProbePromptTemplatesConfig {
  const items = (config?.items ?? [])
    .filter((item) => item.id.trim() && item.name.trim() && item.prompt.trim())
    .map((item) => ({
      id: item.id,
      name: item.name.trim(),
      prompt: item.prompt,
    }));

  if (!items.length) return DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATES;

  const selected_id = items.some((item) => item.id === config?.selected_id)
    ? config?.selected_id ?? items[0].id
    : items[0].id;

  return { selected_id, items };
}

export function selectedEndpointProbePromptTemplate(
  config: EndpointProbePromptTemplatesConfig,
) {
  return config.items.find((item) => item.id === config.selected_id) ?? config.items[0];
}

export function createEndpointProbePromptTemplate(
  items: EndpointProbePromptTemplate[],
  prompt: string,
): EndpointProbePromptTemplate {
  return {
    id: `prompt-template-${crypto.randomUUID()}`,
    name: nextTemplateName(items),
    prompt: prompt.trim() ? prompt : DEFAULT_ENDPOINT_PROBE_PROMPT,
  };
}

function nextTemplateName(items: EndpointProbePromptTemplate[]) {
  let index = 1;
  const names = new Set(items.map((item) => item.name));
  while (names.has(`自定义模板 ${index}`)) index += 1;
  return `自定义模板 ${index}`;
}
