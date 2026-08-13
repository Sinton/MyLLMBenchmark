import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type {
  EndpointProbeBatchDetail,
  EndpointProbeRunSummary,
} from "../../../types/api";
import { EndpointProbeResults } from "./EndpointProbeResults";

type EndpointProbeResultsView = Parameters<typeof EndpointProbeResults>[0]["view"];

describe("EndpointProbeResults", () => {
  it("renders a direct detail panel for a single probe run", () => {
    const markup = renderToStaticMarkup(
      <EndpointProbeResults view={view(batch())} />,
    );

    expect(markup).not.toContain("请求总数");
    expect(markup).toContain("endpoint-probe-batch-caption is-compact");
    expect(markup).toContain("endpoint-probe-single-run-panel");
    expect(markup).not.toContain("endpoint-probe-runs-table");
    expect(markup).not.toContain("并发 1");
  });

  it("keeps aggregate cards for multi-run batches", () => {
    const markup = renderToStaticMarkup(
      <EndpointProbeResults
        view={view(batch({
          total_runs: 2,
          runs: [
            run({ id: "run-a", status: "passed" }),
            run({ id: "run-b", status: "failed" }),
          ],
        }))}
      />,
    );

    expect(markup).toContain("请求总数");
    expect(markup).toContain("可用");
  });

  it("keeps the single result summary to the status badge only", () => {
    const failed = run({
      id: "failed-run",
      status: "failed",
      error_kind: "unauthorized",
      error_message: "API Key 无效",
    });
    const markup = renderToStaticMarkup(
      <EndpointProbeResults view={view(batch({ runs: [failed] }))} />,
    );

    expect(markup).toContain("endpoint-probe-single-run-status");
    expect(markup).toContain("失败");
    expect(markup).not.toContain("unauthorized · API Key 无效");
  });

  it("does not render the single result through an expandable table row", () => {
    const failed = run({
      id: "failed-run",
      status: "failed",
      error_kind: "http_5xx",
      error_message: "HTTP 502 Bad Gateway",
    });
    const markup = renderToStaticMarkup(
      <EndpointProbeResults
        view={view(batch({ runs: [failed] }), {
          expandedRunId: failed.id,
        })}
      />,
    );

    expect(markup).toContain("失败");
    expect(markup).toContain("HTTP 502");
    expect(markup).not.toContain("table-expanded-row");
    expect(markup).not.toContain("endpoint-probe-runs-table");
  });

  it("uses the actual loaded run count to choose the single result layout", () => {
    const markup = renderToStaticMarkup(
      <EndpointProbeResults
        view={view(batch({
          total_runs: 2,
          runs: [run({ id: "only-loaded-run" })],
        }))}
      />,
    );

    expect(markup).toContain("endpoint-probe-single-run-panel");
    expect(markup).not.toContain("endpoint-probe-runs-table");
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
    expect(markup).not.toContain("可保存为服务商");
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
    temperature: 0.2,
    max_output_tokens: 1024,
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
