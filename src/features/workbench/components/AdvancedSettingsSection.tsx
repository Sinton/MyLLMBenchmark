import type { Dispatch, SetStateAction } from "react";
import { Disclosure } from "../../../components/common/Disclosure";
import { Input } from "../../../components/common/Input";
import { SelectField } from "../../../components/common/SelectField";
import { getModelTypeLabel } from "../../../lib/modelTaxonomy";
import { slaStopPolicyOptions } from "../constants";
import type { WorkbenchForm } from "../types";
import { WorkloadConfigSection } from "./WorkloadConfigSection";

type AdvancedSettingsSectionProps = {
  form: WorkbenchForm;
  modelType: string;
  setForm: Dispatch<SetStateAction<WorkbenchForm>>;
};

export function AdvancedSettingsSection({
  form,
  modelType,
  setForm,
}: AdvancedSettingsSectionProps) {
  return (
    <Disclosure
      className="advanced-settings-disclosure"
      description={buildAdvancedSummary(form, modelType)}
      title="高级参数"
    >
      <div className="advanced-settings-stack">
        <WorkloadConfigSection
          form={form}
          modelType={modelType}
          setForm={setForm}
        />
        <div className="form-grid">
          <Input
            label="预热轮次"
            hint="预热轮次不进入阶段汇总，用于让连接与请求窗口先稳定。"
            min={0}
            type="number"
            value={form.warmup_rounds}
            onChange={(event) =>
              setForm({
                ...form,
                warmup_rounds: Number(event.target.value),
                warmup_seconds: Number(event.target.value),
              })
            }
          />
          <Input
            label="请求超时"
            hint="单个 LLM 请求超过该时长才记为 timeout。"
            max={600}
            min={5}
            step={5}
            suffix="s"
            type="number"
            value={form.request_timeout_seconds}
            onChange={(event) =>
              setForm({
                ...form,
                request_timeout_seconds: Number(event.target.value),
              })
            }
          />
        </div>
        <div className="form-grid">
          <Input
            label="SLA P95"
            min={500}
            step={100}
            suffix="ms"
            type="number"
            value={form.sla_p95_ms}
            onChange={(event) =>
              setForm({ ...form, sla_p95_ms: Number(event.target.value) })
            }
          />
          <Input
            label="最低成功率"
            max={100}
            min={90}
            step={0.1}
            suffix="%"
            type="number"
            value={form.min_success_rate}
            onChange={(event) =>
              setForm({
                ...form,
                min_success_rate: Number(event.target.value),
              })
            }
          />
        </div>
        <SelectField
          label="SLA 失败后策略"
          options={slaStopPolicyOptions}
          value={form.sla_stop_policy}
          onChange={(sla_stop_policy) =>
            setForm({
              ...form,
              sla_stop_policy:
                sla_stop_policy as WorkbenchForm["sla_stop_policy"],
            })
          }
        />
      </div>
    </Disclosure>
  );
}

function buildAdvancedSummary(form: WorkbenchForm, modelType: string) {
  const workload = getModelTypeLabel(modelType);
  const sla = `P95 ${form.sla_p95_ms}ms / 成功率 ${form.min_success_rate}%`;
  const timeout = `超时 ${form.request_timeout_seconds}s`;
  const policy =
    form.sla_stop_policy === "stop_on_failure" ? "保护性停止" : "继续完整阶梯";

  if (modelType === "embedding") {
    return `${workload}，Batch ${form.embedding_batch_size}，${sla}，${timeout}，${policy}`;
  }
  if (modelType === "rerank") {
    return `${workload}，Docs/Query ${form.rerank_documents_per_query}，${sla}，${timeout}，${policy}`;
  }
  if (modelType === "multimodal") {
    return `${workload}，图片 ${form.vision_image_profile} x ${form.vision_image_count}，${sla}，${timeout}，${policy}`;
  }

  return `${workload}，Max Output ${form.max_output_tokens}，${
    form.streaming ? "Streaming 开启" : "Streaming 关闭"
  }，${sla}，${timeout}，${policy}`;
}
