import { useEffect, useState } from "react";
import { Badge } from "../../../components/ui/Badge";
import { Button } from "../../../components/ui/Button";
import { Dialog } from "../../../components/ui/Dialog";
import { Download, FileCheck, Pencil, ShieldCheck, Upload } from "../../../components/ui/icons";
import { Input } from "../../../components/ui/Input";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { SelectField } from "../../../components/ui/SelectField";
import { getModelTypeLabel } from "../../../lib/modelTaxonomy";
import type {
  DatasetSampleCreateInput,
  DatasetSampleBatchDeleteInput,
  DatasetSamplePage,
  DatasetSampleUpdateInput,
  DatasetSummary,
  DatasetUpdateInput,
  DatasetAppendInput,
  DatasetExportInput,
  DatasetValidationResult,
} from "../../../types/api";
import { datasetTypes } from "../constants";
import { DatasetSampleList } from "./DatasetSampleList";

const datasetTypeOptions = datasetTypes
  .filter((item) => item.key !== "全部")
  .map((item) => ({
    value: item.key,
    label: item.label,
    description: `${item.label} 压测样本`,
  }));

type DatasetDetailDrawerProps = {
  appendSamples: (input: DatasetAppendInput) => Promise<unknown>;
  appendSamplesPending?: boolean;
  batchDeleteSamples: (input: DatasetSampleBatchDeleteInput) => void;
  batchDeletePending?: boolean;
  createSample: (input: DatasetSampleCreateInput) => Promise<unknown>;
  createSamplePending?: boolean;
  dataset: DatasetSummary | null;
  deleteSample: (sampleId: string) => void;
  deleteSamplePending?: boolean;
  editing: boolean;
  onClose: () => void;
  onEditingChange: (editing: boolean) => void;
  onPageChange: (page: number) => void;
  onPageSizeChange: (value: number) => void;
  onSearchChange: (value: string) => void;
  onUpdateDataset: (input: DatasetUpdateInput) => Promise<unknown>;
  onExportDataset: (input: DatasetExportInput) => void;
  onValidateDataset: (datasetId: string) => void;
  sampleKeyword: string;
  samplePage: DatasetSamplePage;
  samplesFetching?: boolean;
  samplesLoading?: boolean;
  updateDatasetPending?: boolean;
  updateSample: (input: DatasetSampleUpdateInput) => Promise<unknown>;
  updateSamplePending?: boolean;
  exportDatasetPending?: boolean;
  validateDatasetPending?: boolean;
  validationResult: DatasetValidationResult | null;
};

