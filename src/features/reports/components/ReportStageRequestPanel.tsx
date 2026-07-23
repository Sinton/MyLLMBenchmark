import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import { Badge } from "../../../components/ui/Badge";
import { Button } from "../../../components/ui/Button";
import {
  DataTable,
  type DataTableColumn,
} from "../../../components/ui/DataTable";
import { Input } from "../../../components/ui/Input";
import { Pagination } from "../../../components/ui/Pagination";
import { SelectField } from "../../../components/ui/SelectField";
import { MetricHelp } from "../../../components/common/MetricHelp";
import type {
  BenchmarkRequestLogSummary,
  ReportDetail,
} from "../../../types/api";
import { RequestLogExpandedRow } from "./RequestLogExpandedRow";

type ReportStage = ReportDetail["stages"][number];
type StatusFilter = "all" | "success" | "failed";

const statusOptions: Array<{ label: string; value: StatusFilter }> = [
  { label: "全部状态", value: "all" },
  { label: "成功", value: "success" },
  { label: "失败", value: "failed" },
];

type ReportStageRequestPanelProps = {
  detail: ReportDetail;
  stage: ReportStage;
};

export function ReportStageRequestPanel({
  detail,
  stage,
}: ReportStageRequestPanelProps) {
  const taskId = detail.summary.task_id;
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [status, setStatus] = useState<StatusFilter>("all");
  const [keyword, setKeyword] = useState("");
  const [expandedRequestId, setExpandedRequestId] = useState<string | null>(null);
  const normalizedKeyword = keyword.trim();

  useEffect(() => {
    setPage(1);
    setStatus("all");
    setKeyword("");
    setExpandedRequestId(null);
  }, [taskId, stage.stage_index]);

  const logsQuery = useQuery({
    queryKey: queryKeys.benchmarkRequestLogs(
      taskId,
      page,
      pageSize,
      stage.stage_index,
      status,
      normalizedKeyword,
    ),
    queryFn: () =>
      api.listBenchmarkRequestLogsPage({
        task_id: taskId,
        page,
        page_size: pageSize,
        stage_index: stage.stage_index,
        status: status === "all" ? undefined : status,
        keyword: normalizedKeyword || undefined,
      }),
  });

  const pageData = logsQuery.data;
  const savedCountLabel = logsQuery.isLoading
    ? "读取中"
    : `${(pageData?.total ?? 0).toLocaleString("zh-CN")} 条`;

  const columns: Array<DataTableColumn<BenchmarkRequestLogSummary>> = [
    { key: "request", title: "请求", render: (row) => `#${row.request_index}` },
    { key: "sample", title: "样本", render: (row) => `#${row.sample_index}` },
    {
      key: "status",
      title: "状态",
      render: (row) => (
        <Badge tone={row.status === "success" ? "success" : "danger"}>
          {row.status === "success" ? "成功" : "失败"}
        </Badge>
      ),
    },
    {
      key: "latency",
      title: <MetricHelp helpKey="latency">耗时</MetricHelp>,
      render: (row) => `${row.latency_ms}ms`,
    },
    {
      key: "ttft",
      title: <MetricHelp helpKey="ttft">TTFT</MetricHelp>,
      render: (row) => (row.ttft_ms ? `${row.ttft_ms}ms` : "-"),
    },
    {
      key: "tokens",
      title: "Token（入/出/总）",
      render: (row) =>
        `${row.input_tokens}/${row.output_tokens}/${row.total_tokens}`,
    },
    {
      key: "prompt",
      title: "Prompt 摘要",
      render: (row) => (
        <span className="request-log-preview" title={row.prompt_preview ?? undefined}>
          {row.prompt_preview || "-"}
        </span>
      ),
    },
  ];

  const resetResults = () => {
    setPage(1);
    setExpandedRequestId(null);
  };

  return (
    <div className="stage-request-panel">
      <header className="stage-request-panel-head">
        <div>
          <strong>阶段 #{stage.stage_index} 请求证据</strong>
          <span>
            并发 {stage.concurrency.toLocaleString("zh-CN")}，实际请求{" "}
            {stage.request_count.toLocaleString("zh-CN")} 次
          </span>
        </div>
        <div className="stage-request-panel-meta">
          <Badge tone="neutral">已保存 {savedCountLabel}</Badge>
          <Badge tone={detail.request_log_meta.body_available ? "success" : "warning"}>
            {detail.request_log_meta.body_available ? "正文可用" : "仅保存索引"}
          </Badge>
        </div>
      </header>

      <div className="request-log-toolbar stage-request-toolbar">
        <Input
          aria-label={`搜索阶段 #${stage.stage_index} 请求证据`}
          placeholder="搜索 Prompt、响应摘要或错误类型"
          value={keyword}
          onChange={(event) => {
            setKeyword(event.target.value);
            resetResults();
          }}
        />
        <SelectField<StatusFilter>
          options={statusOptions}
          value={status}
          onChange={(nextStatus) => {
            setStatus(nextStatus);
            resetResults();
          }}
        />
      </div>

      <DataTable
        className="report-request-log-table stage-request-table"
        columns={columns}
        empty={
          <div className="report-empty-note stage-request-empty">
            <strong>
              {logsQuery.isLoading
                ? "正在读取阶段请求证据"
                : logsQuery.isError
                  ? "阶段请求证据读取失败"
                  : "本阶段没有请求证据"}
            </strong>
            <span>
              {logsQuery.isError
                ? logsQuery.error instanceof Error
                  ? logsQuery.error.message
                  : "无法读取持久化请求日志。"
                : detail.request_log_meta.enabled
                  ? "当前筛选条件下没有匹配记录，或本阶段没有达到保存上限内的请求明细。"
                  : "本次任务没有开启请求明细采集。"}
            </span>
            {logsQuery.isError && (
              <Button variant="ghost" onClick={() => logsQuery.refetch()}>
                重试
              </Button>
            )}
          </div>
        }
        expandable={{
          expandedRowKey: expandedRequestId,
          expandedRowRender: (row) => <RequestLogExpandedRow requestId={row.id} />,
          expandOnRowClick: true,
          onExpandedRowChange: (key) =>
            setExpandedRequestId(key == null ? null : String(key)),
        }}
        getRowKey={(row) => row.id}
        rows={pageData?.items ?? []}
      />

      {(pageData?.total ?? 0) > 0 && (
        <Pagination
          disabled={logsQuery.isFetching}
          itemLabel="请求"
          page={page}
          pageSize={pageSize}
          pageSizeOptions={[20, 50]}
          total={pageData?.total ?? 0}
          onPageChange={(nextPage) => {
            setPage(nextPage);
            setExpandedRequestId(null);
          }}
          onPageSizeChange={(nextPageSize) => {
            setPageSize(nextPageSize);
            resetResults();
          }}
        />
      )}
    </div>
  );
}
