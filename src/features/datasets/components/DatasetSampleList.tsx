import { useEffect, useState } from "react";
import { Button } from "../../../components/common/Button";
import { DataTable, type DataTableColumn } from "../../../components/common/DataTable";
import { EmptyState } from "../../../components/common/EmptyState";
import { FileText, Pencil, Plus, Search, Trash2 } from "../../../components/common/icons";
import { InlineAlert } from "../../../components/common/InlineAlert";
import { Input } from "../../../components/common/Input";
import { Pagination } from "../../../components/common/Pagination";
import { Popconfirm } from "../../../components/common/Popconfirm";
import { Textarea } from "../../../components/common/Textarea";
import type {
  DatasetSampleCreateInput,
  DatasetSamplePreview,
  DatasetSampleUpdateInput,
} from "../../../types/api";

type DatasetSampleListProps = {
  creating?: boolean;
  datasetId: string;
  deleting?: boolean;
  fetching?: boolean;
  loading?: boolean;
  onCreate: (input: DatasetSampleCreateInput) => Promise<unknown>;
  onDelete: (sampleId: string) => void;
  onBatchDelete: (sampleIds: string[]) => void;
  onPageChange: (page: number) => void;
  onPageSizeChange: (value: number) => void;
  onSearchChange: (value: string) => void;
  onUpdate: (input: DatasetSampleUpdateInput) => Promise<unknown>;
  page: number;
  pageSize: number;
  samples: DatasetSamplePreview[];
  search: string;
  total: number;
  updating?: boolean;
  batchDeleting?: boolean;
};

