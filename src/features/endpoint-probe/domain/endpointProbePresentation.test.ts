import { describe, expect, it } from "vitest";
import type { EndpointProbeResponseDeltaEvent } from "../../../types/api";
import {
  appendEndpointProbeDeltas,
  parseProviderImportJson,
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

function delta(
  batch_id: string,
  run_id: string,
  sequence: number,
  value: string,
): EndpointProbeResponseDeltaEvent {
  return { batch_id, run_id, sequence, delta: value, elapsed_ms: sequence * 10 };
}
