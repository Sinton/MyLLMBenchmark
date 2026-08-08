import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type {
  EndpointProbeBatchDetail,
  EndpointProbeRunSummary,
} from "../../../types/api";
import { EndpointProbeResults } from "./EndpointProbeResults";

type EndpointProbeResultsView = Parameters<typeof EndpointProbeResults>[0]["view"];

describe("EndpointProbeResults", () => {
  it("shows failed run reason directly in the results table", () => {
    const failed = run({
      id: "failed-run",
      status: "failed",
      error_kind: "unauthorized",
      error_message: "API Key 无效",
    });
    const markup = renderToStaticMarkup(
      <EndpointProbeResults view={view(batch({ runs: [failed] }))} />,
    );

    expect(markup).toContain("结果说明");
    expect(markup).toContain("unauthorized · API Key 无效");
  });

  it("shows a header promotion action for a passed single temporary run", () => {
    const passed = run({
      id: "temporary-run",
      source_type: "temporary",
      status: "passed",
    });
    const markup = renderToStaticMarkup(
      <EndpointProbeResults
        view={view(batch({ passed_runs: 1, runs: [passed] }), {
          promotableRun: passed,
        })}
      />,
    );

    expect(markup).toContain("保存为服务商");
    expect(markup).toContain("可保存为服务商");
  });
});

function view(
  activeBatch: EndpointProbeBatchDetail,
  overrides: Partial<EndpointProbeResultsView> = {},
): EndpointProbeResultsView {
  return {
    activeBatch,
    batchDetailError: null,
    batchDetailLoading: false,
    copyProbeText: async () => undefined,
    expandRun: async () => undefined,
    expandedRunId: null,
    loadingRunId: null,
    openPromotion: async () => undefined,
    promotableRun: null,
    runDetailError: null,
    runDetails: {},
    streamText: {},
    ...overrides,
  } as EndpointProbeResultsView;
}

function batch(
  overrides: Partial<EndpointProbeBatchDetail> = {},
): EndpointProbeBatchDetail {
  return {
    id: "batch-id",
    name: "Gateway / model-a",
    status: "completed",
    total_runs: overrides.runs?.length ?? 1,
    pending_runs: 0,
    running_runs: 0,
    passed_runs: 0,
    failed_runs: 0,
    cancelled_runs: 0,
    streaming: true,
    max_output_tokens: 256,
    timeout_seconds: 60,
    save_body: false,
    concurrency: 1,
    prompt_preview: "请回复 1+1 是否等于 2",
    created_at: "2026-08-08T00:00:00Z",
    finished_at: "2026-08-08T00:00:01Z",
    runs: [run()],
    ...overrides,
  };
}

function run(overrides: Partial<EndpointProbeRunSummary> = {}): EndpointProbeRunSummary {
  return {
    id: "run-id",
    batch_id: "batch-id",
    source_type: "provider",
    provider_id: "provider-id",
    name: "Gateway",
    base_url: "https://example.com/v1",
    interface_type: "OpenAI",
    model: "model-a",
    status: "pending",
    latency_ms: 0,
    ttft_ms: 0,
    input_tokens: 0,
    output_tokens: 0,
    total_tokens: 0,
    error_kind: null,
    error_message: null,
    prompt_preview: null,
    response_preview: null,
    body_available: false,
    created_at: "2026-08-08T00:00:00Z",
    finished_at: null,
    ...overrides,
  };
}
