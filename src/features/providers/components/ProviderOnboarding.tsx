import type { FormEvent } from "react";
import { Button } from "../../../components/ui/Button";
import { Input } from "../../../components/ui/Input";
import { Plus } from "../../../components/ui/icons";
import { SelectField } from "../../../components/ui/SelectField";
import type { ProviderInterfaceType } from "../../../types/api";
import { providerTypeOptions } from "../domain/providerView";
import { SecretInput } from "./SecretInput";

type ProviderFormState = {
  name: string;
  base_url: string;
  api_key: string;
  interface_type: ProviderInterfaceType;
};

type ProviderOnboardingProps = {
  form: ProviderFormState;
  mode?: "create" | "edit";
  saving: boolean;
  onCancel: () => void;
  onSubmit: (event: FormEvent) => void;
  setForm: (form: ProviderFormState) => void;
};

export function ProviderOnboarding({
  form,
  mode = "create",
  saving,
  onCancel,
  onSubmit,
  setForm,
}: ProviderOnboardingProps) {
  const isEditing = mode === "edit";

  return (
    <div className="provider-onboarding">
      <div className="provider-onboarding-copy">
        <div className="provider-onboarding-icon">
          <Plus size={22} />
        </div>
        <p className="eyebrow">{isEditing ? "编辑服务商" : "新增服务商"}</p>
        <h2>{isEditing ? "调整模型服务入口" : "连接一个模型服务入口"}</h2>
        <p>
          {isEditing
            ? "修改名称不会影响已有扫描结果；修改 Base URL、接口类型或 API Key 后，需要重新测试连接并扫描模型。"
            : "填入服务名称、接口类型、Base URL 和 API Key。后续模型扫描、压测任务和容量报告都会绑定到这个服务商。"}
        </p>
        <div className="provider-steps">
          {isEditing ? (
            <>
              <span>1. 保存修改</span>
              <span>2. 重新测试</span>
              <span>3. 必要时重扫模型</span>
            </>
          ) : (
            <>
              <span>1. 保存连接</span>
              <span>2. 测试可用性</span>
              <span>3. 扫描模型</span>
            </>
          )}
        </div>
      </div>

      <form className="provider-create-form" onSubmit={onSubmit}>
        <Input
          label="名称"
          placeholder="例如 某银行 Gemini 网关"
          value={form.name}
          onChange={(event) => setForm({ ...form, name: event.target.value })}
        />
        <SelectField
          label="接口类型"
          onChange={(interface_type) => setForm({ ...form, interface_type })}
          options={providerTypeOptions}
          value={form.interface_type}
        />
        <Input
          hint="OpenAI Compatible 推荐填写到 /v1；如果只填域名，系统会按 /v1 自动尝试。"
          label="Base URL"
          placeholder="https://api.example.com/v1"
          value={form.base_url}
          onChange={(event) => setForm({ ...form, base_url: event.target.value })}
        />
        <SecretInput
          hint={isEditing ? "当前 API Key 仅以掩码显示；保持不变可沿用，清空后保存表示移除密钥。" : undefined}
          label="API Key"
          onChange={(api_key) => setForm({ ...form, api_key })}
          placeholder="sk-..."
          value={form.api_key}
        />
        <div className="provider-create-actions">
          <Button disabled={saving} onClick={onCancel} type="button" variant="ghost">
            取消
          </Button>
          <Button
            disabled={saving}
            icon={<Plus size={16} />}
            loading={saving}
            type="submit"
            variant="primary"
          >
            {saving ? "保存中" : isEditing ? "保存修改" : "保存服务商"}
          </Button>
        </div>
      </form>
    </div>
  );
}
