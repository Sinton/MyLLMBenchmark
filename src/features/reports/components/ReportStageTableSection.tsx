import { useEffect, useState } from "react";
import { Badge } from "../../../components/ui/Badge";
import { Card } from "../../../components/ui/Card";
import {
  DataTable,
  type DataTableRowKey,
  type DataTableColumn,
} from "../../../components/ui/DataTable";
import { MetricHelp } from "../../../components/common/MetricHelp";
import type { ReportDetail } from "../../../types/api";
import type { StageColumn } from "../types";
import { ReportStageRequestPanel } from "./ReportStageRequestPanel";

type ReportStage = ReportDetail["stages"][number];

type ReportStageTableSectionProps = {
  detail: ReportDetail;
  stageColumns: StageColumn[];
};

export function ReportStageTableSection({
  detail,
  stageColumns,
}: ReportStageTableSectionProps) {
  const [expandedStageIndex, setExpandedStageIndex] = useState<number | null>(null);
  const hasStopReason = detail.stages.some((stage) => Boolean(stage.stop_reason));
  const hasRequestEvidence = detail.request_log_meta.total_records > 0;
  const canExpandStage = (stage: ReportStage) =>
    hasRequestEvidence && stage.request_count > 0;

  useEffect(() => {
    setExpandedStageIndex(null);
  }, [detail.summary.task_id]);

  const statusLabel = (stage: ReportStage) =>
    stage.status === "stable" ? "稳定" : stage.status === "watch" ? "观察" : "失败";
  const statusTone = (stage: ReportStage) =>
    stage.status === "stable" ? "success" : stage.status === "watch" ? "warning" : "danger";
  const columns: Array<DataTableColumn<ReportStage>> = [
    ...stageColumns.map((column) => ({
      key: column.key,
      title: <MetricHelp helpKey={column.helpKey}>{column.label}</MetricHelp>,
      render: column.render,
    })),
    {
      key: "sla",
      title: <MetricHelp helpKey="sla">SLA</MetricHelp>,
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
      {!hasRequestEvidence && <RequestEvidenceNote detail={detail} />}
      <DataTable
        className="report-stage-table"
        columns={columns}
        expandable={
          hasRequestEvidence
            ? {
                expandedRowKey: expandedStageIndex,
                expandedRowRender: (stage) => (
                  <ReportStageRequestPanel detail={detail} stage={stage} />
                ),
                expandOnRowClick: true,
                onExpandedRowChange: (key: DataTableRowKey | null) => {
                  setExpandedStageIndex(key == null ? null : Number(key));
                },
                rowExpandable: canExpandStage,
              }
            : undefined
        }
        getRowKey={(stage) => stage.stage_index}
        rows={detail.stages}
      />
    </Card>
  );
}

function RequestEvidenceNote({ detail }: { detail: ReportDetail }) {
  return (
    <div className="report-request-evidence-note">
      <strong>
        {detail.request_log_meta.enabled
          ? "本报告没有可展开的请求证据"
          : "本次压测未采集请求明细"}
      </strong>
      <span>
        请求/响应正文只有在压测开始前开启“保存请求/响应明细”后才会写入，
        历史缺失数据无法补录。
      </span>
    </div>
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
