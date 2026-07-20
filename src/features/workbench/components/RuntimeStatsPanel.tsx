import { Card } from "../../../components/ui/Card";
import { Wand2 } from "../../../components/ui/icons";
import { statusLabel } from "../../../domain/statusPresentation";
import type {
  BenchmarkTaskSummary,
  MetricsTick,
  ReportSummary,
  StageChangedEvent,
} from "../../../types/api";

type RuntimeStatsPanelProps = {
  activeTask: BenchmarkTaskSummary | null;
  currentStage: StageChangedEvent | null;
  generatedReport: ReportSummary | null;
  latestTick: MetricsTick | null;
};

export function RuntimeStatsPanel({
  activeTask,
  currentStage,
  generatedReport,
  latestTick,
}: RuntimeStatsPanelProps) {
  return (
    <>
      <Card title="运行状态" eyebrow="状态检查器">
        <div className="side-stat">
          <span>当前任务</span>
          <strong>{activeTask?.name ?? "未开始"}</strong>
        </div>
        <div className="side-stat">
          <span>任务状态</span>
          <strong>{activeTask ? statusLabel(activeTask.status) : "空闲"}</strong>
        </div>
        <div className="side-stat">
          <span>当前阶段</span>
          <strong>
            {currentStage?.stage_index != null && currentStage?.stage_total
              ? `${currentStage.stage_index}/${currentStage.stage_total}`
              : "-"}
          </strong>
        </div>
        <div className="side-stat">
          <span>当前并发</span>
          <strong>
            {currentStage?.concurrency ??
              latestTick?.in_flight ??
              activeTask?.concurrency ??
              "-"}
          </strong>
        </div>
        <div className="side-stat">
          <span>完成请求</span>
          <strong>{latestTick?.request_count ?? "-"}</strong>
        </div>
        <div className="side-stat">
          <span>成功率</span>
          <strong>
            {latestTick ? `${latestTick.success_rate.toFixed(2)}%` : "-"}
          </strong>
        </div>
        <div className="side-stat">
          <span>错误数</span>
          <strong>{latestTick?.errors ?? 0}</strong>
        </div>
        <div className="side-stat">
          <span>报告状态</span>
          <strong>{generatedReport ? "已生成" : "未生成"}</strong>
        </div>
      </Card>

      {generatedReport && (
        <Card title="报告已生成" eyebrow="测试报告">
          <div className="report-mini">
            <Wand2 size={18} />
            <div>
              <strong>推荐并发 {generatedReport.recommended_concurrency}</strong>
              <span>{generatedReport.recommendation}</span>
            </div>
          </div>
        </Card>
      )}
    </>
  );
}
