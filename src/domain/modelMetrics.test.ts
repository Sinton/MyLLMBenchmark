import { describe, expect, it } from "vitest";
import { getModelChartTabs } from "./modelMetrics";

describe("modelMetrics", () => {
  it("uses model-specific chart tabs for embedding and rerank", () => {
    expect(getModelChartTabs("embedding").map((tab) => tab.label)).toEqual([
      "QPS",
      "Input Token/s",
      "Latency",
      "Success",
      "Error",
    ]);
    expect(getModelChartTabs("rerank").map((tab) => tab.label)).toEqual([
      "Query/s",
      "Pair/s",
      "Latency",
      "Success",
      "Error",
    ]);
  });

  it("can hide error tabs for report trend charts", () => {
    expect(
      getModelChartTabs("text_generation", { includeErrors: false }).some(
        (tab) => tab.key === "errors",
      ),
    ).toBe(false);
  });
});
