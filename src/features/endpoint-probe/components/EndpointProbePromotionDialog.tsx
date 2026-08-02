import { useEffect, useState } from "react";
import { Button } from "../../../components/ui/Button";
import { Dialog } from "../../../components/ui/Dialog";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { Input } from "../../../components/ui/Input";
import { Toggle } from "../../../components/ui/Toggle";
import type { EndpointProbeRunDetail } from "../../../types/api";

type EndpointProbePromotionDialogProps = {
  run: EndpointProbeRunDetail | null;
  defaultKey: string;
  pending: boolean;
  error: unknown;
  onClose: () => void;
  onSubmit: (name: string, apiKey: string, syncModels: boolean) => void;
};

export function EndpointProbePromotionDialog({
  run,
  defaultKey,
  pending,
  error,
  onClose,
  onSubmit,
}: EndpointProbePromotionDialogProps) {
  const [name, setName] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [syncModels, setSyncModels] = useState(true);

  useEffect(() => {
    if (!run) {
      setName("");
      setApiKey("");
      return;
    }
    setName(run.name === "临时站点" ? "" : run.name);
    setApiKey(defaultKey);
    setSyncModels(true);
  }, [defaultKey, run]);

  return (
    <Dialog
      open={Boolean(run)}
      title="保存为服务商"
      description="将已验证的临时连接加入模型服务商，后续可直接测活或压测。"
      width="520px"
      onClose={onClose}
      footer={
        <>
          <Button variant="ghost" onClick={onClose}>取消</Button>
          <Button
            disabled={!name.trim() || !apiKey.trim()}
            loading={pending}
            variant="primary"
            onClick={() => onSubmit(name, apiKey, syncModels)}
          >
            保存服务商
          </Button>
        </>
      }
    >
      {run && (
        <div className="endpoint-probe-dialog-form">
          <div className="endpoint-probe-promote-target">
            <span>{run.base_url}</span>
            <strong>{run.interface_type} · {run.model}</strong>
          </div>
          <Input
            data-autofocus
            label="服务商名称"
            placeholder="例如：华东中转站"
            value={name}
            onChange={(event) => setName(event.target.value)}
          />
          <Input
            autoComplete="off"
            label="API Key"
            hint="Key 不会从测活历史恢复；历史记录收录时需要重新填写。"
            placeholder="请输入用于该服务商的 Key"
            type="password"
            value={apiKey}
            onChange={(event) => setApiKey(event.target.value)}
          />
          <div className="endpoint-probe-toggle-item">
            <div>
              <strong>同步模型列表</strong>
              <span>同步失败时仍会保存本次已验证模型。</span>
            </div>
            <Toggle
              ariaLabel="同步模型列表"
              checked={syncModels}
              onChange={setSyncModels}
            />
          </div>
          <InlineAlert tone="info">
            将按规范化 Base URL + 接口类型检查重复，不会覆盖已有名称或 Key。
          </InlineAlert>
          {Boolean(error) && (
            <InlineAlert tone="danger" title="保存失败">
              {error instanceof Error ? error.message : String(error)}
            </InlineAlert>
          )}
        </div>
      )}
    </Dialog>
  );
}
