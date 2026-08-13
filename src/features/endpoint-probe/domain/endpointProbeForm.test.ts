import { describe, expect, it } from "vitest";
import {
  buildEndpointProbeStartInput,
  createEndpointProbeCommonForm,
  createEndpointProbeTemporaryForm,
  validateEndpointProbeStart,
  type EndpointProbeFormSnapshot,
} from "./endpointProbeForm";

describe("endpoint probe form", () => {
  it("uses stable probe defaults", () => {
    expect(createEndpointProbeCommonForm()).toMatchObject({
      streaming: true,
      temperature: 0.2,
      max_output_tokens: 1024,
      timeout_seconds: 60,
      save_body: false,
      concurrency: 3,
    });
  });

  it("passes temperature through to the start input", () => {
    const input = buildEndpointProbeStartInput(
      snapshot({
        common: {
          ...createEndpointProbeCommonForm(),
          temperature: 0.4,
        },
      }),
    );

    expect(input.temperature).toBe(0.4);
    expect(input.max_output_tokens).toBe(1024);
    expect(input.timeout_seconds).toBe(60);
  });

  it("validates the OpenAI-compatible temperature range at the form boundary", () => {
    expect(
      validateEndpointProbeStart(
        snapshot({
          common: {
            ...createEndpointProbeCommonForm(),
            temperature: 2.1,
          },
        }),
        true,
      ),
    ).toBe("Temperature 需在 0-2 之间。");
  });
});

function snapshot(
  overrides: Partial<EndpointProbeFormSnapshot> = {},
): EndpointProbeFormSnapshot {
  return {
    workspaceMode: "single",
    singleSource: "provider",
    common: createEndpointProbeCommonForm(),
    temporary: createEndpointProbeTemporaryForm(),
    singleProviderId: "provider-id",
    singleProviderModel: "gpt-5-mini",
    batchModels: {},
    ...overrides,
  };
}
