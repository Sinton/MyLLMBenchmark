import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import { Badge } from "../../../components/ui/Badge";
import { Button } from "../../../components/ui/Button";
import { Card } from "../../../components/ui/Card";
import { DataTable, type DataTableColumn } from "../../../components/ui/DataTable";
import { Dialog } from "../../../components/ui/Dialog";
import { Input } from "../../../components/ui/Input";
import { MetricHelp } from "../../../components/common/MetricHelp";
import { Pagination } from "../../../components/ui/Pagination";
import { SelectField } from "../../../components/ui/SelectField";
import type {
  BenchmarkRequestLogSummary,
  ReportDetail,
} from "../../../types/api";

type ReportRequestLogsSectionProps = {
  detail: ReportDetail;
};

export function ReportRequestLogsSection({ detail }: ReportRequestLogsSectionProps) {
  const taskId = detail.summary.task_id;
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(20);
  const [status, setStatus] = useState<"all" | "success" | "failed">("all");
  const [keyword, setKeyword] = useState("");
  const [selectedRequestId, setSelectedRequestId] = useState<string | null>(null);
  const requestLogMeta = detail.request_log_meta;

  const logsQuery = useQuery({
    queryKey: queryKeys.benchmarkRequestLogs(
      taskId,
      page,
      pageSize,
      undefined,
      status,
      keyword,
    ),
    queryFn: () =>
      api.listBenchmarkRequestLogsPage({
        task_id: taskId,
        page,
        page_size: pageSize,
        status: status === "all" ? undefined : status,
        keyword: keyword.trim() || undefined,
      }),
  });

  const detailQuery = useQuery({
    queryKey: queryKeys.benchmarkRequestLogDetail(selectedRequestId ?? ""),
    queryFn: () => api.getBenchmarkRequestLogDetail(selectedRequestId ?? ""),
    enabled: Boolean(selectedRequestId),
  });

  const pageData = logsQuery.data;
  const columns: Array<DataTableColumn<BenchmarkRequestLogSummary>> = [
    { key: "stage", title: "阶段", render: (row) => `#${row.stage_index}` },
    { key: "request", title: "请求", render: (row) => row.request_index },
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
      title: <MetricHelp helpKey="token_s">Token</MetricHelp>,
      render: (row) => `${row.input_tokens}/${row.output_tokens}/${row.total_tokens}`,
    },
    {
      key: "prompt",
      title: "Prompt 摘要",
      render: (row) => row.prompt_preview || "-",
    },
    {
      key: "action",
      title: "操作",
      render: (row) => (
        <Button variant="ghost" onClick={() => setSelectedRequestId(row.id)}>
          查看
        </Button>
      ),
    },
  ];

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
        仅在启动压测时开启“保存请求明细”后才会记录。默认不保存 Prompt 和响应正文；开启正文保存后，可在详情抽屉中查看单次请求证据。
      </p>
      <div className="request-log-toolbar">
        <Input
          aria-label="搜索请求明细"
          placeholder="搜索 Prompt、响应摘要或错误类型"
          value={keyword}
          onChange={(event) => {
            setKeyword(event.target.value);
            setPage(1);
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
            setPage(1);
          }}
        />
      </div>
      <DataTable
        className="report-request-log-table"
        columns={columns}
        empty={
          <div className="report-empty-note">
            <strong>{logsQuery.isLoading ? "正在读取请求明细" : "未找到请求明细"}</strong>
            <span>
              {requestLogMeta.enabled
                ? "当前筛选条件下没有匹配记录。"
                : "本次任务没有开启请求明细采集，或当前数据源没有可用明细。"}
            </span>
          </div>
        }
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
          onPageChange={setPage}
          onPageSizeChange={(nextPageSize) => {
            setPageSize(nextPageSize);
            setPage(1);
          }}
        />
      )}
      <Dialog
        open={Boolean(selectedRequestId)}
        title="请求详情"
        variant="drawer"
        width="620px"
        onClose={() => setSelectedRequestId(null)}
      >
        {detailQuery.data ? (
          <div className="request-log-detail">
            <div className="request-log-detail-grid">
              <span>状态</span>
              <strong>{detailQuery.data.status === "success" ? "成功" : "失败"}</strong>
              <span>耗时</span>
              <strong>{detailQuery.data.latency_ms}ms</strong>
              <span>TTFT</span>
              <strong>{detailQuery.data.ttft_ms ? `${detailQuery.data.ttft_ms}ms` : "-"}</strong>
              <span>Token</span>
              <strong>
                {detailQuery.data.input_tokens}/{detailQuery.data.output_tokens}/
                {detailQuery.data.total_tokens}
              </strong>
            </div>
            <LogBlock title="Prompt" value={detailQuery.data.prompt ?? detailQuery.data.prompt_preview} />
            <LogBlock
              title="响应"
              value={detailQuery.data.response_text ?? detailQuery.data.response_preview}
            />
            <LogBlock title="错误" value={detailQuery.data.raw_error ?? detailQuery.data.error_kind} />
            {Boolean(detailQuery.data.raw_usage) && (
              <LogBlock
                title="Usage"
                value={JSON.stringify(detailQuery.data.raw_usage, null, 2)}
              />
            )}
            {!detailQuery.data.body_available && (
              <div className="report-empty-note">
                <strong>未保存正文</strong>
                <span>本次压测未开启请求/响应正文保存，只能查看摘要和指标。</span>
              </div>
            )}
          </div>
        ) : (
          <div className="report-empty-note">
            <strong>正在读取请求详情</strong>
            <span>请稍候。</span>
          </div>
        )}
      </Dialog>
    </Card>
  );
}

function LogBlock({ title, value }: { title: string; value?: string | null }) {
  if (!value) return null;
  return (
    <section className="request-log-block">
      <h4>{title}</h4>
      <pre>{value}</pre>
    </section>
  );
}
