import { Card } from "../../../components/ui/Card";
import { Disclosure } from "../../../components/ui/Disclosure";
import { statusLabel } from "../../../domain/statusPresentation";
import { getModelTypeLabel } from "../../../lib/modelTaxonomy";
import type {
  BenchmarkTaskSummary,
  DatasetSummary,
  ModelSummary,
  ProviderSummary,
} from "../../../types/api";
import type { WorkbenchForm } from "../types";
import { RequestPreviewPanel } from "./RequestPreviewPanel";

type TaskSummaryPanelProps = {
  form: WorkbenchForm;
  provider?: ProviderSummary;
  model?: ModelSummary;
  dataset?: DatasetSummary;
  historyTask?: BenchmarkTaskSummary | null;
  modelType: string;
  isStaircase: boolean;
  stageSequence: number[];
  estimatedSeconds: number;
};

export function TaskSummaryPanel({
  form,
  provider,
  model,
  dataset,
  historyTask,
  modelType,
  isStaircase,
  stageSequence,
  estimatedSeconds,
}: TaskSummaryPanelProps) {
  if (historyTask) {
    return <HistoryTaskSummary task={historyTask} />;
  }

  const estimatedRequests = estimateRequests(form, isStaircase, stageSequence);
  const estimatedTokens = estimateTokens(
    form,
    dataset,
    modelType,
    estimatedRequests,
  );

  return (
    <Card title="任务摘要" eyebrow="压测计划" className="task-summary-card">
      <div className="summary-section">
        <h3>测试对象</h3>
        <SummaryRow label="服务商" value={provider?.name ?? "未选择"} />
        <SummaryRow label="模型" value={model?.name ?? "未选择"} />
        <SummaryRow label="数据集" value={dataset?.name ?? "未选择"} />
      </div>

      <div className="summary-section">
        <h3>执行计划</h3>
        <SummaryRow label="模式" value={form.mode} />
        <SummaryRow
          label="阶段序列"
          value={
            isStaircase
              ? stageSequence.join(" -> ")
              : `${form.concurrency} 并发`
          }
          strong
        />
        <SummaryRow
          label="请求轮次"
          value={`${estimatedSeconds} 轮（含预热 ${form.warmup_rounds} 轮/阶段）`}
        />
        <SummaryRow label="预计请求" value={formatNumber(estimatedRequests)} />
        <SummaryRow label="预计 Token" value={formatNumber(estimatedTokens)} />
      </div>

      <div className="summary-metrics">
        <div>
          <span>数据集样本</span>
          <strong>{dataset?.sample_count ?? "-"}</strong>
        </div>
        <div>
          <span>平均 Token</span>
          <strong>{dataset?.average_tokens ?? "-"}</strong>
        </div>
      </div>

      <div className="summary-section">
        <h3>SLA 与运行策略</h3>
        <SummaryRow
          label="SLA"
          value={`P95 <= ${form.sla_p95_ms}ms，成功率 >= ${form.min_success_rate}%`}
        />
        <SummaryRow
          label="请求超时"
          value={`${form.request_timeout_seconds}s；超时前不会主动丢弃已发请求`}
        />
        <SummaryRow label="失败策略" value={slaPolicyLabel(form.sla_stop_policy)} />
        <SummaryRow label="证据采集" value={requestLogSummary(form)} />
        <SummaryRow label="模型类型" value={getModelTypeLabel(modelType)} />
        <SummaryRow label="专项负载" value={workloadSummary(form, modelType)} />
      </div>

      <Disclosure
        className="debug-payload-disclosure"
        description="仅用于排障参考，默认不参与主流程判断"
        title="调试 Payload"
      >
        <RequestPreviewPanel
          dataset={dataset}
          form={form}
          model={model}
          modelType={modelType}
          provider={provider}
        />
      </Disclosure>
    </Card>
  );
}

