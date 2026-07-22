import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import { Badge } from "../../../components/ui/Badge";
import { Button } from "../../../components/ui/Button";
import { Card } from "../../../components/ui/Card";
import { DataTable, type DataTableColumn } from "../../../components/ui/DataTable";
import { Input } from "../../../components/ui/Input";
import { MetricHelp } from "../../../components/common/MetricHelp";
import { Pagination } from "../../../components/ui/Pagination";
import { SelectField } from "../../../components/ui/SelectField";
import type {
  BenchmarkRequestLogSummary,
  ReportDetail,
} from "../../../types/api";
import { RequestLogExpandedRow } from "./RequestLogExpandedRow";

type ReportRequestLogsSectionProps = {
  detail: ReportDetail;
  stageFilter?: number;
  onStageFilterChange: (stageIndex?: number) => void;
};

export function ReportRequestLogsSection({
  detail,
  stageFilter,
  onStageFilterChange,
}: ReportRequestLogsSectionProps) {
  const taskId = detail.summary.task_id;
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [status, setStatus] = useState<"all" | "success" | "failed">("all");
  const [keyword, setKeyword] = useState("");
  const [expandedRequestId, setExpandedRequestId] = useState<string | null>(null);
  const requestLogMeta = detail.request_log_meta;

  useEffect(() => {
    setPage(1);
    setExpandedRequestId(null);
  }, [stageFilter, taskId]);

  const logsQuery = useQuery({
    queryKey: queryKeys.benchmarkRequestLogs(
      taskId,
      page,
      pageSize,
      stageFilter,
      status,
      keyword,
    ),
    queryFn: () =>
      api.listBenchmarkRequestLogsPage({
        task_id: taskId,
        page,
        page_size: pageSize,
        stage_index: stageFilter,
        status: status === "all" ? undefined : status,
        keyword: keyword.trim() || undefined,
      }),
  });

  const pageData = logsQuery.data;
  const columns: Array<DataTableColumn<BenchmarkRequestLogSummary>> = [
    { key: "stage", title: "阶段", render: (row) => `#${row.stage_index}` },
    { key: "request", title: "请求", render: (row) => `#${row.request_index}` },
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
      render: (row) => `${row.input_tokens}/${row.output_tokens}/${row.total_tokens}`,
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
    <Card title="请求明细">
      <div className="request-log-meta-strip">
        <Badge tone={requestLogMeta.enabled ? "success" : "neutral"}>
          {requestLogMeta.enabled ? "已记录明细索引" : "未采集请求明细"}
        </Badge>
        <span>{requestLogMeta.total_records.toLocaleString("zh-CN")} 条索引</span>
        <span>
          {requestLogMeta.body_available
            ? `${requestLogMeta.body_records.toLocaleString("zh-CN")} 条正文可用`
            : "未保存正文"}
        </span>
      </div>
      <p className="report-section-copy">
        点击请求行展开输入、输出和指标。完整正文仅在启动压测前开启“保存 Prompt / 响应正文”后可用。
      </p>
      <div className="request-log-toolbar">
        <Input
          aria-label="搜索请求明细"
          placeholder="搜索 Prompt、响应摘要或错误类型"
          value={keyword}
          onChange={(event) => {
            setKeyword(event.target.value);
            resetResults();
          }}
        />
        <SelectField
          options={[
            { label: "全部阶段", value: "all" },
            ...detail.stages.map((stage) => ({
              label: `阶段 #${stage.stage_index}`,
              value: String(stage.stage_index),
            })),
          ]}
          value={stageFilter == null ? "all" : String(stageFilter)}
          onChange={(value) => {
            onStageFilterChange(value === "all" ? undefined : Number(value));
            resetResults();
          }}
        />
        <SelectField
          options={[
            { label: "全部状态", value: "all" },
            { label: "成功", value: "success" },
            { label: "失败", value: "failed" },
          ]}
          value={status}
          onChange={(nextStatus) => {
            setStatus(nextStatus);
            resetResults();
          }}
        />
      </div>
      <DataTable
        className="report-request-log-table"
        columns={columns}
        empty={
          <div className="report-empty-note">
            <strong>
              {logsQuery.isLoading
                ? "正在读取请求明细"
                : logsQuery.isError
                  ? "请求明细读取失败"
                  : "未找到请求明细"}
            </strong>
            <span>
              {logsQuery.isError
                ? logsQuery.error instanceof Error
                  ? logsQuery.error.message
                  : "无法读取持久化请求日志。"
                : requestLogMeta.enabled
                  ? "当前筛选条件下没有匹配记录。"
                  : "本次任务没有开启请求明细采集，或当前数据源没有可用明细。"}
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
    </Card>
  );
}
