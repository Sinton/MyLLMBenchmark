import { RealtimeChart } from "../../../charts/RealtimeChart";
import { Card } from "../../../components/ui/Card";
import { MetricCard } from "../../../components/ui/MetricCard";
import { MetricHelp } from "../../../components/common/MetricHelp";
import { Tabs } from "../../../components/ui/Tabs";
import type { BenchmarkTaskSummary, MetricsTick } from "../../../types/api";
import type { ChartMetric } from "../types";

type RealtimeTrendPanelProps = {
  activeTask: BenchmarkTaskSummary | null;
  ticks: MetricsTick[];
  latestTick: MetricsTick | null;
  metricCards: Array<{ label: string; helpKey?: string; value: string | number; unit?: string }>;
  historyError?: unknown | null;
  historyLoading: boolean;
  isHistoryView: boolean;
  startPending: boolean;
  tabs: Array<{ key: ChartMetric; label: string }>;
  chartMetric: ChartMetric;
  onChartMetricChange: (metric: ChartMetric) => void;
};

export function RealtimeTrendPanel({
  activeTask,
  ticks,
  latestTick,
  metricCards,
  historyError,
  historyLoading,
  isHistoryView,
  startPending,
  tabs,
  chartMetric,
  onChartMetricChange,
}: RealtimeTrendPanelProps) {
  const emptyState = getChartEmptyState(activeTask, startPending, {
    historyError,
    historyLoading,
    isHistoryView,
  });
  const trendNote = getTrendNote(activeTask, latestTick, ticks.length, isHistoryView);

  return (
    <div className="workbench-main">
      <div className="workbench-metrics">
        {metricCards.map((metric) => (
          <MetricCard
            key={metric.label}
            label={<MetricHelp helpKey={metric.helpKey}>{metric.label}</MetricHelp>}
            unit={metric.unit}
            value={metric.value}
          />
        ))}
      </div>

      <Card
        title="实时趋势"
        eyebrow="Realtime"
        action={<Tabs items={tabs} onChange={onChartMetricChange} value={chartMetric} />}
      >
        <RealtimeChart data={ticks} emptyState={emptyState} metric={chartMetric} />
        {trendNote && <div className="chart-status-note">{trendNote}</div>}
      </Card>
    </div>
  );
}

function getChartEmptyState(
  activeTask: BenchmarkTaskSummary | null,
  startPending: boolean,
  history: {
    historyError?: unknown | null;
    historyLoading: boolean;
    isHistoryView: boolean;
  },
) {
  if (history.historyLoading) {
    return {
      title: "正在加载历史任务",
      description: "正在从本地数据源读取任务摘要和持久化指标。",
      tone: "loading" as const,
    };
  }

  if (history.historyError) {
    return {
      title: "历史任务加载失败",
      description: getErrorMessage(history.historyError),
      tone: "idle" as const,
    };
  }

  if (startPending) {
    return {
      title: "正在创建压测任务",
      description: "配置已经提交，正在建立后端任务和实时指标通道。",
      tone: "loading" as const,
    };
  }

  if (activeTask?.status === "running") {
    return {
      title: "任务已启动，等待首批实时指标",
      description: "Mock 压测通常会在 1 秒内推送第一批指标，随后趋势图会自动刷新。",
      tone: "waiting" as const,
    };
  }

  if (activeTask?.status === "stopping") {
    return {
      title: "正在停止压测",
      description: "停止请求已发出，等待后端确认后会冻结当前指标。",
      tone: "loading" as const,
    };
  }

  if (history.isHistoryView && activeTask) {
    return {
      title: "该历史任务没有可回放指标",
      description:
        "这个任务没有持久化 tick，可能是任务初始化失败、旧版本任务，或 Mock 内存数据已丢失。",
      tone: "idle" as const,
    };
  }

  return {
    title: "尚未开始压测",
    description: "选择服务商、模型和数据集后点击开始压测，这里会展示实时趋势。",
    tone: "idle" as const,
  };
}

function getTrendNote(
  activeTask: BenchmarkTaskSummary | null,
  latestTick: MetricsTick | null,
  tickCount: number,
  isHistoryView: boolean,
) {
  if (!tickCount) return null;

  if (isHistoryView) {
    return "历史任务已加载，当前趋势图展示后端持久化指标；事件日志只显示当前会话信息。";
  }

  if (activeTask?.status === "cancelled") {
    return `任务已停止，展示停止前第 ${latestTick?.elapsed_seconds ?? tickCount} 轮的最后指标数据。`;
  }

  if (activeTask?.status === "completed") {
    return "压测已完成，当前趋势图保留本次任务的完整指标数据。";
  }

  return null;
}

function getErrorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "任务不存在或当前数据源不可用。";
}