export function DatasetDetailDrawer({
  appendSamples,
  appendSamplesPending = false,
  batchDeleteSamples,
  batchDeletePending = false,
  createSample,
  createSamplePending = false,
  dataset,
  deleteSample,
  deleteSamplePending = false,
  editing,
  onClose,
  onEditingChange,
  onPageChange,
  onPageSizeChange,
  onSearchChange,
  onUpdateDataset,
  onExportDataset,
  onValidateDataset,
  sampleKeyword,
  samplePage,
  samplesFetching = false,
  samplesLoading = false,
  updateDatasetPending = false,
  updateSample,
  updateSamplePending = false,
  exportDatasetPending = false,
  validateDatasetPending = false,
  validationResult,
}: DatasetDetailDrawerProps) {
  const [name, setName] = useState("");
  const [datasetType, setDatasetType] = useState("Chat");
  const [error, setError] = useState<string | null>(null);
  const [appendFormat, setAppendFormat] = useState("JSONL");
  const [appendFile, setAppendFile] = useState<File | null>(null);

  useEffect(() => {
    if (!dataset) return;
    setName(dataset.name);
    setDatasetType(dataset.dataset_type);
    setError(null);
    setAppendFormat("JSONL");
    setAppendFile(null);
  }, [dataset]);

  if (!dataset) return null;

  const submitDataset = async () => {
    try {
      const nextName = name.trim();
      if (!nextName) {
        setError("数据集名称不能为空。");
        return;
      }
      setError(null);
      await onUpdateDataset({
        id: dataset.id,
        name: nextName,
        dataset_type: datasetType,
      });
      onEditingChange(false);
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : String(submitError));
    }
  };

  const submitAppend = async () => {
    try {
      setError(null);
      if (!appendFile) {
        setError("请先选择要追加的样本文件。");
        return;
      }
      if (appendFile.size > 10 * 1024 * 1024) {
        setError("追加文件不能超过 10MB。");
        return;
      }
      await appendSamples({
        dataset_id: dataset.id,
        format: appendFormat,
        file_name: appendFile.name,
        content_base64: await readFileAsBase64(appendFile),
      });
      setAppendFile(null);
    } catch (submitError) {
      setError(submitError instanceof Error ? submitError.message : String(submitError));
    }
  };

  return (
    <Dialog
      open={Boolean(dataset)}
      title={dataset.name}
      description="查看和维护压测 Prompt 样本，保证容量结论可审计。"
      variant="drawer"
      width="820px"
      onClose={onClose}
      footer={<Button onClick={onClose}>关闭</Button>}
    >
      <div className="dataset-detail-drawer">
        <section className="dataset-detail-hero">
          <div className="dataset-detail-icon">
            <FileCheck size={22} />
          </div>
          <div>
            <div className="dataset-detail-title-row">
              <h3>{dataset.name}</h3>
              <Badge tone="success">可审计</Badge>
            </div>
            <p>
              {getModelTypeLabel(dataset.dataset_type)} /{" "}
              {dataset.sample_count.toLocaleString("zh-CN")} 条样本
            </p>
          </div>
          <Button
            icon={<Pencil size={15} />}
            variant="ghost"
            onClick={() => onEditingChange(!editing)}
          >
            {editing ? "收起编辑" : "编辑信息"}
          </Button>
        </section>

        <div className="dataset-stat-grid">
          <div>
            <span>样本数</span>
            <strong>{dataset.sample_count.toLocaleString("zh-CN")}</strong>
          </div>
          <div>
            <span>平均 Token</span>
            <strong>{dataset.average_tokens}</strong>
          </div>
          <div>
            <span>更新时间</span>
            <strong>{new Date(dataset.updated_at).toLocaleString("zh-CN")}</strong>
          </div>
        </div>

        <section className="dataset-tools-panel">
          <div className="dataset-tool-row">
            <SelectField
              label="追加格式"
              onChange={setAppendFormat}
              options={[
                { value: "JSONL", label: "JSONL", description: "结构化样本" },
                { value: "CSV", label: "CSV", description: "表格样本" },
                { value: "TXT", label: "TXT", description: "一行一个样本" },
                { value: "Excel", label: "Excel", description: "第一个工作表" },
              ]}
              value={appendFormat}
            />
            <Input
              accept=".jsonl,.csv,.txt,.xlsx"
              label="追加样本文件"
              type="file"
              onChange={(event) => setAppendFile(event.target.files?.[0] ?? null)}
            />
            <Button
              icon={<Upload size={15} />}
              loading={appendSamplesPending}
              variant="primary"
              onClick={submitAppend}
            >
              追加样本
            </Button>
          </div>
          <div className="dataset-tool-actions">
            {(["jsonl", "csv", "txt"] as const).map((format) => (
              <Button
                key={format}
                disabled={exportDatasetPending}
                icon={<Download size={15} />}
                variant="ghost"
                onClick={() => onExportDataset({ dataset_id: dataset.id, format })}
              >
                导出 {format.toUpperCase()}
              </Button>
            ))}
            <Button
              icon={<ShieldCheck size={15} />}
              loading={validateDatasetPending}
              variant="ghost"
              onClick={() => onValidateDataset(dataset.id)}
            >
              质量检查
            </Button>
          </div>
          {validationResult && validationResult.dataset_id === dataset.id && (
            <InlineAlert
              title={`质量检查：${validationResult.status}`}
              tone={validationResult.status === "passed" ? "success" : "warning"}
            >
              {validationResult.issues.length === 0
                ? validationResult.recommendations[0]
                : validationResult.issues
                    .map((issue) => `${issue.label} ${issue.count} 项`)
                    .join("；")}
            </InlineAlert>
          )}
        </section>

        {editing && (
          <section className="dataset-edit-panel">
            <Input
              error={error ?? undefined}
              label="数据集名称"
              value={name}
              onChange={(event) => setName(event.target.value)}
            />
            <SelectField
              label="数据集类型"
              options={datasetTypeOptions}
              value={datasetType}
              onChange={setDatasetType}
            />
            <div className="dataset-edit-actions">
              <Button
                disabled={updateDatasetPending}
                variant="ghost"
                onClick={() => {
                  setName(dataset.name);
                  setDatasetType(dataset.dataset_type);
                  setError(null);
                  onEditingChange(false);
                }}
              >
                取消
              </Button>
              <Button
                loading={updateDatasetPending}
                variant="primary"
                onClick={submitDataset}
              >
                保存信息
              </Button>
            </div>
          </section>
        )}

        <DatasetSampleList
          creating={createSamplePending}
          datasetId={dataset.id}
          deleting={deleteSamplePending}
          batchDeleting={batchDeletePending}
          fetching={samplesFetching}
          loading={samplesLoading}
          onCreate={createSample}
          onBatchDelete={(sampleIds) =>
            batchDeleteSamples({ dataset_id: dataset.id, sample_ids: sampleIds })
          }
          onDelete={deleteSample}
          onPageChange={onPageChange}
          onPageSizeChange={onPageSizeChange}
          onSearchChange={onSearchChange}
          onUpdate={updateSample}
          page={samplePage.page}
          pageSize={samplePage.page_size}
          samples={samplePage.items}
          search={sampleKeyword}
          total={samplePage.total}
          updating={updateSamplePending}
        />
      </div>
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
