import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type {
  EndpointProbeModelOption,
  ProviderSummary,
} from "../../../types/api";
import { createEndpointProbeCommonForm } from "../domain/endpointProbeForm";
import {
  DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATES,
} from "../domain/endpointProbePromptTemplates";
import { EndpointProbeConfiguration } from "./EndpointProbeConfiguration";

type EndpointProbeConfigurationView =
  Parameters<typeof EndpointProbeConfiguration>[0]["view"];

describe("EndpointProbeConfiguration", () => {
  it("uses the single probe provider and model pickers in the main flow", () => {
    const markup = renderToStaticMarkup(
      <EndpointProbeConfiguration view={view()} />,
    );

    expect(markup).toContain("endpoint-probe-provider-trigger");
    expect(markup).toContain("endpoint-probe-model-trigger");
    expect(markup).not.toContain("endpoint-probe-provider-table");
    expect(markup).not.toContain("已明确选择");
  });

  it("keeps low-frequency request parameters behind the wrench trigger", () => {
    const markup = renderToStaticMarkup(
      <EndpointProbeConfiguration view={view()} />,
    );

    expect(markup).toContain("展开请求参数");
    expect(markup).toContain('aria-expanded="false"');
    expect(markup).not.toContain("最大输出 Token");
    expect(markup).not.toContain("请求超时（秒）");
  });
});

function view(
  overrides: Partial<EndpointProbeConfigurationView> = {},
): EndpointProbeConfigurationView {
  return {
    activeBatch: null,
    addPromptTemplate: () => undefined,
    common: createEndpointProbeCommonForm(),
    listenersReady: true,
    promptTemplateDirty: false,
    promptTemplates: DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATES.items,
    promptTemplatesLoading: false,
    providers: [provider()],
    providerModels: { "provider-id": [model()] },
    refreshProviderModels: () => undefined,
    resetTemporaryModels: () => undefined,
    running: false,
    scanTemporaryModels: () => undefined,
    scanningProviderId: null,
    scanningTemporary: false,
    saveCurrentPromptTemplate: () => undefined,
    savingPromptTemplate: false,
    selectPromptTemplate: () => undefined,
    selectedPromptTemplateId: DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATES.selected_id,
    setSingleProviderId: () => undefined,
    setSingleProviderModel: () => undefined,
    setSingleSource: () => undefined,
    setTemporary: () => undefined,
    singleProviderId: "provider-id",
    singleProviderModel: "gpt-5-mini",
    singleProviderModels: [model()],
    singleSource: "provider",
    start: () => undefined,
    startIssue: null,
    stop: () => undefined,
    stopping: false,
    temporary: {
      name: "",
      base_url: "",
      api_key: "",
      interface_type: "OpenAI",
      model: "",
    },
    temporaryModels: [],
    ...overrides,
  } as EndpointProbeConfigurationView;
}

function provider(overrides: Partial<ProviderSummary> = {}): ProviderSummary {
  return {
    id: "provider-id",
    name: "自建中转站",
    base_url_masked: "https://api.example.com/v1",
    api_key_masked: "sk-***",
    interface_type: "OpenAI",
    status: "online",
    model_count: 1,
    last_checked_at: "2026-08-08T00:00:00Z",
    created_at: "2026-08-08T00:00:00Z",
    ...overrides,
  };
}

function model(
  overrides: Partial<EndpointProbeModelOption> = {},
): EndpointProbeModelOption {
  return {
    name: "gpt-5-mini",
    model_type: "text_generation",
    capabilities: ["chat"],
    supports_streaming: true,
    ...overrides,
  };
}
