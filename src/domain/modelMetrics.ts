import { normalizeModelType } from "../lib/modelTaxonomy";

export type ChartMetric = "latency" | "ttft" | "qps" | "tps" | "success" | "errors";

export type ChartMetricTab = {
  key: ChartMetric;
  label: string;
};

const textChartTabs: ChartMetricTab[] = [
  { key: "latency", label: "Latency" },
  { key: "ttft", label: "TTFT" },
  { key: "qps", label: "QPS" },
  { key: "tps", label: "TPS" },
  { key: "success", label: "Success" },
  { key: "errors", label: "Error" },
];

const embeddingChartTabs: ChartMetricTab[] = [
  { key: "qps", label: "QPS" },
  { key: "tps", label: "Input Token/s" },
  { key: "latency", label: "Latency" },
  { key: "success", label: "Success" },
  { key: "errors", label: "Error" },
];

const rerankChartTabs: ChartMetricTab[] = [
  { key: "qps", label: "Query/s" },
  { key: "tps", label: "Pair/s" },
  { key: "latency", label: "Latency" },
  { key: "success", label: "Success" },
  { key: "errors", label: "Error" },
];

export function getModelChartTabs(
  modelTypeValue: string,
  options: { includeErrors?: boolean } = {},
): ChartMetricTab[] {
  const modelType = normalizeModelType(modelTypeValue);
  const includeErrors = options.includeErrors ?? true;
  const tabs =
    modelType === "embedding"
      ? embeddingChartTabs
      : modelType === "rerank"
        ? rerankChartTabs
        : textChartTabs;

  return includeErrors ? tabs : tabs.filter((tab) => tab.key !== "errors");
}
