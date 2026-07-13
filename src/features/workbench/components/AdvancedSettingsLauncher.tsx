import { FileText, ShieldCheck, SlidersHorizontal } from "../../../components/common/icons";
import { getModelTypeLabel } from "../../../lib/modelTaxonomy";
import type { WorkbenchForm } from "../types";
import type { AdvancedSettingsSectionKey } from "./AdvancedSettingsDrawer";

type AdvancedSettingsLauncherProps = {
  form: WorkbenchForm;
  modelType: string;
  onOpen: (section: AdvancedSettingsSectionKey) => void;
};

export function AdvancedSettingsLauncher({
  form,
  modelType,
  onOpen,
}: AdvancedSettingsLauncherProps) {
  const items = [
    {
      key: "workload" as const,
      icon: <SlidersHorizontal size={16} />,
      title: "负载参数",
      summary: workloadSummary(form, modelType),
    },
    {
      key: "protection" as const,
      icon: <ShieldCheck size={16} />,
      title: "运行保护",
      summary: `Timeout ${form.request_timeout_seconds}s / P95 ${form.sla_p95_ms}ms`,
    },
    {
      key: "evidence" as const,
      icon: <FileText size={16} />,
      title: "证据采集",
      summary: requestLogSummary(form),
    },
  ];

  return (
    <div className="advanced-settings-launcher">
      <div className="workbench-section-heading">
        <h3>高级设置</h3>
        <span>按需调整</span>
      </div>
      <div className="advanced-settings-entry-list">
        {items.map((item) => (
          <button
            className="advanced-settings-entry"
            key={item.key}
            onClick={() => onOpen(item.key)}
            type="button"
          >
            <span className="advanced-settings-entry-icon">{item.icon}</span>
            <span className="advanced-settings-entry-copy">
              <strong>{item.title}</strong>
              <span>{item.summary}</span>
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}

function workloadSummary(form: WorkbenchForm, modelType: string) {
  if (modelType === "embedding") {
    return `${getModelTypeLabel(modelType)} / Batch ${form.embedding_batch_size}`;
  }
  if (modelType === "rerank") {
    return `${getModelTypeLabel(modelType)} / Docs ${form.rerank_documents_per_query}`;
  }
  if (modelType === "multimodal") {
    return `${getModelTypeLabel(modelType)} / ${form.vision_image_count} 张图`;
  }
  return `${form.streaming ? "Streaming 开启" : "Streaming 关闭"} / Max Output ${form.max_output_tokens}`;
}

function requestLogSummary(form: WorkbenchForm) {
  if (!form.request_log_enabled) return "不保存请求明细";
  if (form.request_log_capture_body) {
    return `索引 + 正文 / 每阶段 ${form.request_log_max_records_per_stage} 条`;
  }
  return `仅保存索引 / 每阶段 ${form.request_log_max_records_per_stage} 条`;
}
