import type {
  EndpointProbeInterfaceType,
  EndpointProbeRunSummary,
  EndpointProbeResponseDeltaEvent,
  ProviderImportItem,
} from "../../../types/api";

export const endpointProbeInterfaceOptions: Array<{
  value: EndpointProbeInterfaceType;
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

export function endpointProbeInterfaceLabel(value: string) {
  return endpointProbeInterfaceOptions.find((option) => option.value === value)?.label ?? value;
}

export function endpointProbeStatusLabel(status: string) {
  return (
    {
      pending: "排队中",
      running: "测活中",
      passed: "可用",
      failed: "失败",
      cancelled: "已停止",
      completed: "已完成",
    }[status] ?? status
  );
}

export function endpointProbeStatusTone(status: string) {
  if (status === "passed" || status === "completed") return "success" as const;
  if (status === "running" || status === "pending") return "info" as const;
  if (status === "cancelled") return "warning" as const;
  return "danger" as const;
}

export function pickDefaultEndpointProbeRunId(
  runs: EndpointProbeRunSummary[],
) {
  return (
    runs.find((run) => run.status === "failed")?.id ??
    runs[0]?.id ??
    null
  );
}

export function endpointProbeRunResultText(run: EndpointProbeRunSummary) {
  if (run.status === "failed") {
    return [run.error_kind, run.error_message].filter(Boolean).join(" · ") || "请求失败";
  }
  if (run.status === "running") return "正在接收响应";
  if (run.status === "pending") return "等待调度";
  if (run.status === "passed" && run.source_type === "temporary") {
    return "可保存为服务商";
  }
  if (run.status === "passed") return "请求可用";
  if (run.status === "cancelled") return "已停止";
  return endpointProbeStatusLabel(run.status);
}

export function canPromoteEndpointProbeRun(run: EndpointProbeRunSummary) {
  return run.source_type === "temporary" && run.status === "passed";
}

export type EndpointProbeStreamBuffer = {
  batchId: string;
  text: string;
  lastSequence: number;
  finished: boolean;
};

export function appendEndpointProbeDeltas(
  current: Record<string, EndpointProbeStreamBuffer>,
  events: EndpointProbeResponseDeltaEvent[],
) {
  const next = { ...current };
  const ordered = [...events].sort((left, right) => {
    if (left.run_id === right.run_id) return left.sequence - right.sequence;
    return left.run_id.localeCompare(right.run_id);
  });

  for (const event of ordered) {
    const existing = next[event.run_id];
    if (existing && existing.batchId !== event.batch_id) continue;
    if (existing?.finished || event.sequence <= (existing?.lastSequence ?? -1)) continue;
    next[event.run_id] = {
      batchId: event.batch_id,
      text: `${existing?.text ?? ""}${event.delta}`,
      lastSequence: event.sequence,
      finished: false,
    };
  }
  return next;
}

export function parseProviderImportJson(source: string): ProviderImportItem[] {
  const parsed: unknown = JSON.parse(source);
  const records = Array.isArray(parsed)
    ? parsed
    : isRecord(parsed) && Array.isArray(parsed.providers)
      ? parsed.providers
      : null;
  if (!records) throw new Error("JSON 顶层必须是数组，或包含 providers 数组。");
  if (!records.length) throw new Error("导入文件中没有服务商记录。");

  return records.map((record, index) => {
    if (!isRecord(record)) throw new Error(`第 ${index + 1} 条记录不是对象。`);
    const name = requiredString(record.name, index, "name");
    const base_url = requiredString(record.base_url, index, "base_url");
    let parsedUrl: URL;
    try {
      parsedUrl = new URL(base_url);
    } catch {
      throw new Error(`第 ${index + 1} 条记录的 base_url 不是有效 URL。`);
    }
    if (!["http:", "https:"].includes(parsedUrl.protocol)) {
      throw new Error(`第 ${index + 1} 条记录的 base_url 只支持 http 或 https。`);
    }
    if (parsedUrl.search || parsedUrl.hash) {
      throw new Error(`第 ${index + 1} 条记录的 base_url 不能包含 query 或 fragment。`);
    }
    const interface_type = requiredString(record.interface_type, index, "interface_type");
    if (!["OpenAI", "OpenAI-Response", "Anthropic"].includes(interface_type)) {
      throw new Error(`第 ${index + 1} 条记录的 interface_type 不受支持。`);
    }
    if (record.models !== undefined && !Array.isArray(record.models)) {
      throw new Error(`第 ${index + 1} 条记录的 models 必须是字符串数组。`);
    }
    const models = (record.models as unknown[] | undefined)?.map((model) => {
      if (typeof model !== "string" || !model.trim()) {
        throw new Error(`第 ${index + 1} 条记录包含无效模型名。`);
      }
      return model.trim();
    });
    if (record.api_key !== undefined && typeof record.api_key !== "string") {
      throw new Error(`第 ${index + 1} 条记录的 api_key 必须是字符串。`);
    }
    return {
      name,
      base_url,
      api_key: typeof record.api_key === "string" ? record.api_key : undefined,
      interface_type: interface_type as ProviderImportItem["interface_type"],
      models,
    };
  });
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function requiredString(
  value: unknown,
  index: number,
  field: string,
) {
  if (typeof value !== "string" || !value.trim()) {
    throw new Error(`第 ${index + 1} 条记录缺少 ${field}。`);
  }
  return value.trim();
}
