import type { DatasetSummary, ModelSummary, ProviderSummary } from "../../../types/api";
import type { WorkbenchForm } from "../types";

type RequestPreviewPanelProps = {
  form: WorkbenchForm;
  provider?: ProviderSummary;
  model?: ModelSummary;
  dataset?: DatasetSummary;
  modelType: string;
};

export function RequestPreviewPanel({
  form,
  provider,
  model,
  dataset,
  modelType,
}: RequestPreviewPanelProps) {
  const preview = {
    runtime: "Tauri backend benchmark task",
    provider: {
      name: provider?.name ?? "未选择",
      base_url: provider?.base_url_masked ?? "未选择",
      api_key: "使用服务商配置中的本地 API Key",
    },
    benchmark_input: {
      model: model?.name ?? form.model_id,
      dataset: dataset?.name ?? form.dataset_id,
      dataset_type: dataset?.dataset_type ?? "-",
      workload: modelType,
      mode: form.mode,
      concurrency:
        form.mode === "阶梯加压"
          ? `${form.start_concurrency} -> ${form.end_concurrency}`
          : form.concurrency,
      stage_sample_rounds: form.stage_sample_rounds,
      warmup_rounds: form.warmup_rounds,
      request_timeout_seconds: form.request_timeout_seconds,
      sla: {
        p95_ms: form.sla_p95_ms,
        success_rate: form.min_success_rate,
        stop_policy: form.sla_stop_policy,
      },
    },
  };

  return (
    <div className="request-preview-panel">
      <div className="request-preview-header">
        <span>调试 Payload</span>
        <strong>排障参考</strong>
      </div>
      <pre className="request-preview-body">{JSON.stringify(preview, null, 2)}</pre>
    </div>
  );
}
