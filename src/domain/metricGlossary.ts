export type MetricGlossaryKey =
  | "qps"
  | "query_s"
  | "p95"
  | "ttft"
  | "latency"
  | "out_tps"
  | "token_s"
  | "success_rate"
  | "sla"
  | "batch"
  | "text_s"
  | "input_s"
  | "docs_q"
  | "pair_s"
  | "image_s";

export const metricGlossary: Record<MetricGlossaryKey, string> = {
  qps: "QPS（Queries Per Second）：每秒完成的请求数，用于衡量接口请求吞吐。",
  query_s: "Query/s：每秒完成的查询数，Rerank 场景下等价于每秒处理 query 的能力。",
  p95: "P95：95% 请求的耗时不超过该值，比平均值更能反映尾延迟。",
  ttft:
    "TTFT（Time To First Token）：从请求发出到收到首个输出 token 的耗时，Streaming 场景下代表模型开始响应的速度；非 Streaming 场景下按完整响应耗时近似。",
  latency: "Latency：请求从发出到完成的总耗时，包含排队、推理和返回结果时间。",
  out_tps: "Output TPS：每秒输出 token 数，主要衡量生成阶段的输出速度。",
  token_s: "Token/s：token 吞吐，文本生成通常表示总 token 或输出 token 吞吐，Embedding/Rerank 以输入侧吞吐为主。",
  success_rate: "成功率：成功请求数 / 总请求数，timeout、HTTP 错误和解析错误会计入失败。",
  sla: "SLA：本次压测配置的服务等级阈值，当前阶段会按 P95 延迟和最低成功率判断是否达标。",
  batch: "Batch：单次 Embedding 请求中包含的文本条数。",
  text_s: "Text/s：Embedding 每秒处理的文本条数。",
  input_s: "Input/s：每秒处理的输入 token 数，常用于 Embedding 和 Rerank 的输入吞吐。",
  docs_q: "Docs/Q：每个 Rerank query 携带的候选文档数量。",
  pair_s: "Pair/s：每秒处理的 query-doc 对数量，是 Rerank 的核心吞吐指标。",
  image_s: "Image/s：每秒处理的图片数量，等于 QPS 与每请求图片数的组合吞吐。",
};

export function getMetricHelp(key?: string) {
  if (!key) return undefined;
  return metricGlossary[key as MetricGlossaryKey];
}
