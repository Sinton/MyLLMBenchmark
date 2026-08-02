import { Badge } from "../../../components/ui/Badge";
import { Button } from "../../../components/ui/Button";
import { DataTable, type DataTableColumn } from "../../../components/ui/DataTable";
import { EmptyState } from "../../../components/ui/EmptyState";
import { Eye, Network, Search, Trash2 } from "../../../components/ui/icons";
import { Input } from "../../../components/ui/Input";
import { Pagination } from "../../../components/ui/Pagination";
import { Popconfirm } from "../../../components/ui/Popconfirm";
import { SelectField } from "../../../components/ui/SelectField";
import { Tooltip } from "../../../components/ui/Tooltip";
import type { SiteProbeHistoryPage, SiteProbeRunSummary } from "../../../types/api";

type SiteProbeHistoryProps = {
  history?: SiteProbeHistoryPage;
  loading: boolean;
  error: unknown;
  keyword: string;
  page: number;
  pageSize: number;
  selectedRunId: string | null;
  statusFilter: string;
  deletingRunId: string | null;
  onDelete: (runId: string) => void;
  onKeywordChange: (value: string) => void;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
  onSelect: (runId: string) => void;
  onStatusFilterChange: (status: string) => void;
};

export function SiteProbeHistory({
  history,
  loading,
  error,
  keyword,
  page,
  pageSize,
  selectedRunId,
  statusFilter,
  deletingRunId,
  onDelete,
  onKeywordChange,
  onPageChange,
  onPageSizeChange,
  onSelect,
  onStatusFilterChange,
}: SiteProbeHistoryProps) {
  const columns: Array<DataTableColumn<SiteProbeRunSummary>> = [
    {
      key: "name",
      title: "站点",
      render: (row) => (
        <button
          className={`site-probe-history-primary ${
            selectedRunId === row.id ? "active" : ""
          }`}
          type="button"
          onClick={() => onSelect(row.id)}
        >
          <strong>{row.name}</strong>
          <span>{row.base_url}</span>
        </button>
      ),
    },
    {
      key: "status",
      title: "状态",
      width: 74,
      align: "center",
      render: (row) => (
        <Badge tone={row.status === "passed" ? "success" : "danger"}>
          {row.status === "passed" ? "可用" : "失败"}
        </Badge>
      ),
    },
    {
      key: "latency",
      title: "耗时",
      width: 74,
      align: "right",
      render: (row) => `${row.latency_ms}ms`,
    },
    {
      key: "actions",
      title: "操作",
      width: 82,
      align: "center",
      fixed: "right",
      render: (row) => (
        <div className="site-probe-history-actions">
          <Tooltip content="查看详情" triggerFocusable={false}>
            <Button
              aria-label={`查看 ${row.name} 测活记录`}
              icon={<Eye size={15} />}
              variant="ghost"
              onClick={() => onSelect(row.id)}
            />
          </Tooltip>
          <Popconfirm
            title="删除测活记录"
            description="只删除本条历史记录和对应正文文件。"
            confirmText="删除"
            onConfirm={() => onDelete(row.id)}
          >
            <Tooltip content="删除记录" triggerFocusable={false}>
              <Button
                aria-label={`删除 ${row.name} 测活记录`}
                disabled={deletingRunId === row.id}
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
    <aside className="site-probe-history-card">
      <div className="site-probe-history-head">
        <div>
          <h2>测活历史</h2>
        </div>
        {loading && <span className="site-probe-history-sync">同步中</span>}
      </div>

      <div className="site-probe-history-filters">
        <Input
          aria-label="搜索测活历史"
          placeholder="搜索站点 / 模型 / 摘要"
          prefix={<Search size={14} />}
          value={keyword}
          onChange={(event) => onKeywordChange(event.target.value)}
        />
        <SelectField
          value={statusFilter}
          onChange={onStatusFilterChange}
          options={[
            { label: "全部状态", value: "all" },
            { label: "可用", value: "passed" },
            { label: "失败", value: "failed" },
          ]}
        />
      </div>

      {error ? (
        <div className="site-probe-history-error">
          {error instanceof Error ? error.message : String(error)}
        </div>
      ) : (
        <DataTable
          className="site-probe-history-table"
          columns={columns}
          empty={
            <EmptyState
              icon={<Network size={20} />}
              title="暂无测活历史"
              description="完成一次测活后，历史记录会保留站点、模型、耗时、Token 和可选正文状态。"
            />
          }
          getRowKey={(row) => row.id}
          rows={history?.items ?? []}
          scrollX={560}
        />
      )}

      <Pagination
        disabled={loading}
        itemLabel="记录"
        page={page}
        pageSize={pageSize}
        pageSizeOptions={[20, 50, 100]}
        total={history?.total ?? 0}
        onPageChange={onPageChange}
        onPageSizeChange={onPageSizeChange}
      />
    </aside>
  );
}
