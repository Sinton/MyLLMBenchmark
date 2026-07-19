import { useState, type Dispatch, type FormEvent, type SetStateAction } from "react";
import { Card } from "../../../components/ui/Card";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import type {
  DatasetSummary,
  ModelSummary,
  ProviderDiagnosticsResult,
  ProviderSummary,
} from "../../../types/api";
import type { StartNotice, WorkbenchForm } from "../types";
import {
  AdvancedSettingsDrawer,
  type AdvancedSettingsSectionKey,
} from "./AdvancedSettingsDrawer";
import { AdvancedSettingsLauncher } from "./AdvancedSettingsLauncher";
import { BenchmarkModeSection } from "./BenchmarkModeSection";
import { FixedLoadConfigSection } from "./FixedLoadConfigSection";
import { StaircaseConfigSection } from "./StaircaseConfigSection";
import { StartActionFooter } from "./StartActionFooter";
import { StartReadiness } from "./StartReadiness";
import { TargetSelectorSection } from "./TargetSelectorSection";

type BenchmarkConfigPanelProps = {
  form: WorkbenchForm;
  setForm: Dispatch<SetStateAction<WorkbenchForm>>;
  providers: ProviderSummary[];
  providerModels: ModelSummary[];
  datasets: DatasetSummary[];
  selectedProvider?: ProviderSummary;
  providerDiagnostics: ProviderDiagnosticsResult | null;
  providerDiagnosticsFetching: boolean;
  selectedModel?: ModelSummary;
  selectedModelType: string;
  isStaircase: boolean;
  startNotice: StartNotice | null;
  startBlockReason: string | null;
  canSubmitStart: boolean;
  startPending: boolean;
  onSubmit: (event: FormEvent) => void;
};

export function BenchmarkConfigPanel({
  form,
  setForm,
  providers,
  providerModels,
  datasets,
  selectedProvider,
  providerDiagnostics,
  providerDiagnosticsFetching,
  selectedModel,
  selectedModelType,
  isStaircase,
  startNotice,
  startBlockReason,
  canSubmitStart,
  startPending,
  onSubmit,
}: BenchmarkConfigPanelProps) {
  const [advancedOpen, setAdvancedOpen] = useState(false);
  const [advancedSection, setAdvancedSection] =
    useState<AdvancedSettingsSectionKey>("workload");

  return (
    <Card title="任务配置" eyebrow="核心配置" className="workbench-config">
      <form className="workbench-config-form" onSubmit={onSubmit}>
        <div className="workbench-config-scroll">
          <div className="workbench-form-section">
            <div className="workbench-section-heading">
              <h3>测试对象</h3>
              <span>{selectedProvider?.name ?? "先选择服务商"}</span>
            </div>
            <TargetSelectorSection
              datasets={datasets}
              form={form}
              providerModels={providerModels}
              providers={providers}
              setForm={setForm}
            />
            {selectedProvider && isUnsupportedRealInterface(selectedProvider.interface_type) && (
              <InlineAlert title="真实压测兼容性提示" tone="warning">
                {selectedProvider.interface_type} 当前版本未启用真实压测引擎；如果设置中选择真实引擎，启动时不会按 OpenAI 协议误发请求，请等待后续协议适配。
              </InlineAlert>
            )}
            {selectedProvider && !providerDiagnostics && !providerDiagnosticsFetching && (
              <InlineAlert title="尚未执行兼容性诊断" tone="info">
                可先在服务商页运行兼容性诊断。未诊断不会阻止启动，但报告会缺少诊断证据。
              </InlineAlert>
            )}
            {providerDiagnostics && providerDiagnostics.status !== "passed" && (
              <InlineAlert title="最近诊断存在风险" tone="warning">
                最近诊断状态为 {providerDiagnostics.status}。后端会在启动前继续做硬校验，明显不兼容的任务会被阻止。
              </InlineAlert>
            )}
          </div>

          <div className="workbench-form-section">
            <div className="workbench-section-heading">
              <h3>压测策略</h3>
              <span>{selectedModel?.name ?? "等待模型"}</span>
            </div>
            <BenchmarkModeSection form={form} setForm={setForm} />
            {isStaircase ? (
              <StaircaseConfigSection form={form} setForm={setForm} />
            ) : (
              <FixedLoadConfigSection form={form} setForm={setForm} />
            )}
          </div>
        </div>

        <div className="workbench-start-dock">
          <AdvancedSettingsLauncher
            summary={advancedSettingsSummary(form, selectedModelType)}
            onOpen={() => setAdvancedOpen(true)}
          />
          <StartReadiness notice={startNotice} reason={startBlockReason} />
          <StartActionFooter
            canSubmitStart={canSubmitStart}
            startBlockReason={startBlockReason}
            startPending={startPending}
          />
        </div>
      </form>

      <AdvancedSettingsDrawer
        form={form}
        modelType={selectedModelType}
        open={advancedOpen}
        section={advancedSection}
        setForm={setForm}
        onClose={() => setAdvancedOpen(false)}
        onSectionChange={setAdvancedSection}
      />
    </Card>
  );
}

function isUnsupportedRealInterface(interfaceType: string) {
  return ["OpenAI-Response", "Anthropic", "Gemini"].includes(interfaceType);
}

function advancedSettingsSummary(form: WorkbenchForm, modelType: string) {
  const workload =
    modelType === "text_generation"
      ? `${form.streaming ? "Streaming" : "非 Streaming"} / ${form.max_output_tokens} tokens`
      : getModelTypeWorkloadLabel(modelType);
  const evidence = form.request_log_enabled
    ? form.request_log_capture_body
      ? "保存正文"
      : "保存索引"
    : "不保存明细";

  return `${workload} · Timeout ${form.request_timeout_seconds}s · ${evidence}`;
}

function getModelTypeWorkloadLabel(modelType: string) {
  if (modelType === "embedding") return "Embedding 批量";
  if (modelType === "rerank") return "Rerank 排序";
  if (modelType === "multimodal") return "Vision 图文";
  return "负载参数";
}
