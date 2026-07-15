import { Button } from "../../../components/ui/Button";
import { Dialog } from "../../../components/ui/Dialog";
import type { DatasetSummary } from "../../../types/api";

type DatasetDeleteDialogProps = {
  dataset: DatasetSummary | null;
  deleting?: boolean;
  onClose: () => void;
  onConfirm: (datasetId: string) => void;
};

export function DatasetDeleteDialog({
  dataset,
  deleting = false,
  onClose,
  onConfirm,
}: DatasetDeleteDialogProps) {
  return (
    <Dialog
      open={Boolean(dataset)}
      title={dataset ? `删除「${dataset.name}」？` : "删除数据集"}
      description="删除后会清理该数据集的 Prompt 样本正文，历史任务和报告仍可读取数据集名称。"
      onClose={onClose}
      footer={
        <>
          <Button disabled={deleting} onClick={onClose}>
            取消
          </Button>
          <Button
            loading={deleting}
            variant="danger"
            onClick={() => dataset && onConfirm(dataset.id)}
          >
            删除数据集
          </Button>
        </>
      }
    >
      <div className="dataset-delete-summary">
        <span>样本数量</span>
        <strong>{dataset?.sample_count.toLocaleString("zh-CN") ?? 0}</strong>
        <span>平均 Token</span>
        <strong>{dataset?.average_tokens ?? 0}</strong>
      </div>
    </Dialog>
  );
}
