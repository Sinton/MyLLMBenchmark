import { describe, expect, it } from "vitest";
import { buildFallbackPlan, buildStageSequence } from "./benchmarkPlan";

describe("benchmarkPlan", () => {
  it("builds doubling staircase stages and includes the configured end value", () => {
    expect(buildStageSequence(1, 64, "double", 2)).toEqual([
      1, 2, 4, 8, 16, 32, 64,
    ]);
    expect(buildStageSequence(1, 40, "double", 2)).toEqual([
      1, 2, 4, 8, 16, 32, 40,
    ]);
  });

  it("builds fallback plans with fixed mode as a single stage", () => {
    expect(
      buildFallbackPlan({
        endConcurrency: 16,
        mode: "固定并发",
        stageSampleRounds: 1,
        startConcurrency: 1,
        stepStrategy: "linear",
        stepValue: 4,
        warmupRounds: -1,
      }),
    ).toEqual({
      stages: [16],
      stageSampleRounds: 1,
      warmupRounds: 0,
    });
  });
});
