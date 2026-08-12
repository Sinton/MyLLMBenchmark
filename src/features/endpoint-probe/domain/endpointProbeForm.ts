import type {
  EndpointProbeInterfaceType,
  EndpointProbeStartInput,
} from "../../../types/api";
import { DEFAULT_ENDPOINT_PROBE_PROMPT } from "./endpointProbePromptTemplates";

export type EndpointProbeWorkspaceMode = "single" | "batch";
export type EndpointProbeSingleSource = "provider" | "temporary";

export type EndpointProbeCommonForm = {
  prompt: string;
  streaming: boolean;
  max_output_tokens: number;
  timeout_seconds: number;
  save_body: boolean;
  concurrency: number;
};

export type EndpointProbeTemporaryForm = {
  name: string;
  base_url: string;
  api_key: string;
  interface_type: EndpointProbeInterfaceType;
  model: string;
};

export const createEndpointProbeCommonForm = (): EndpointProbeCommonForm => ({
  prompt: DEFAULT_ENDPOINT_PROBE_PROMPT,
  streaming: true,
  max_output_tokens: 1024,
  timeout_seconds: 60,
  save_body: false,
  concurrency: 3,
});

export const createEndpointProbeTemporaryForm = (): EndpointProbeTemporaryForm => ({
  name: "",
  base_url: "",
  api_key: "",
  interface_type: "OpenAI",
  model: "",
});

export type EndpointProbeFormSnapshot = {
  workspaceMode: EndpointProbeWorkspaceMode;
  singleSource: EndpointProbeSingleSource;
  common: EndpointProbeCommonForm;
  temporary: EndpointProbeTemporaryForm;
  singleProviderId: string;
  singleProviderModel: string;
  batchModels: Record<string, string[]>;
};

export function countSelectedProbeRuns(batchModels: Record<string, string[]>) {
  return Object.values(batchModels).reduce((total, models) => total + models.length, 0);
}

export function validateEndpointProbeStart(
  snapshot: EndpointProbeFormSnapshot,
  listenersReady: boolean,
) {
  const selectedRunCount = countSelectedProbeRuns(snapshot.batchModels);
  if (!listenersReady) return "实时事件通道仍在初始化，请稍候。";
  if (!snapshot.common.prompt.trim()) return "请输入用于测活的 Prompt。";
  if (snapshot.common.max_output_tokens < 1 || snapshot.common.max_output_tokens > 8_192) {
    return "最大输出 Token 需在 1-8192 之间。";
  }
  if (snapshot.common.timeout_seconds < 5 || snapshot.common.timeout_seconds > 600) {
    return "请求超时需在 5-600 秒之间。";
  }
  if (snapshot.workspaceMode === "batch") {
    if (snapshot.common.concurrency < 1 || snapshot.common.concurrency > 10) {
      return "批量并发需在 1-10 之间。";
    }
    if (!selectedRunCount) return "请为至少一个服务商明确选择模型。";
    if (selectedRunCount > 200) return "单个批次最多包含 200 个模型请求。";
    return null;
  }
  if (snapshot.singleSource === "provider") {
    if (!snapshot.singleProviderId) return "请选择服务商。";
    if (!snapshot.singleProviderModel.trim()) return "请选择模型。";
    return null;
  }
  if (!snapshot.temporary.base_url.trim()) return "请输入临时站点 Base URL。";
  if (!snapshot.temporary.model.trim()) return "请选择或填写模型名称。";
  return null;
}

export function buildEndpointProbeStartInput(
  snapshot: EndpointProbeFormSnapshot,
): EndpointProbeStartInput {
  const targets = snapshot.workspaceMode === "batch"
    ? Object.entries(snapshot.batchModels)
        .filter(([, models]) => models.length > 0)
        .map(([provider_id, models]) => ({
          source: "provider" as const,
          provider_id,
          models,
        }))
    : snapshot.singleSource === "provider"
      ? [{
          source: "provider" as const,
          provider_id: snapshot.singleProviderId,
          models: [snapshot.singleProviderModel],
        }]
      : [{
          source: "temporary" as const,
          name: snapshot.temporary.name.trim() || undefined,
          base_url: snapshot.temporary.base_url.trim(),
          api_key: snapshot.temporary.api_key,
          interface_type: snapshot.temporary.interface_type,
          models: [snapshot.temporary.model.trim()],
        }];

  return {
    targets,
    prompt: snapshot.common.prompt,
    streaming: snapshot.common.streaming,
    max_output_tokens: Number(snapshot.common.max_output_tokens),
    timeout_seconds: Number(snapshot.common.timeout_seconds),
    save_body: snapshot.common.save_body,
    concurrency: snapshot.workspaceMode === "single" ? 1 : Number(snapshot.common.concurrency),
  };
}
