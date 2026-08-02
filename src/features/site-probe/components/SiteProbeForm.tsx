import type { Dispatch, FormEvent, SetStateAction } from "react";
import { Badge } from "../../../components/ui/Badge";
import { Button } from "../../../components/ui/Button";
import { Card } from "../../../components/ui/Card";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { Input } from "../../../components/ui/Input";
import { KeyRound, Network, Play } from "../../../components/ui/icons";
import { SelectField } from "../../../components/ui/SelectField";
import { Textarea } from "../../../components/ui/Textarea";
import { Toggle } from "../../../components/ui/Toggle";
import type {
  SiteProbeInterfaceType,
  SiteProbeModelOption,
} from "../../../types/api";
import { siteProbeInterfaceOptions } from "../domain/siteProbePresentation";
import type { SiteProbeFormState } from "../hooks/useSiteProbeView";
import { SiteProbeModelField } from "./SiteProbeModelField";

type SiteProbeFormProps = {
  form: SiteProbeFormState;
  manualModelEntry: boolean;
  modelOptions: SiteProbeModelOption[];
  modelScanError: unknown;
  modelScanMessage: string | null;
  running: boolean;
  scanningModels: boolean;
  setForm: Dispatch<SetStateAction<SiteProbeFormState>>;
  onConnectionConfigChange: () => void;
  onManualModelEntryChange: (manual: boolean) => void;
  onScanModels: () => void;
  onSubmit: () => void;
};

export function SiteProbeForm({
  form,
  manualModelEntry,
  modelOptions,
  modelScanError,
  modelScanMessage,
  running,
  scanningModels,
  setForm,
  onConnectionConfigChange,
  onManualModelEntryChange,
  onScanModels,
  onSubmit,
}: SiteProbeFormProps) {
  const update = <K extends keyof SiteProbeFormState>(
    key: K,
    value: SiteProbeFormState[K],
  ) => {
    setForm((current) => ({ ...current, [key]: value }));
  };

  const updateConnection = (
    key: "base_url" | "api_key" | "interface_type",
    value: string,
  ) => {
    onConnectionConfigChange();
    setForm((current) => ({
      ...current,
      [key]: value,
      model: "",
    }));
  };

  const submit = (event: FormEvent) => {
    event.preventDefault();
    onSubmit();
  };

  return (
    <Card className="site-probe-form-card" title="测活配置">
      <form className="site-probe-form" onSubmit={submit}>
        <Input
          label="站点名称"
          placeholder="new-api 内网网关"
          value={form.name}
          onChange={(event) => update("name", event.target.value)}
        />
        <Input
          label="Base URL"
          placeholder="https://gateway.example.com/v1"
          prefix={<Network size={15} />}
          value={form.base_url}
          onChange={(event) => updateConnection("base_url", event.target.value)}
        />
        <Input
          label="API Key"
          placeholder="sk-..."
          prefix={<KeyRound size={15} />}
          type="password"
          value={form.api_key}
          onChange={(event) => updateConnection("api_key", event.target.value)}
        />
        <SelectField
          label="接口类型"
          value={form.interface_type}
          onChange={(value: SiteProbeInterfaceType) =>
            updateConnection("interface_type", value)
          }
          options={siteProbeInterfaceOptions}
        />
        <SiteProbeModelField
          manualEntry={manualModelEntry}
          model={form.model}
          models={modelOptions}
          scanError={modelScanError}
          scanMessage={modelScanMessage}
          scanning={scanningModels}
          onManualEntryChange={onManualModelEntryChange}
          onModelChange={(model) => update("model", model)}
          onScan={onScanModels}
        />
        <Textarea
          label="测试 Prompt"
          rows={7}
          value={form.prompt}
          onChange={(event) => update("prompt", event.target.value)}
        />

        <div className="site-probe-options">
          <div className="site-probe-toggle-row">
            <Toggle
              checked={form.streaming}
              ariaLabel="Streaming"
              onChange={(checked) => update("streaming", checked)}
            />
            <div className="site-probe-toggle-copy">
              <strong>流式响应</strong>
              <span>开启后按 SSE 读取首 token 延迟</span>
            </div>
          </div>
          <div className="site-probe-toggle-row">
            <Toggle
              checked={form.save_body}
              ariaLabel="保存正文"
              onChange={(checked) => update("save_body", checked)}
            />
            <div className="site-probe-toggle-copy">
              <strong>保存正文</strong>
              <span>历史记录保留 Prompt、响应和请求 Payload</span>
            </div>
          </div>
        </div>

        {form.save_body && (
          <InlineAlert tone="warning" title="敏感数据提示">
            Prompt、模型响应和请求 Payload 会写入本地历史，请确认其中不包含敏感业务数据。
          </InlineAlert>
        )}

        <div className="site-probe-number-grid">
          <Input
            hint="限制单次响应最多生成的 Token 数"
            label="最大输出 Token"
            min={1}
            max={8192}
            type="number"
            value={form.max_output_tokens}
            onChange={(event) =>
              update("max_output_tokens", Number(event.target.value))
            }
          />
          <Input
            hint="超过该时间仍未完成则判定超时"
            label="请求超时"
            min={5}
            max={600}
            suffix="秒"
            type="number"
            value={form.timeout_seconds}
            onChange={(event) =>
              update("timeout_seconds", Number(event.target.value))
            }
          />
        </div>

        <div className="site-probe-form-footer">
          <Badge tone="info">单站单模</Badge>
          <Button
            icon={<Play size={16} />}
            loading={running}
            type="submit"
            variant="primary"
          >
            开始测活
          </Button>
        </div>
      </form>
    </Card>
  );
}
