import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { EndpointProbeRunDetail } from "../../../types/api";
import { EndpointProbeRunExpanded } from "./EndpointProbeRunExpanded";

describe("EndpointProbeRunExpanded", () => {
  it("shows failed HTTP status in the response toolbar", () => {
    const detail = runDetail({
      status: "failed",
      error_kind: "http_5xx",
      error_message: "HTTP 502 Bad Gateway",
      raw_error: "HTTP 502 Bad Gateway",
    });

    const markup = renderToStaticMarkup(
      <EndpointProbeRunExpanded
        detail={detail}
        error={null}
        liveText=""
        loading={false}
        run={detail}
        onCopy={async () => undefined}
        onRetry={() => undefined}
      />,
    );

    expect(markup).toContain("响应已完成");
    expect(markup).toContain("HTTP 502");
    expect(markup).not.toContain("endpoint-probe-error-detail");
  });

  it("shows readable error details instead of raw error kind in metrics", () => {
    const detail = runDetail({
      status: "failed",
      error_kind: "http_5xx",
      error_message: "HTTP 502 Bad Gateway: model gpt-5.5 is not available",
      raw_error: "HTTP 502 Bad Gateway: model gpt-5.5 is not available",
    });

    const markup = renderToStaticMarkup(
      <EndpointProbeRunExpanded
        detail={detail}
        error={null}
        initialTab="metrics"
        liveText=""
        loading={false}
        run={detail}
        onCopy={async () => undefined}
        onRetry={() => undefined}
      />,
    );

    expect(markup).toContain("错误说明");
    expect(markup).toContain("model gpt-5.5 is not available");
    expect(markup).toContain("服务端错误（5xx）");
    expect(markup).not.toContain(">http_5xx<");
  });
});

function runDetail(overrides: Partial<EndpointProbeRunDetail> = {}): EndpointProbeRunDetail {
  return {
    id: "run-id",
    batch_id: "batch-id",
    source_type: "provider",
    provider_id: "provider-id",
    name: "Gateway",
    base_url: "https://example.com/v1",
    interface_type: "OpenAI",
    model: "model-a",
    status: "passed",
    latency_ms: 120,
    ttft_ms: 80,
    input_tokens: 1,
    output_tokens: 1,
    total_tokens: 2,
    error_kind: null,
    error_message: null,
    prompt_preview: "Prompt",
    response_preview: "OK",
    body_available: true,
    created_at: "2026-08-08T00:00:00Z",
    finished_at: "2026-08-08T00:00:01Z",
    prompt: "Prompt",
    response_text: null,
    request_payload: null,
    raw_error: null,
    raw_usage: null,
    ...overrides,
  };
}
