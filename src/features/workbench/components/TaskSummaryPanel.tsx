import type { ReactNode } from "react";
import { Card } from "../../../components/ui/Card";
import { Disclosure } from "../../../components/ui/Disclosure";
import { InlineAlert } from "../../../components/ui/InlineAlert";
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
  const stageCount = isStaircase ? stageSequence.length : 1;

  return (
    <Card title="任务摘要" eyebrow="启动前确认" className="task-summary-card">
      <SummaryHero
        label="本次计划"
        meta={`${formatNumber(estimatedRequests)} 请求 · Timeout ${form.request_timeout_seconds}s`}
        title={`${form.mode} · ${stageCount} 阶段`}
      />

      <SummaryBlock title="测试对象">
        <SummaryItem label="服务商" value={provider?.name ?? "未选择"} />
        <SummaryItem label="模型" value={model?.name ?? "未选择"} strong />
        <SummaryItem label="数据集" value={dataset?.name ?? "未选择"} />
      </SummaryBlock>

      <SummaryBlock title="执行计划">
        <SummaryItem label="模式" value={form.mode} />
        <StageSequence
          concurrency={form.concurrency}
          isStaircase={isStaircase}
          stages={stageSequence}
        />
        <SummaryItem
          label={isStaircase ? "每阶段" : "请求轮次"}
          value={
            isStaircase
              ? `${form.stage_sample_rounds} 轮请求 + ${form.warmup_rounds} 轮预热`
              : `${form.duration_seconds} 轮`
          }
        />
      </SummaryBlock>

      <SummaryBlock title="运行保护">
        <SummaryItem
          label="SLA"
          value={`P95 <= ${form.sla_p95_ms}ms / 成功率 >= ${form.min_success_rate}%`}
        />
        <SummaryItem label="Timeout" value={`${form.request_timeout_seconds}s`} />
        <SummaryItem label="策略" value={slaPolicyLabel(form.sla_stop_policy)} />
      </SummaryBlock>

      {form.request_log_enabled && form.request_log_capture_body && (
        <InlineAlert tone="warning" title="正文采集已开启">
          本次会在本地保存 Prompt 和响应正文，请确认样本中不包含敏感信息。
        </InlineAlert>
      )}

      <Disclosure
        className="summary-detail-disclosure"
        description="Token、专项负载和证据采集"
        title="更多参数"
      >
        <div className="summary-detail-grid">
          <SummaryMetric label="数据集样本" value={dataset?.sample_count ?? "-"} />
          <SummaryMetric label="平均 Token" value={dataset?.average_tokens ?? "-"} />
          <SummaryMetric label="预计 Token" value={formatNumber(estimatedTokens)} />
          <SummaryMetric label="轮次合计" value={`${estimatedSeconds} 轮`} />
        </div>
        <div className="summary-section">
          <SummaryItem label="模型类型" value={getModelTypeLabel(modelType)} />
          <SummaryItem label="专项负载" value={workloadSummary(form, modelType)} />
          <SummaryItem label="证据采集" value={requestLogSummary(form)} />
        </div>
      </Disclosure>

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
      <SummaryHero
        label="历史状态"
        meta={`${task.provider_name} · ${task.model_name}`}
        title={statusLabel(task.status)}
      />

      <div className="summary-detail-grid">
        <SummaryMetric label="Goodput" value={`${formatDecimal(task.goodput_qps)} qps`} />
        <SummaryMetric label="P95" value={`${task.p95_latency_ms} ms`} />
        <SummaryMetric label="成功率" value={`${formatPercent(task.success_rate)}%`} />
        <SummaryMetric label="并发" value={task.concurrency} />
      </div>

      <SummaryBlock title="测试对象">
        <SummaryItem label="服务商" value={task.provider_name} />
        <SummaryItem label="模型" value={task.model_name} strong />
        <SummaryItem label="数据集" value={task.dataset_name} />
      </SummaryBlock>

      <Disclosure
        className="summary-detail-disclosure"
        description="任务名称、创建时间和模型类型"
        title="历史详情"
      >
        <div className="summary-section">
          <SummaryItem label="任务名称" value={task.name} />
          <SummaryItem label="创建时间" value={formatDate(task.created_at)} />
          <SummaryItem label="模型类型" value={getModelTypeLabel(task.model_type)} />
        </div>
      </Disclosure>
    </Card>
  );
}

function SummaryHero({
  label,
  meta,
  title,
}: {
  label: string;
  meta: string;
  title: string;
}) {
  return (
    <div className="summary-hero">
      <span>{label}</span>
      <strong>{title}</strong>
      <em>{meta}</em>
    </div>
  );
}

function SummaryBlock({ children, title }: { children: ReactNode; title: string }) {
  return (
    <div className="summary-section">
      <h3>{title}</h3>
      {children}
    </div>
  );
}

type SummaryItemProps = {
  label: string;
  value: string | number;
  strong?: boolean;
};

function SummaryItem({ label, value, strong = false }: SummaryItemProps) {
  return (
    <div className={`summary-row ${strong ? "summary-row-strong" : ""}`}>
      <span>{label}</span>
      <strong title={String(value)}>{value}</strong>
    </div>
  );
}

function SummaryMetric({
  label,
  value,
}: {
  label: string;
  value: string | number;
}) {
  return (
    <div>
      <span>{label}</span>
      <strong title={String(value)}>{value}</strong>
    </div>
  );
}

function StageSequence({
  concurrency,
  isStaircase,
  stages,
}: {
  concurrency: number;
  isStaircase: boolean;
  stages: number[];
}) {
  const chips = isStaircase ? stages : [concurrency];

  return (
    <div className="summary-plan-chips" aria-label="阶段序列">
      {chips.map((stage, index) => (
        <span key={`${stage}-${index}`}>{stage}</span>
      ))}
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

function formatDecimal(value: number) {
  return Number.isFinite(value) ? value.toFixed(2) : "0.00";
}

function formatPercent(value: number) {
  return Number.isFinite(value) ? value.toFixed(2) : "0.00";
}

function formatDate(value: string) {
  return new Date(value).toLocaleString("zh-CN", { hour12: false });
}
