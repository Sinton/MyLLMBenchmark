import { describe, expect, it } from "vitest";
import { defaultWorkbenchForm } from "../types";
import { buildBenchmarkStartInput } from "./startInput";

const validForm = {
  ...defaultWorkbenchForm,
  provider_id: "provider-1",
  model_id: "model-1",
  dataset_id: "dataset-1",
};

describe("buildBenchmarkStartInput", () => {
  it("builds staircase benchmark input with workload config", () => {
    const result = buildBenchmarkStartInput({
      estimatedSeconds: 35,
      form: validForm,
      isStaircase: true,
      selectedModelType: "text_generation",
    });

    expect(result.ok).toBe(true);
    if (!result.ok) return;
    expect(result.input.concurrency).toBe(validForm.end_concurrency);
    expect(result.input.duration_seconds).toBe(35);
    expect(result.input.stage_sample_rounds).toBe(validForm.stage_sample_rounds);
    expect(result.input.stage_duration_seconds).toBe(validForm.stage_sample_rounds);
    expect(result.input.warmup_rounds).toBe(validForm.warmup_rounds);
    expect(result.input.request_timeout_seconds).toBe(120);
    expect(result.input.sla_stop_policy).toBe("continue_full_staircase");
    expect(result.input.workload_config).toMatchObject({
      max_output_tokens: 512,
      prompt_profile: "mixed",
      streaming: true,
    });
  });

  it("rejects invalid staircase ranges", () => {
    const result = buildBenchmarkStartInput({
      estimatedSeconds: 35,
      form: {
        ...validForm,
        start_concurrency: 16,
        end_concurrency: 8,
      },
      isStaircase: true,
      selectedModelType: "embedding",
    });

    expect(result).toEqual({
      ok: false,
      message: "结束并发不能小于起始并发。",
    });
  });
});