function HistoryTaskSummary({ task }: { task: BenchmarkTaskSummary }) {
  return (
    <Card title="任务摘要" eyebrow="历史任务" className="task-summary-card">
      <div className="summary-section">
        <h3>测试对象</h3>
        <SummaryRow label="服务商" value={task.provider_name} />
        <SummaryRow label="模型" value={task.model_name} />
        <SummaryRow label="数据集" value={task.dataset_name} />
        <SummaryRow label="模型类型" value={getModelTypeLabel(task.model_type)} />
      </div>

      <div className="summary-section">
        <h3>执行结果</h3>
        <SummaryRow label="任务状态" value={statusLabel(task.status)} strong />
        <SummaryRow label="并发" value={task.concurrency} />
        <SummaryRow label="Goodput" value={`${formatNumber(task.goodput_qps)} qps`} />
        <SummaryRow label="P95" value={`${task.p95_latency_ms} ms`} />
        <SummaryRow label="成功率" value={`${formatPercent(task.success_rate)}%`} />
      </div>

      <div className="summary-section">
        <h3>历史信息</h3>
        <SummaryRow label="任务名称" value={task.name} />
        <SummaryRow label="创建时间" value={formatDate(task.created_at)} />
      </div>
    </Card>
  );
}

type SummaryRowProps = {
  label: string;
  value: string | number;
  strong?: boolean;
};

function SummaryRow({ label, value, strong = false }: SummaryRowProps) {
  return (
    <div className={`summary-row ${strong ? "summary-row-strong" : ""}`}>
      <span>{label}</span>
      <strong title={String(value)}>{value}</strong>
    </div>
  );
}

function estimateRequests(
  form: WorkbenchForm,
  isStaircase: boolean,
  stageSequence: number[],
) {
  if (isStaircase) {
    const sampleRounds = Number(form.stage_sample_rounds) || 0;
    return stageSequence.reduce(
      (total, concurrency) => total + concurrency * sampleRounds,
      0,
    );
  }
  return (Number(form.concurrency) || 0) * (Number(form.duration_seconds) || 0);
}

function estimateTokens(
  form: WorkbenchForm,
  dataset: DatasetSummary | undefined,
  modelType: string,
  estimatedRequests: number,
) {
  const inputTokens = Number(dataset?.average_tokens ?? 0);
  const outputTokens =
    modelType === "text_generation" ? form.max_output_tokens : 0;
  return estimatedRequests * (inputTokens + outputTokens);
}

function workloadSummary(form: WorkbenchForm, modelType: string) {
  if (modelType === "embedding") {
    return `Batch ${form.embedding_batch_size}，文本/请求 ${form.embedding_text_count_per_request}`;
  }
  if (modelType === "rerank") {
    return `Docs/Query ${form.rerank_documents_per_query}，TopK ${form.rerank_top_k}`;
  }
  if (modelType === "multimodal") {
    return `${imageProfileLabel(form.vision_image_profile)}，${form.vision_image_count} 张/请求`;
  }
  return `Max Output ${form.max_output_tokens}，${
    form.streaming ? "Streaming 开启" : "Streaming 关闭"
  }，Prompt ${promptProfileLabel(form.prompt_profile)}`;
}

function slaPolicyLabel(value: WorkbenchForm["sla_stop_policy"]) {
  return value === "stop_on_failure" ? "保护性停止" : "继续完整阶梯";
}

function requestLogSummary(form: WorkbenchForm) {
  if (!form.request_log_enabled) return "不保存请求明细";
  if (form.request_log_capture_body) return "保存明细索引 + 正文";
  return "保存明细索引";
}

function promptProfileLabel(value: string) {
  return value === "short" ? "短" : value === "long" ? "长" : "混合";
}

function imageProfileLabel(value: string) {
  return value === "small" ? "小图" : value === "large" ? "大图" : "中等图";
}

function formatNumber(value: number) {
  return new Intl.NumberFormat("zh-CN").format(Math.max(0, Math.round(value)));
}

function formatPercent(value: number) {
  return Number.isFinite(value) ? value.toFixed(2) : "0.00";
}

function formatDate(value: string) {
  return new Date(value).toLocaleString("zh-CN", { hour12: false });
}
