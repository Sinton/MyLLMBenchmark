import { statusLabel } from "../../../domain/statusPresentation";
import type {
  BenchmarkTaskSummary,
  DatasetSummary,
  ModelSummary,
  ProviderSummary,
} from "../../../types/api";
import type { WorkbenchForm } from "../types";

type TaskSummaryPanelProps = {
  form: WorkbenchForm;
  provider?: ProviderSummary;
  model?: ModelSummary;
  dataset?: DatasetSummary;
  historyTask?: BenchmarkTaskSummary | null;
  isStaircase: boolean;
  stageSequence: number[];
};

export function TaskSummaryPanel({
  form,
  provider,
  model,
  dataset,
  historyTask,
  isStaircase,
  stageSequence,
}: TaskSummaryPanelProps) {
  if (historyTask) {
    return <HistoryTaskSummary task={historyTask} />;
  }

  const estimatedRequests = estimateRequests(form, isStaircase, stageSequence);
  const stageCount = isStaircase ? stageSequence.length : 1;
  const targetTitle = `${provider?.name ?? "未选择服务商"} / ${
    model?.name ?? "未选择模型"
  }`;
  const datasetTitle = dataset?.name ?? "未选择数据集";

  return (
    <section className="task-summary-strip" aria-label="启动前摘要">
      <div className="task-summary-primary">
        <span>启动前确认</span>
        <strong title={targetTitle}>{targetTitle}</strong>
        <em title={datasetTitle}>{datasetTitle}</em>
      </div>

      <div className="task-summary-chips" aria-label="关键参数">
        <SummaryChip value={form.mode} title={`压测模式：${form.mode}`} />
        <SummaryChip
          value={concurrencySummary(form, isStaircase)}
          title={`并发策略：${concurrencySummary(form, isStaircase)}`}
        />
        <SummaryChip
          value={`${stageCount} 阶段`}
          title={`计划阶段数：${stageCount}`}
        />
        <SummaryChip
          value={`${formatNumber(estimatedRequests)} 请求`}
          title={`预计请求数：${formatNumber(estimatedRequests)}`}
        />
        <SummaryChip
          value={`P95 <= ${form.sla_p95_ms}ms`}
          title={`SLA：P95 延迟不超过 ${form.sla_p95_ms}ms`}
        />
        <SummaryChip
          value={`Timeout ${form.request_timeout_seconds}s`}
          title={`单请求超时：${form.request_timeout_seconds}s`}
        />
        <SummaryChip
          tone={form.request_log_enabled && form.request_log_capture_body ? "warning" : "default"}
          value={requestLogSummary(form)}
          title={`证据采集：${requestLogSummary(form)}`}
        />
      </div>
    </section>
  );
}

function HistoryTaskSummary({ task }: { task: BenchmarkTaskSummary }) {
  const targetTitle = `${task.provider_name} / ${task.model_name}`;

  return (
    <section
      className="task-summary-strip task-summary-strip-history"
      aria-label="历史任务摘要"
    >
      <div className="task-summary-primary">
        <span>历史任务</span>
        <strong title={targetTitle}>{targetTitle}</strong>
        <em title={task.dataset_name}>{task.dataset_name}</em>
      </div>
      <div className="task-summary-chips" aria-label="历史任务关键指标">
        <SummaryChip
          value={statusLabel(task.status)}
          title={`任务状态：${statusLabel(task.status)}`}
        />
        <SummaryChip
          value={`Goodput ${formatDecimal(task.goodput_qps)} qps`}
          title={`有效吞吐：${formatDecimal(task.goodput_qps)} qps`}
        />
        <SummaryChip
          value={`P95 ${task.p95_latency_ms} ms`}
          title={`P95 延迟：${task.p95_latency_ms} ms`}
        />
        <SummaryChip
          value={`成功率 ${formatPercent(task.success_rate)}%`}
          title={`成功率：${formatPercent(task.success_rate)}%`}
        />
      </div>
    </section>
  );
}

function SummaryChip({
  tone = "default",
  title,
  value,
}: {
  tone?: "default" | "warning";
  title: string;
  value: string;
}) {
  return (
    <span
      className={`task-summary-chip ${tone === "warning" ? "warning" : ""}`}
      title={title}
    >
      {value}
    </span>
  );
}

function concurrencySummary(form: WorkbenchForm, isStaircase: boolean) {
  if (isStaircase) {
    return `${form.start_concurrency}-${form.end_concurrency} 并发`;
  }
  return `${form.concurrency} 并发`;
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

function requestLogSummary(form: WorkbenchForm) {
  if (!form.request_log_enabled) return "不保存明细";
  if (form.request_log_capture_body) return "保存正文";
  return "保存索引";
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
