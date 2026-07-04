import { getModelChartTabs } from "../../../domain/modelMetrics";
import { normalizeModelType } from "../../../lib/modelTaxonomy";
import type { MetricsTick } from "../../../types/api";
import type { ChartMetric } from "../types";

export function getLiveMetricCards(modelTypeValue: string, tick: MetricsTick | null) {
  const modelType = normalizeModelType(modelTypeValue);

  if (modelType === "embedding") {
    return [
      { label: "QPS", helpKey: "qps", value: tick?.qps ?? "-" },
      { label: "Batch", helpKey: "batch", value: tick?.batch_size || "-" },
      { label: "Text/s", helpKey: "text_s", value: tick?.text_count || "-" },
      { label: "Input Token/s", helpKey: "input_s", value: tick?.input_tokens ?? "-", unit: "tok/s" },
      { label: "Success", helpKey: "success_rate", value: tick?.success_rate ?? "-", unit: "%" },
    ];
  }

  if (modelType === "rerank") {
    return [
      { label: "Query/s", helpKey: "query_s", value: tick?.qps ?? "-" },
      { label: "Pair/s", helpKey: "pair_s", value: tick?.pair_count || "-" },
      { label: "Docs/Query", helpKey: "docs_q", value: tick?.documents_per_query || "-" },
      { label: "Latency", helpKey: "latency", value: tick?.latency_ms ?? "-", unit: "ms" },
      { label: "Success", helpKey: "success_rate", value: tick?.success_rate ?? "-", unit: "%" },
    ];
  }

  if (modelType === "multimodal") {
    return [
      { label: "QPS", helpKey: "qps", value: tick?.qps ?? "-" },
      {
        label: "Image/s",
        helpKey: "image_s",
        value: tick ? Math.round(tick.qps * Math.max(1, tick.image_count)) : "-",
      },
      { label: "TTFT", helpKey: "ttft", value: tick?.ttft_ms ?? "-", unit: "ms" },
      { label: "Latency", helpKey: "latency", value: tick?.latency_ms ?? "-", unit: "ms" },
      { label: "Success", helpKey: "success_rate", value: tick?.success_rate ?? "-", unit: "%" },
    ];
  }

  return [
    { label: "QPS", helpKey: "qps", value: tick?.qps ?? "-" },
    { label: "TTFT", helpKey: "ttft", value: tick?.ttft_ms ?? "-", unit: "ms" },
    { label: "Latency", helpKey: "latency", value: tick?.latency_ms ?? "-", unit: "ms" },
    { label: "Output TPS", helpKey: "out_tps", value: tick?.tps ?? "-", unit: "tok/s" },
    { label: "Success", helpKey: "success_rate", value: tick?.success_rate ?? "-", unit: "%" },
  ];
}

export function getChartTabs(modelTypeValue: string): Array<{ key: ChartMetric; label: string }> {
  return getModelChartTabs(modelTypeValue);
}
