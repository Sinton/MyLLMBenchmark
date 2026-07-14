import type { Dispatch, SetStateAction } from "react";
import { Dialog } from "../../../components/common/Dialog";
import { Input } from "../../../components/common/Input";
import { InlineAlert } from "../../../components/common/InlineAlert";
import { SelectField } from "../../../components/common/SelectField";
import { Tabs } from "../../../components/common/Tabs";
import { Toggle } from "../../../components/common/Toggle";
import { slaStopPolicyOptions } from "../constants";
import type { WorkbenchForm } from "../types";
import { WorkloadConfigSection } from "./WorkloadConfigSection";

export type AdvancedSettingsSectionKey = "workload" | "protection" | "evidence";

type AdvancedSettingsDrawerProps = {
  form: WorkbenchForm;
  modelType: string;
  open: boolean;
  section: AdvancedSettingsSectionKey;
  setForm: Dispatch<SetStateAction<WorkbenchForm>>;
  onClose: () => void;
  onSectionChange: (section: AdvancedSettingsSectionKey) => void;
};

const advancedTabs: Array<{ key: AdvancedSettingsSectionKey; label: string }> = [
  { key: "workload", label: "负载参数" },
  { key: "protection", label: "运行保护" },
  { key: "evidence", label: "证据采集" },
];

export function AdvancedSettingsDrawer({
  form,
  modelType,
  open,
  section,
  setForm,
  onClose,
  onSectionChange,
}: AdvancedSettingsDrawerProps) {
  return (
    <Dialog
      description="高级参数会立即写入当前压测配置，关闭后仍会保留。"
      open={open}
      title="高级设置"
      variant="drawer"
      width="520px"
      onClose={onClose}
    >
      <div className="advanced-settings-drawer">
        <Tabs
          ariaLabel="高级设置分类"
          className="advanced-settings-tabs"
          items={advancedTabs}
          variant="line"
          value={section}
          onChange={onSectionChange}
        />
        {section === "workload" && (
          <section className="advanced-settings-panel">
            <SectionHeading
              description="控制单次请求的模型负载形态，按当前模型类型切换。"
              title="负载参数"
            />
            <WorkloadConfigSection
              form={form}
              modelType={modelType}
              setForm={setForm}
            />
          </section>
        )}
        {section === "protection" && (
          <section className="advanced-settings-panel">
            <SectionHeading
              description="控制请求超时、SLA 判断和不达标后的运行策略。"
              title="运行保护"
            />
            <ProtectionSettings form={form} setForm={setForm} />
          </section>
        )}
        {section === "evidence" && (
          <section className="advanced-settings-panel">
            <SectionHeading
              description="控制是否保存请求级证据，默认不保存正文。"
              title="证据采集"
            />
            <EvidenceSettings form={form} setForm={setForm} />
          </section>
        )}
      </div>
    </Dialog>
  );
}

function ProtectionSettings({
  form,
  setForm,
}: {
  form: WorkbenchForm;
  setForm: Dispatch<SetStateAction<WorkbenchForm>>;
}) {
  return (
    <div className="advanced-settings-stack">
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
  );
}

function EvidenceSettings({
  form,
  setForm,
}: {
  form: WorkbenchForm;
  setForm: Dispatch<SetStateAction<WorkbenchForm>>;
}) {
  return (
    <div className="request-log-config request-log-config-drawer">
      <div className="request-log-config-header">
        <strong>请求级证据链</strong>
        <span>用于在报告中按请求查看耗时、TTFT、Token、错误和可选正文。</span>
      </div>
      <div className="request-log-option">
        <div className="request-log-option-copy">
          <strong>保存请求明细索引</strong>
          <span>记录每次请求的状态、耗时、Token 和 Prompt / 响应摘要。</span>
        </div>
        <Toggle
          ariaLabel="保存请求明细索引"
          checked={form.request_log_enabled}
          onChange={(request_log_enabled) =>
            setForm({
              ...form,
              request_log_enabled,
              request_log_capture_body: request_log_enabled
                ? form.request_log_capture_body
                : false,
            })
          }
        />
      </div>
      {form.request_log_enabled && (
        <>
          <div className="request-log-option">
            <div className="request-log-option-copy">
              <strong>保存 Prompt / 响应正文</strong>
              <span>完整正文写入本地 JSONL，关闭时只保留摘要和指标。</span>
            </div>
            <Toggle
              ariaLabel="保存 Prompt / 响应正文"
              checked={form.request_log_capture_body}
              onChange={(request_log_capture_body) =>
                setForm({ ...form, request_log_capture_body })
              }
            />
          </div>
          <Input
            className="request-log-limit"
            label="每阶段最多保存"
            hint="超过上限后仍继续压测，只是不再保存更多请求明细。"
            max={1000}
            min={1}
            step={50}
            suffix="条"
            type="number"
            value={form.request_log_max_records_per_stage}
            onChange={(event) =>
              setForm({
                ...form,
                request_log_max_records_per_stage: Number(event.target.value),
              })
            }
          />
          {form.request_log_capture_body && (
            <InlineAlert tone="warning" title="敏感数据提示">
              开启后会在本地保存 Prompt 和模型响应正文，请确认样本中不包含敏感信息。
            </InlineAlert>
          )}
        </>
      )}
    </div>
  );
}

function SectionHeading({
  description,
  title,
}: {
  description: string;
  title: string;
}) {
  return (
    <div className="advanced-settings-panel-heading">
      <h3>{title}</h3>
      <p>{description}</p>
    </div>
  );
}
