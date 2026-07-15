import { useEffect, useState } from "react";
import { Button } from "../../../components/ui/Button";
import { Dialog } from "../../../components/ui/Dialog";
import { Copy } from "../../../components/ui/icons";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { SelectField } from "../../../components/ui/SelectField";
import type { ReportExportResult, ReportSummary } from "../../../types/api";

type ReportExportDialogProps = {
  open: boolean;
  report?: ReportSummary;
  exporting?: boolean;
  onClose: () => void;
  onExport: (input: { format: string; template: string }) => Promise<ReportExportResult>;
};

export function ReportExportDialog({
  open,
  report,
  exporting = false,
  onClose,
  onExport,
}: ReportExportDialogProps) {
  const [format, setFormat] = useState("HTML");
  const [template, setTemplate] = useState("交付摘要版");
  const [result, setResult] = useState<ReportExportResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (open) {
      setResult(null);
      setError(null);
      setCopied(false);
    }
  }, [open, report?.id]);

  const submit = async () => {
    setError(null);
    setResult(null);
    try {
      const exported = await onExport({ format, template });
      setResult(exported);
      setCopied(false);
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    }
  };

  const copyPath = async () => {
    if (!result) return;
    await navigator.clipboard.writeText(result.file_path);
    setCopied(true);
  };

  return (
    <Dialog
      open={open}
      title="导出测试报告"
      description="统一配置报告格式、模板和敏感信息策略，文件会写入本机 MyLLMBenchmark 应用数据目录。"
      onClose={onClose}
      footer={
        <>
          <Button disabled={exporting} onClick={onClose}>
            关闭
          </Button>
          <Button
            disabled={!report || exporting}
            loading={exporting}
            variant="primary"
            onClick={submit}
          >
            导出文件
          </Button>
        </>
      }
    >
      <div className="dialog-form-grid">
        <SelectField
          label="导出格式"
          onChange={setFormat}
          options={[
            { value: "HTML", label: "HTML", description: "本地交付文件" },
            { value: "PDF", label: "PDF", description: "客户验收常用格式" },
            { value: "Word", label: "Word", description: "可编辑交付文档" },
            { value: "JSON", label: "JSON", description: "结构化原始报告" },
          ]}
          value={format}
        />
        <SelectField
          label="报告模板"
          onChange={setTemplate}
          options={[
            { value: "交付摘要版", label: "交付摘要版", description: "面向售前和项目验收" },
            { value: "运维容量版", label: "运维容量版", description: "包含阶段、错误和阈值" },
            { value: "详细审计版", label: "详细审计版", description: "包含更多测试条件" },
          ]}
          value={template}
        />
      </div>
      <InlineAlert title="脱敏策略">
        导出默认隐藏 Base URL 细节、API Key、完整请求正文和完整响应正文，仅保留容量结论与聚合指标。
      </InlineAlert>
      <InlineAlert title="格式说明" tone="info">
        HTML 是推荐交付格式，适合预览和浏览器打印；PDF / Word 是离线交付文件；JSON 用于审计和二次处理。
      </InlineAlert>
      {result && (
        <InlineAlert title="导出完成" tone="success">
          <div className="export-result">
            <span>{result.message}</span>
            <code>{result.file_path}</code>
            <Button icon={<Copy size={14} />} onClick={copyPath} variant="ghost">
              {copied ? "已复制" : "复制路径"}
            </Button>
          </div>
        </InlineAlert>
      )}
      {error && (
        <InlineAlert title="导出失败" tone="danger">
          {error}
        </InlineAlert>
      )}
    </Dialog>
  );
}
