import { Badge } from "../../../components/common/Badge";
import { Button } from "../../../components/common/Button";
import { DataTable, type DataTableColumn } from "../../../components/common/DataTable";
import { EmptyState } from "../../../components/common/EmptyState";
import { Eye, FileCheck, Pencil, Trash2 } from "../../../components/common/icons";
import { getModelTypeLabel } from "../../../lib/modelTaxonomy";
import type { DatasetSummary } from "../../../types/api";

type DatasetTableProps = {
  datasets: DatasetSummary[];
  onDelete: (dataset: DatasetSummary) => void;
  onEdit: (dataset: DatasetSummary) => void;
  onView: (dataset: DatasetSummary) => void;
};

export function DatasetTable({
  datasets,
  onDelete,
  onEdit,
  onView,
}: DatasetTableProps) {
  const columns: Array<DataTableColumn<DatasetSummary>> = [
    {
      key: "name",
      title: "名称",
      render: (dataset) => (
        <button
          className="dataset-name-button"
          type="button"
          onClick={() => onView(dataset)}
        >
          <FileCheck size={16} />
          <span>{dataset.name}</span>
        </button>
      ),
    },
    {
      key: "type",
      title: "类型",
      render: (dataset) => getModelTypeLabel(dataset.dataset_type),
    },
    {
      key: "sample_count",
      title: "样本数",
      render: (dataset) => dataset.sample_count.toLocaleString("zh-CN"),
      align: "right",
    },
    {
      key: "average_tokens",
      title: "平均 Token",
      render: (dataset) => dataset.average_tokens,
      align: "right",
    },
    {
      key: "updated_at",
      title: "更新时间",
      render: (dataset) => new Date(dataset.updated_at).toLocaleDateString("zh-CN"),
    },
    {
      key: "status",
      title: "状态",
      render: () => <Badge tone="success">可审计</Badge>,
    },
    {
      key: "actions",
      title: "操作",
      render: (dataset) => (
        <div className="dataset-row-actions">
          <Button
            aria-label={`查看 ${dataset.name}`}
            icon={<Eye size={15} />}
            onClick={() => onView(dataset)}
            title="查看样本"
            variant="ghost"
          />
          <Button
            aria-label={`编辑 ${dataset.name}`}
            icon={<Pencil size={15} />}
            onClick={() => onEdit(dataset)}
            title="编辑数据集"
            variant="ghost"
          />
          <Button
            aria-label={`删除 ${dataset.name}`}
            icon={<Trash2 size={15} />}
            onClick={() => onDelete(dataset)}
            title="删除数据集"
            variant="ghost"
          />
        </div>
      ),
    },
  ];

  return (
    <DataTable
      columns={columns}
      empty={
        <EmptyState
          compact
          description="当前类型下没有可用样本，可以通过导入向导补充。"
          icon={<FileCheck size={22} />}
          title="暂无数据集"
        />
      }
      getRowKey={(dataset) => dataset.id}
      rows={datasets}
    />
  );
}
