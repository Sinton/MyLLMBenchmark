import { Badge } from "../../../components/common/Badge";
import { Card } from "../../../components/common/Card";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/common/DataTable";
import type { ReportDetail } from "../../../types/api";
import type { StageColumn } from "../types";

type ReportStage = ReportDetail["stages"][number];

type ReportStageTableSectionProps = {
  detail: ReportDetail;
  stageColumns: StageColumn[];
};

export function ReportStageTableSection({
  detail,
  stageColumns,
}: ReportStageTableSectionProps) {
  const hasStopReason = detail.stages.some((stage) => Boolean(stage.stop_reason));
  const statusLabel = (stage: ReportStage) =>
    stage.status === "stable" ? "稳定" : stage.status === "watch" ? "观察" : "失败";
  const statusTone = (stage: ReportStage) =>
    stage.status === "stable" ? "success" : stage.status === "watch" ? "warning" : "danger";
  const columns: Array<DataTableColumn<ReportStage>> = [
    ...stageColumns.map((column) => ({
      key: column.key,
      title: column.label,
      render: column.render,
    })),
    {
      key: "sla",
      title: "SLA",
      render: (stage) => (
        <Badge tone={stage.sla_passed ? "success" : "warning"}>
          {stage.sla_passed ? "达标" : "未达标"}
        </Badge>
      ),
    },
    ...(hasStopReason
      ? [
          {
            key: "stop_reason",
            title: "说明",
            render: (stage: ReportStage) => stage.stop_reason || "-",
          },
        ]
      : []),
    {
      key: "status",
      title: "状态",
      render: (stage) => (
        <Badge tone={statusTone(stage)}>{statusLabel(stage)}</Badge>
      ),
    },
  ];

  return (
    <Card title="阶梯阶段明细">
      <EvidenceSummary detail={detail} />
      <DataTable
        className="report-stage-table"
        columns={columns}
        getRowKey={(stage) => stage.stage_index}
        rows={detail.stages}
      />
    </Card>
  );
}

function EvidenceSummary({ detail }: { detail: ReportDetail }) {
  const planned = detail.planned_stages?.length
    ? detail.planned_stages.join(" -> ")
    : "-";
  const executed = detail.executed_stages?.length
    ? detail.executed_stages.join(" -> ")
    : "-";
  const policy =
    detail.sla_stop_policy === "stop_on_failure" ? "保护性停止" : "继续完整阶梯";

  return (
    <div className="report-stage-evidence">
      <div>
        <span>计划阶梯</span>
        <strong>{planned}</strong>
      </div>
      <div>
        <span>实际执行</span>
        <strong>{executed}</strong>
      </div>
      <div>
        <span>请求配置</span>
        <strong>
          {detail.stage_sample_rounds || "-"} 轮/阶段，超时{" "}
          {detail.request_timeout_seconds || "-"}s
        </strong>
      </div>
      <div>
        <span>SLA 策略</span>
        <strong>{policy}</strong>
      </div>
      {detail.early_stop_reason && (
        <div className="report-stage-evidence-wide">
          <span>停止/异常说明</span>
          <strong>{detail.early_stop_reason}</strong>
        </div>
      )}
    </div>
  );
}
