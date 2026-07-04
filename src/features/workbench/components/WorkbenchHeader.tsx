import { Badge, statusLabel, statusTone } from "../../../components/common/Badge";
import { Button } from "../../../components/common/Button";
import { WorkspaceHeader } from "../../../components/common/WorkspaceHeader";
import { FileText, Square } from "../../../components/common/icons";
import type { BenchmarkTaskSummary } from "../../../types/api";

type WorkbenchHeaderProps = {
  activeTask: BenchmarkTaskSummary | null;
  canStop: boolean;
  canGenerateReport: boolean;
  stopPending: boolean;
  reportPending: boolean;
  onStop: () => void;
  onGenerateReport: () => void;
};

export function WorkbenchHeader({
  activeTask,
  canStop,
  canGenerateReport,
  stopPending,
  reportPending,
  onStop,
  onGenerateReport,
}: WorkbenchHeaderProps) {
  return (
    <WorkspaceHeader
      breadcrumb="控制台"
      title="压测工作台"
      subtitle="配置任务、观察实时指标，并在同一工作台生成交付报告。"
      actions={
        <>
          {activeTask && (
            <Badge tone={statusTone(activeTask.status)}>
              {statusLabel(activeTask.status)}
            </Badge>
          )}
          <Button
            disabled={!canStop}
            icon={<Square size={15} />}
            loading={stopPending}
            onClick={onStop}
          >
            {stopPending || activeTask?.status === "stopping" ? "停止中" : "停止"}
          </Button>
          <Button
            disabled={!canGenerateReport || reportPending}
            icon={<FileText size={15} />}
            loading={reportPending}
            onClick={onGenerateReport}
            variant="primary"
          >
            生成报告
          </Button>
        </>
      }
    />
  );
}
