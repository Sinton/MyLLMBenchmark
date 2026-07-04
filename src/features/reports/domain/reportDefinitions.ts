import { getModelChartTabs } from "../../../domain/modelMetrics";
import { normalizeModelType } from "../../../lib/modelTaxonomy";
import type { ReportDetail } from "../../../types/api";
import type { ChartMetric, StageColumn } from "../types";

export function getModelTypeLabel(type: string) {
  const normalized = normalizeModelType(type);
  const labels: Record<string, string> = {
    text_generation: "文本生成",
    embedding: "向量嵌入",
    rerank: "重排序",
    multimodal: "视觉多模态",
  };
  return labels[normalized] ?? type;
}

export function getReportKpis(detail: ReportDetail) {
  const modelType = normalizeModelType(detail.model_type);
  const common = [
    { label: "推荐生产并发", value: detail.summary.recommended_concurrency, unit: "路", hint: "上线初始限流阈值" },
    { label: "最大稳定并发", value: detail.summary.max_stable_concurrency, unit: "路", hint: "满足 SLA 的最高稳定水位" },
    { label: "P95 延迟", value: detail.summary.p95_latency_ms, unit: "ms", hint: "请求尾延迟" },
    { label: "稳定 QPS", value: detail.stable_qps, unit: "req/s", hint: `成功率 ${detail.summary.success_rate}%` },
  ];

  if (modelType === "embedding") {
    return [
      ...common,
      { label: "Batch Size", value: detail.workload_config.batch_size ?? 16, unit: "条/批", hint: "每个请求文本数量" },
      { label: "Text/s", value: latestStage(detail)?.text_count ?? "-", unit: "条/s", hint: "每秒处理文本条数" },
      { label: "Input Token/s", value: detail.input_tokens, unit: "token/s", hint: "向量化输入吞吐" },
      { label: "运行时长", value: detail.duration_seconds || detail.trends.length, unit: "s", hint: detail.mode },
    ];
  }

  if (modelType === "rerank") {
    return [
      ...common,
      { label: "Docs/Query", value: detail.workload_config.documents_per_query ?? 30, unit: "docs", hint: "每个 query 候选数" },
      { label: "Pair/s", value: latestStage(detail)?.pair_count ?? "-", unit: "pair/s", hint: "query-doc 对吞吐" },
      { label: "TopK", value: detail.workload_config.top_k ?? 10, unit: "docs", hint: "返回结果数量" },
      { label: "运行时长", value: detail.duration_seconds || detail.trends.length, unit: "s", hint: detail.mode },
    ];
  }

  if (modelType === "multimodal") {
    return [
      ...common,
      { label: "Image/s", value: latestTrend(detail) ? Math.round((latestTrend(detail)?.qps ?? 0) * Math.max(1, latestTrend(detail)?.image_count ?? 1)) : "-", unit: "张/s", hint: "图片输入吞吐" },
      { label: "TTFT", value: detail.ttft_ms || "-", unit: detail.ttft_ms ? "ms" : "", hint: "首 token / 首段响应" },
      { label: "Token Throughput", value: detail.token_throughput, unit: "token/s", hint: "图文输入与输出吞吐" },
      { label: "运行时长", value: detail.duration_seconds || detail.trends.length, unit: "s", hint: detail.mode },
    ];
  }

  return [
    ...common,
    { label: "TTFT", value: detail.ttft_ms || "-", unit: detail.ttft_ms ? "ms" : "", hint: "首 token / 首段响应" },
    { label: "Output TPS", value: detail.tps || "-", unit: detail.tps ? "token/s" : "", hint: "输出 token 速度" },
    { label: "Token Throughput", value: detail.token_throughput, unit: "token/s", hint: "输入与输出合计吞吐" },
    { label: "运行时长", value: detail.duration_seconds || detail.trends.length, unit: "s", hint: detail.mode },
  ];
}

export function getReportChartTabs(modelTypeValue: string): Array<{ key: ChartMetric; label: string }> {
  return getModelChartTabs(modelTypeValue, { includeErrors: false });
}

export function getStageColumns(modelTypeValue: string): StageColumn[] {
  const modelType = normalizeModelType(modelTypeValue);
  const commonStart: StageColumn[] = [
    { key: "stage", label: "阶段", render: (stage) => `#${stage.stage_index}` },
    { key: "concurrency", label: "并发", render: (stage) => stage.concurrency },
    { key: "rounds", label: "轮次", render: (stage) => stage.sample_rounds || "-" },
    { key: "requests", label: "请求数", render: (stage) => stage.request_count || "-" },
    {
      key: "success-failure",
      label: "成功/失败",
      render: (stage) => `${stage.success_count || 0}/${stage.failure_count || 0}`,
    },
  ];
  const commonEnd: StageColumn[] = [
    { key: "p95", label: "P95", render: (stage) => `${stage.p95_latency_ms}ms` },
    { key: "success", label: "成功率", render: (stage) => `${stage.success_rate}%` },
  ];

  if (modelType === "embedding") {
    return [
      ...commonStart,
      { key: "qps", label: "QPS", render: (stage) => stage.qps },
      { key: "batch", label: "Batch", render: (stage) => stage.batch_size || "-" },
      { key: "text", label: "Text/s", render: (stage) => stage.text_count || "-" },
      { key: "input", label: "Input/s", render: (stage) => stage.input_tokens },
      ...commonEnd,
    ];
  }

  if (modelType === "rerank") {
    return [
      ...commonStart,
      { key: "qps", label: "Query/s", render: (stage) => stage.qps },
      { key: "docs", label: "Docs/Q", render: (stage) => stage.documents_per_query || "-" },
      { key: "pair", label: "Pair/s", render: (stage) => stage.pair_count || "-" },
      ...commonEnd,
    ];
  }

  if (modelType === "multimodal") {
    return [
      ...commonStart,
      { key: "qps", label: "QPS", render: (stage) => stage.qps },
      { key: "image", label: "Image/s", render: (stage) => Math.round(stage.qps * Math.max(1, stage.image_count)) },
      { key: "ttft", label: "TTFT", render: (stage) => `${stage.ttft_ms}ms` },
      ...commonEnd,
    ];
  }

  return [
    ...commonStart,
    { key: "qps", label: "QPS", render: (stage) => stage.qps },
    { key: "p95", label: "P95", render: (stage) => `${stage.p95_latency_ms}ms` },
    { key: "ttft", label: "TTFT", render: (stage) => `${stage.ttft_ms}ms` },
    { key: "tps", label: "Out TPS", render: (stage) => stage.tps || "-" },
    { key: "tokens", label: "Token/s", render: (stage) => stage.total_tokens },
    { key: "success", label: "成功率", render: (stage) => `${stage.success_rate}%` },
  ];
}

export function formatDate(value: string) {
  return new Date(value).toLocaleString("zh-CN", { hour12: false });
}

function latestStage(detail: ReportDetail) {
  return detail.stages.at(-1);
}

function latestTrend(detail: ReportDetail) {
  return detail.trends.at(-1);
}
