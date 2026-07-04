import { Card } from "../../../components/common/Card";
import { MetricCard } from "../../../components/common/MetricCard";
import { Tabs } from "../../../components/common/Tabs";
import { RealtimeChart } from "../../../charts/RealtimeChart";
import type { BenchmarkTaskSummary, MetricsTick } from "../../../types/api";
import type { ChartMetric } from "../types";

type RealtimeTrendPanelProps = {
  activeTask: BenchmarkTaskSummary | null;
  ticks: MetricsTick[];
  latestTick: MetricsTick | null;
  metricCards: Array<{ label: string; value: string | number; unit?: string }>;
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
  startPending,
  tabs,
  chartMetric,
  onChartMetricChange,
}: RealtimeTrendPanelProps) {
  const emptyState = getChartEmptyState(activeTask, startPending);
  const trendNote = getTrendNote(activeTask, latestTick, ticks.length);

  return (
    <div className="workbench-main">
      <div className="workbench-metrics">
        {metricCards.map((metric) => (
          <MetricCard
            key={metric.label}
            label={metric.label}
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
) {
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
) {
  if (!tickCount) return null;

  if (activeTask?.status === "cancelled") {
    return `任务已停止，展示停止前第 ${latestTick?.elapsed_seconds ?? tickCount} 轮的最后指标数据。`;
  }

  if (activeTask?.status === "completed") {
    return "压测已完成，当前趋势图保留本次任务的完整指标数据。";
  }

  return null;
}
