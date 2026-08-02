import { useEffect, useState } from "react";
import { Badge } from "../../../components/ui/Badge";
import { Button } from "../../../components/ui/Button";
import { DataTable, type DataTableColumn } from "../../../components/ui/DataTable";
import { Dialog } from "../../../components/ui/Dialog";
import { FilePicker } from "../../../components/ui/FilePicker";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { Textarea } from "../../../components/ui/Textarea";
import type {
  ProviderImportItem,
  ProviderImportItemResult,
  ProviderImportResult,
} from "../../../types/api";
import { parseProviderImportJson } from "../domain/endpointProbePresentation";

type ProviderImportDialogProps = {
  open: boolean;
  pending: boolean;
  result?: ProviderImportResult;
  onClose: () => void;
  onImport: (items: ProviderImportItem[]) => Promise<ProviderImportResult>;
};

const exampleJson = `[
  {
    "name": "华东中转站",
    "base_url": "https://api.example.com/v1",
    "api_key": "sk-...",
    "interface_type": "OpenAI",
    "models": ["gpt-4.1-mini"]
  }
]`;

export function ProviderImportDialog({
  open,
  pending,
  result,
  onClose,
  onImport,
}: ProviderImportDialogProps) {
  const [source, setSource] = useState(exampleJson);
  const [file, setFile] = useState<File | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [submitted, setSubmitted] = useState(false);

  useEffect(() => {
    if (!open) {
      setFile(null);
      setError(null);
      setSubmitted(false);
      setSource(exampleJson);
    }
  }, [open]);

  const columns: Array<DataTableColumn<ProviderImportItemResult>> = [
    { key: "index", title: "序号", width: 64, align: "center", render: (item) => item.index },
    {
      key: "status",
      title: "结果",
      width: 88,
      align: "center",
      render: (item) => (
        <Badge tone={item.status === "created" ? "success" : item.status === "failed" ? "danger" : "warning"}>
          {item.status === "created" ? "已创建" : item.status === "failed" ? "失败" : "已跳过"}
        </Badge>
      ),
    },
    { key: "message", title: "说明", render: (item) => item.message },
  ];

  const submit = async () => {
    try {
      setError(null);
      const items = parseProviderImportJson(source);
      await onImport(items);
      setSubmitted(true);
    } catch (nextError) {
      setError(nextError instanceof Error ? nextError.message : String(nextError));
    }
  };

  return (
    <Dialog
      className="endpoint-probe-import-dialog"
      open={open}
      title="批量导入服务商"
      description="支持 JSON 文件或粘贴导入；合法记录会部分成功写入，重复连接自动跳过。"
      width="680px"
      onClose={onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>关闭</Button>
          <Button loading={pending} variant="primary" onClick={submit}>校验并导入</Button>
        </>
      }
    >
      <div className="endpoint-probe-dialog-form">
        <FilePicker
          accept="application/json,.json"
          file={file}
          hint="文件内容只在本地读取，并通过 Tauri command 交给 Rust 后端。"
          label="JSON 文件"
          onFileChange={(nextFile) => {
            setFile(nextFile);
            if (!nextFile) return;
            void nextFile.text().then(setSource).catch((readError) => {
              setError(readError instanceof Error ? readError.message : String(readError));
            });
          }}
        />
        <Textarea
          label="JSON 内容"
          rows={12}
          spellCheck={false}
          value={source}
          onChange={(event) => setSource(event.target.value)}
        />
        <InlineAlert tone="info">
          导入结果不会回显 API Key。新服务商初始状态为“未检查”，测活通过后才会更新在线状态。
        </InlineAlert>
        {error && <InlineAlert tone="danger" title="JSON 无法导入">{error}</InlineAlert>}
        {result && submitted && (
          <section className="endpoint-probe-import-result">
            <div>
              <strong>导入结果</strong>
              <span>创建 {result.created} · 跳过 {result.skipped} · 失败 {result.failed}</span>
            </div>
            <DataTable
              columns={columns}
              getRowKey={(item) => `${item.index}-${item.status}`}
              rows={result.items}
              scrollX={520}
            />
          </section>
        )}
      </div>
    </Dialog>
  );
}
