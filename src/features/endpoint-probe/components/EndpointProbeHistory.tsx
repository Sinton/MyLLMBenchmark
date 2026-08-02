import { Badge } from "../../../components/ui/Badge";
import { Button } from "../../../components/ui/Button";
import { Card } from "../../../components/ui/Card";
import { DataTable, type DataTableColumn } from "../../../components/ui/DataTable";
import { EmptyState } from "../../../components/ui/EmptyState";
import { Input } from "../../../components/ui/Input";
import { Eye, Network, Search, Trash2 } from "../../../components/ui/icons";
import { Pagination } from "../../../components/ui/Pagination";
import { Popconfirm } from "../../../components/ui/Popconfirm";
import { SelectField } from "../../../components/ui/SelectField";
import { Tooltip } from "../../../components/ui/Tooltip";
import type { EndpointProbeBatchSummary } from "../../../types/api";
import {
  endpointProbeStatusLabel,
  endpointProbeStatusTone,
} from "../domain/endpointProbePresentation";
import type { useEndpointProbeView } from "../hooks/useEndpointProbeView";

type EndpointProbeView = ReturnType<typeof useEndpointProbeView>;

export function EndpointProbeHistory({ view }: { view: EndpointProbeView }) {
  const columns: Array<DataTableColumn<EndpointProbeBatchSummary>> = [
    {
      key: "batch",
      title: "批次",
      render: (batch) => (
        <button
          className={`endpoint-probe-history-primary ${view.selectedBatchId === batch.id ? "active" : ""}`}
          type="button"
          onClick={() => view.selectBatch(batch.id)}
        >
          <strong>{batch.name}</strong>
          <span>{batch.prompt_preview ?? "无 Prompt 摘要"}</span>
        </button>
      ),
    },
    {
      key: "status",
      title: "状态",
      width: 90,
      align: "center",
      render: (batch) => (
        <Badge tone={endpointProbeStatusTone(batch.status)}>
          {endpointProbeStatusLabel(batch.status)}
        </Badge>
      ),
    },
    {
      key: "runs",
      title: "请求",
      width: 82,
      align: "right",
      render: (batch) => batch.total_runs,
    },
    {
      key: "result",
      title: "可用 / 失败",
      width: 112,
      align: "right",
      render: (batch) => `${batch.passed_runs} / ${batch.failed_runs}`,
    },
    {
      key: "time",
      title: "创建时间",
      width: 156,
      render: (batch) => formatDate(batch.created_at),
    },
    {
      key: "actions",
      title: "操作",
      width: 82,
      align: "center",
      fixed: "right",
      render: (batch) => (
        <div className="endpoint-probe-history-actions">
          <Tooltip content="查看批次" triggerFocusable={false}>
            <Button
              aria-label={`查看 ${batch.name}`}
              icon={<Eye size={15} />}
              variant="ghost"
              onClick={() => view.selectBatch(batch.id)}
            />
          </Tooltip>
          <Popconfirm
            title="删除测活批次"
            description="将同时删除批次内所有请求记录和已保存正文。"
            confirmText="删除"
            onConfirm={() => view.deleteBatch(batch.id)}
          >
            <Tooltip content={isActive(batch.status) ? "运行中的批次不能删除" : "删除批次"} triggerFocusable={false}>
              <Button
                aria-label={`删除 ${batch.name}`}
                disabled={isActive(batch.status) || view.deletingBatchId === batch.id}
                icon={<Trash2 size={15} />}
                variant="ghost"
              />
            </Tooltip>
          </Popconfirm>
        </div>
      ),
    },
  ];

  return (
    <Card className="endpoint-probe-history-card">
      <div className="endpoint-probe-panel-head">
        <div>
          <h2>测活历史</h2>
          <p>单次与批量测活统一按批次留痕。</p>
        </div>
        {view.historyLoading && <span className="endpoint-probe-syncing">同步中</span>}
      </div>
      <div className="endpoint-probe-history-filters">
        <Input
          aria-label="搜索测活历史"
          placeholder="搜索批次或 Prompt 摘要"
          prefix={<Search size={14} />}
          value={view.historyKeyword}
          onChange={(event) => view.setHistoryKeyword(event.target.value)}
        />
        <SelectField
          ariaLabel="筛选批次状态"
          options={[
            { label: "全部状态", value: "all" },
            { label: "执行中", value: "running" },
            { label: "已完成", value: "completed" },
            { label: "已停止", value: "cancelled" },
            { label: "调度失败", value: "failed" },
          ]}
          value={view.historyStatus}
          onChange={view.setHistoryStatus}
        />
      </div>
      {view.historyError ? (
        <div className="endpoint-probe-history-error">{toErrorMessage(view.historyError)}</div>
      ) : (
        <DataTable
          className="endpoint-probe-history-table"
          columns={columns}
          empty={
            <EmptyState
              compact
              icon={<Network size={20} />}
              title="暂无测活历史"
              description="完成一次端点测活后，批次及请求指标会保留在这里。"
            />
          }
          getRowKey={(batch) => batch.id}
          rows={view.history?.items ?? []}
          scrollX={760}
        />
      )}
      <Pagination
        disabled={view.historyLoading}
        itemLabel="批次"
        page={view.historyPage}
        pageSize={view.historyPageSize}
        pageSizeOptions={[20, 50, 100]}
        total={view.history?.total ?? 0}
        onPageChange={view.setHistoryPage}
        onPageSizeChange={view.setHistoryPageSize}
      />
    </Card>
  );
}

function isActive(status: string) {
  return status === "pending" || status === "running";
}

function formatDate(value: string) {
  return new Date(value).toLocaleString("zh-CN");
}

function toErrorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}
