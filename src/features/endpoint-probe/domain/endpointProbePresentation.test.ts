import { describe, expect, it } from "vitest";
import type {
  EndpointProbeResponseDeltaEvent,
  EndpointProbeRunSummary,
} from "../../../types/api";
import {
  appendEndpointProbeDeltas,
  canPromoteEndpointProbeRun,
  endpointProbeRunResultText,
  parseProviderImportJson,
  pickDefaultEndpointProbeRunId,
} from "./endpointProbePresentation";

describe("appendEndpointProbeDeltas", () => {
  it("orders chunks, ignores duplicates, and isolates batches", () => {
    const events: EndpointProbeResponseDeltaEvent[] = [
      delta("batch-a", "run-a", 1, "B"),
      delta("batch-a", "run-a", 0, "A"),
      delta("batch-a", "run-a", 1, "duplicate"),
      delta("batch-b", "run-b", 0, "other"),
    ];
    const first = appendEndpointProbeDeltas({}, events);

    expect(first["run-a"]).toMatchObject({ batchId: "batch-a", text: "AB", lastSequence: 1 });
    expect(first["run-b"]).toMatchObject({ batchId: "batch-b", text: "other" });

    const second = appendEndpointProbeDeltas(first, [
      delta("wrong-batch", "run-a", 2, "leak"),
      delta("batch-a", "run-a", 2, "C"),
    ]);
    expect(second["run-a"].text).toBe("ABC");
  });
});

describe("parseProviderImportJson", () => {
  it("accepts array and providers envelope without exposing extra fields", () => {
    const items = parseProviderImportJson(JSON.stringify({
      providers: [
        {
          name: "Gateway",
          base_url: "https://example.com/v1/",
          api_key: "secret",
          interface_type: "OpenAI-Response",
          models: ["model-a"],
          ignored: "value",
        },
      ],
    }));

    expect(items).toEqual([
      {
        name: "Gateway",
        base_url: "https://example.com/v1/",
        api_key: "secret",
        interface_type: "OpenAI-Response",
        models: ["model-a"],
      },
    ]);
  });

  it("rejects unsupported probe protocols and malformed models", () => {
    expect(() =>
      parseProviderImportJson(
        JSON.stringify([
          { name: "Gemini", base_url: "https://example.com", interface_type: "Gemini" },
        ]),
      ),
    ).toThrow("interface_type");
    expect(() =>
      parseProviderImportJson(
        JSON.stringify([
          { name: "Gateway", base_url: "https://example.com", interface_type: "OpenAI", models: "all" },
        ]),
      ),
    ).toThrow("models");
  });
});

describe("endpoint probe result presentation", () => {
  it("auto-focuses the failed run before falling back to the first run", () => {
    expect(
      pickDefaultEndpointProbeRunId([
        run({ id: "run-a", status: "passed" }),
        run({ id: "run-b", status: "failed" }),
      ]),
    ).toBe("run-b");
    expect(
      pickDefaultEndpointProbeRunId([
        run({ id: "run-a", status: "passed" }),
        run({ id: "run-b", status: "passed" }),
      ]),
    ).toBe("run-a");
    expect(pickDefaultEndpointProbeRunId([])).toBeNull();
  });

  it("summarizes failures and temporary promotion hints", () => {
    expect(
      endpointProbeRunResultText(
        run({
          status: "failed",
          error_kind: "unauthorized",
          error_message: "API Key 无效",
        }),
      ),
    ).toBe("unauthorized · API Key 无效");
    expect(endpointProbeRunResultText(run({ status: "running" }))).toBe("正在接收响应");
    expect(
      endpointProbeRunResultText(
        run({ status: "passed", source_type: "temporary" }),
      ),
    ).toBe("可保存为服务商");
  });

  it("marks only passed temporary runs as promotable", () => {
    expect(
      canPromoteEndpointProbeRun(run({ status: "passed", source_type: "temporary" })),
    ).toBe(true);
    expect(
      canPromoteEndpointProbeRun(run({ status: "failed", source_type: "temporary" })),
    ).toBe(false);
    expect(canPromoteEndpointProbeRun(run({ status: "passed", source_type: "provider" }))).toBe(false);
  });
});

function delta(
  batch_id: string,
  run_id: string,
  sequence: number,
  value: string,
): EndpointProbeResponseDeltaEvent {
  return { batch_id, run_id, sequence, delta: value, elapsed_ms: sequence * 10 };
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