export function DatasetSampleList({
  creating = false,
  datasetId,
  deleting = false,
  fetching = false,
  loading = false,
  onCreate,
  onDelete,
  onBatchDelete,
  onPageChange,
  onPageSizeChange,
  onSearchChange,
  onUpdate,
  page,
  pageSize,
  samples,
  search,
  total,
  updating = false,
  batchDeleting = false,
}: DatasetSampleListProps) {
  const [newPrompt, setNewPrompt] = useState("");
  const [editingSampleId, setEditingSampleId] = useState<string | null>(null);
  const [editingPrompt, setEditingPrompt] = useState("");
  const [formError, setFormError] = useState<string | null>(null);
  const [selectedIds, setSelectedIds] = useState<string[]>([]);

  useEffect(() => {
    setSelectedIds([]);
  }, [page, pageSize, search, samples]);

  const allCurrentSelected =
    samples.length > 0 && samples.every((sample) => selectedIds.includes(sample.id));
  const toggleAll = () => {
    setSelectedIds(allCurrentSelected ? [] : samples.map((sample) => sample.id));
  };
  const toggleOne = (sampleId: string) => {
    setSelectedIds((current) =>
      current.includes(sampleId)
        ? current.filter((item) => item !== sampleId)
        : [...current, sampleId],
    );
  };
  const submitBatchDelete = () => {
    if (selectedIds.length === 0) {
      setFormError("请先勾选当前页需要删除的样本。");
      return;
    }
    setFormError(null);
    onBatchDelete(selectedIds);
    setSelectedIds([]);
  };

  const submitCreate = async () => {
    const prompt = newPrompt.trim();
    if (!prompt) {
      setFormError("Prompt 样本不能为空。");
      return;
    }
    setFormError(null);
    await onCreate({ dataset_id: datasetId, prompt });
    setNewPrompt("");
  };

  const startEdit = (sample: DatasetSamplePreview) => {
    setEditingSampleId(sample.id);
    setEditingPrompt(sample.prompt);
    setFormError(null);
  };

  const submitEdit = async (sampleId: string) => {
    const prompt = editingPrompt.trim();
    if (!prompt) {
      setFormError("Prompt 样本不能为空。");
      return;
    }
    setFormError(null);
    await onUpdate({ sample_id: sampleId, prompt });
    setEditingSampleId(null);
    setEditingPrompt("");
  };

  const columns: Array<DataTableColumn<DatasetSamplePreview>> = [
    {
      key: "select",
      title: "选择",
      render: (sample) => (
        <input
          aria-label={`选择第 ${sample.sample_index + 1} 条样本`}
          checked={selectedIds.includes(sample.id)}
          type="checkbox"
          onChange={() => toggleOne(sample.id)}
        />
      ),
    },
    {
      key: "index",
      title: "序号",
      render: (sample) => (
        <strong className="dataset-sample-index">#{sample.sample_index + 1}</strong>
      ),
    },
    {
      key: "prompt",
      title: "Prompt 内容",
      render: (sample) => {
        const editing = editingSampleId === sample.id;
        if (editing) {
          return (
            <div className="dataset-sample-editor">
              <Textarea
                aria-label="编辑 Prompt 样本"
                value={editingPrompt}
                onChange={(event) => setEditingPrompt(event.target.value)}
              />
              <div className="dataset-sample-actions">
                <Button
                  disabled={updating}
                  variant="ghost"
                  onClick={() => {
                    setEditingSampleId(null);
                    setEditingPrompt("");
                  }}
                >
                  取消
                </Button>
                <Button
                  loading={updating}
                  variant="primary"
                  onClick={() => submitEdit(sample.id)}
                >
                  保存样本
                </Button>
              </div>
            </div>
          );
        }

        return <p className="dataset-sample-prompt">{sample.prompt}</p>;
      },
    },
    {
      key: "tokens",
      title: "估算 Token",
      align: "right",
      render: (sample) => sample.estimated_tokens.toLocaleString("zh-CN"),
    },
    {
      key: "actions",
      title: "操作",
      render: (sample) => (
        <div className="dataset-sample-row-actions">
          <Button
            icon={<Pencil size={14} />}
            variant="ghost"
            onClick={() => startEdit(sample)}
          >
            编辑
          </Button>
          <Popconfirm
            title="删除这条样本？"
            description="删除后会重新计算数据集统计。"
            confirmText="删除"
            onConfirm={() => onDelete(sample.id)}
          >
            <Button
              disabled={deleting}
              icon={<Trash2 size={14} />}
              variant="ghost"
            >
              删除
            </Button>
          </Popconfirm>
        </div>
      ),
    },
  ];

  return (
    <div className="dataset-samples">
      <div className="dataset-samples-toolbar">
        <Input
          placeholder="搜索 Prompt 内容"
          prefix={<Search size={15} />}
          value={search}
          onChange={(event) => onSearchChange(event.target.value)}
        />
        <div className="dataset-sample-count">
          <span>
            {loading
              ? "正在读取样本..."
              : `${total.toLocaleString("zh-CN")} 条匹配样本`}
          </span>
          {fetching && !loading && <span>正在刷新当前页</span>}
        </div>
        <Button disabled={samples.length === 0} variant="ghost" onClick={toggleAll}>
          {allCurrentSelected ? "取消全选" : "全选本页"}
        </Button>
        <Button
          disabled={selectedIds.length === 0 || batchDeleting}
          icon={<Trash2 size={14} />}
          loading={batchDeleting}
          variant="danger"
          onClick={submitBatchDelete}
        >
          删除选中 {selectedIds.length > 0 ? selectedIds.length : ""}
        </Button>
      </div>

      {formError && (
        <InlineAlert tone="danger" title="样本内容无效">
          {formError}
        </InlineAlert>
      )}

      <DataTable
        className="dataset-sample-table"
        columns={columns}
        empty={
          <EmptyState
            compact
            description={
              search
                ? "没有匹配当前搜索条件的 Prompt。"
                : "当前数据集还没有可用 Prompt 样本。"
            }
            icon={<FileText size={22} />}
            title={loading ? "正在读取样本" : "暂无样本"}
          />
        }
        getRowKey={(sample) => sample.id}
        rows={loading ? [] : samples}
      />

      <Pagination
        disabled={loading}
        page={page}
        pageSize={pageSize}
        total={total}
        onPageChange={onPageChange}
        onPageSizeChange={onPageSizeChange}
      />

      <div className="dataset-sample-create">
        <Textarea
          label="新增 Prompt 样本"
          placeholder="输入一条真实业务 Prompt，用于后续压测抽样。"
          value={newPrompt}
          onChange={(event) => setNewPrompt(event.target.value)}
        />
        <Button
          icon={<Plus size={15} />}
          loading={creating}
          variant="primary"
          onClick={submitCreate}
        >
          新增样本
        </Button>
      </div>
    </div>
  );
}
