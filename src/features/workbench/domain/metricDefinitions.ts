import { getModelChartTabs } from "../../../domain/modelMetrics";
import { normalizeModelType } from "../../../lib/modelTaxonomy";
import type { MetricsTick } from "../../../types/api";
import type { ChartMetric } from "../types";

export function getLiveMetricCards(modelTypeValue: string, tick: MetricsTick | null) {
  const modelType = normalizeModelType(modelTypeValue);

  if (modelType === "embedding") {
    return [
      { label: "QPS", value: tick?.qps ?? "-" },
      { label: "Batch", value: tick?.batch_size || "-" },
      { label: "Text/s", value: tick?.text_count || "-" },
      { label: "Input Token/s", value: tick?.input_tokens ?? "-", unit: "tok/s" },
      { label: "Success", value: tick?.success_rate ?? "-", unit: "%" },
    ];
  }

  if (modelType === "rerank") {
    return [
      { label: "Query/s", value: tick?.qps ?? "-" },
      { label: "Pair/s", value: tick?.pair_count || "-" },
      { label: "Docs/Query", value: tick?.documents_per_query || "-" },
      { label: "Latency", value: tick?.latency_ms ?? "-", unit: "ms" },
      { label: "Success", value: tick?.success_rate ?? "-", unit: "%" },
    ];
  }

  if (modelType === "multimodal") {
    return [
      { label: "QPS", value: tick?.qps ?? "-" },
      {
        label: "Image/s",
        value: tick ? Math.round(tick.qps * Math.max(1, tick.image_count)) : "-",
      },
      { label: "TTFT", value: tick?.ttft_ms ?? "-", unit: "ms" },
      { label: "Latency", value: tick?.latency_ms ?? "-", unit: "ms" },
      { label: "Success", value: tick?.success_rate ?? "-", unit: "%" },
    ];
  }

  return [
    { label: "QPS", value: tick?.qps ?? "-" },
    { label: "TTFT", value: tick?.ttft_ms ?? "-", unit: "ms" },
    { label: "Latency", value: tick?.latency_ms ?? "-", unit: "ms" },
    { label: "Output TPS", value: tick?.tps ?? "-", unit: "tok/s" },
    { label: "Success", value: tick?.success_rate ?? "-", unit: "%" },
  ];
}

export function getChartTabs(modelTypeValue: string): Array<{ key: ChartMetric; label: string }> {
  return getModelChartTabs(modelTypeValue);
}
