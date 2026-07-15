import { useState } from "react";
import { Button } from "../../../components/ui/Button";
import { Dialog } from "../../../components/ui/Dialog";
import { Input } from "../../../components/ui/Input";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { SelectField } from "../../../components/ui/SelectField";
import type { DatasetImportInput } from "../../../types/api";
import { datasetTypes } from "../constants";

const importTypeOptions = datasetTypes
  .filter((item) => item.key !== "全部")
  .map((item) => ({
    value: item.key,
    label: item.label,
    description: `${item.label} 压测样本`,
  }));

type DatasetImportDialogProps = {
  open: boolean;
  onClose: () => void;
  onSubmit: (input: DatasetImportInput) => Promise<unknown> | unknown;
  submitting?: boolean;
};

export function DatasetImportDialog({
  open,
  onClose,
  onSubmit,
  submitting = false,
}: DatasetImportDialogProps) {
  const [name, setName] = useState("");
  const [type, setType] = useState("Chat");
  const [format, setFormat] = useState("JSONL");
  const [file, setFile] = useState<File | null>(null);
  const [error, setError] = useState<string | null>(null);

  const submit = async () => {
    setError(null);
    if (!file) {
      setError("请先选择一个数据集文件。");
      return;
    }
    if (file.size > 10 * 1024 * 1024) {
      setError("数据集文件不能超过 10MB。");
      return;
    }
    const displayName = name.trim() || file.name.replace(/\.[^.]+$/, "") || `${type} 业务样本`;
    const content_base64 = await readFileAsBase64(file);
    try {
      await onSubmit({
        name: displayName,
        dataset_type: type,
        format,
        file_name: file.name,
        content_base64,
      });
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : String(submitError));
      return;
    }
    setName("");
    setType("Chat");
    setFormat("JSONL");
    setFile(null);
    onClose();
  };

  return (
    <Dialog
      open={open}
      title="导入测试数据集"
      description="支持 JSONL、CSV、TXT 和 Excel 导入；Chat Prompt 样本会保存到本地数据源用于真实压测。"
      onClose={onClose}
      footer={
        <>
          <Button disabled={submitting} onClick={onClose}>取消</Button>
          <Button loading={submitting} variant="primary" onClick={submit}>
            导入数据集
          </Button>
        </>
      }
    >
      <div className="dialog-form-grid">
        <Input
          label="数据集名称"
          placeholder="例如 Chat-客服问答-短文本"
          value={name}
          onChange={(event) => setName(event.target.value)}
        />
        <SelectField
          label="数据集类型"
          onChange={setType}
          options={importTypeOptions}
          value={type}
        />
        <SelectField
          label="文件格式"
          onChange={setFormat}
          options={[
            { value: "JSONL", label: "JSONL", description: "推荐用于结构化样本" },
            { value: "CSV", label: "CSV", description: "适合表格数据" },
            { value: "TXT", label: "TXT", description: "一行一个样本" },
            { value: "Excel", label: "Excel", description: "读取第一个工作表" },
          ]}
          value={format}
        />
        <Input
          accept=".jsonl,.csv,.txt,.xlsx"
          label="数据集文件"
          onChange={(event) => setFile(event.target.files?.[0] ?? null)}
          type="file"
        />
      </div>
      {error && <InlineAlert tone="danger" title="导入失败">{error}</InlineAlert>}
      <InlineAlert title="导入策略">
        当前会保存 Prompt 样本用于真实 Chat 压测；模型响应正文不会自动落盘。
      </InlineAlert>
    </Dialog>
  );
}

function readFileAsBase64(file: File) {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onerror = () => reject(new Error("文件读取失败"));
    reader.onload = () => {
      const result = String(reader.result ?? "");
      resolve(result.includes(",") ? result.split(",")[1] : result);
    };
    reader.readAsDataURL(file);
  });
}
