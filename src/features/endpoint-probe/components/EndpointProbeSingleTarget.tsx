import { useEffect, useState } from "react";
import { Badge } from "../../../components/ui/Badge";
import { Button } from "../../../components/ui/Button";
import { Input } from "../../../components/ui/Input";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { RefreshCw } from "../../../components/ui/icons";
import { SelectField } from "../../../components/ui/SelectField";
import { Tabs } from "../../../components/ui/Tabs";
import { Tooltip } from "../../../components/ui/Tooltip";
import type { EndpointProbeInterfaceType } from "../../../types/api";
import {
  endpointProbeInterfaceOptions,
  endpointProbeModelDescription,
} from "../domain/endpointProbePresentation";
import type { useEndpointProbeView } from "../hooks/useEndpointProbeView";

type EndpointProbeView = ReturnType<typeof useEndpointProbeView>;

export function EndpointProbeSingleTarget({ view }: { view: EndpointProbeView }) {
  const [manualModel, setManualModel] = useState(false);
  const selectedProvider = view.providers.find((item) => item.id === view.singleProviderId);

  useEffect(() => {
    if (view.singleProviderModels.length) setManualModel(false);
  }, [view.singleProviderId, view.singleProviderModels.length]);

  const updateTemporary = <K extends keyof typeof view.temporary>(
    key: K,
    value: (typeof view.temporary)[K],
    resetsModels = false,
  ) => {
    view.setTemporary((current) => ({ ...current, [key]: value }));
    if (resetsModels) view.resetTemporaryModels();
  };

  return (
    <section className="endpoint-probe-config-section">
      <div className="endpoint-probe-section-title endpoint-probe-source-heading">
        <span>测试目标</span>
      </div>
      <Tabs
        ariaLabel="站点来源"
        className="endpoint-probe-source-tabs"
        items={[
          { key: "provider", label: "已保存服务商" },
          { key: "temporary", label: "临时站点" },
        ]}
        value={view.singleSource}
        onChange={view.setSingleSource}
      />

      {view.singleSource === "provider" ? (
        <div className="endpoint-probe-field-stack">
          {view.providers.length ? (
            <>
              <SelectField
                label="服务商"
                options={view.providers.map((provider) => ({
                  label: provider.name,
                  value: provider.id,
                  description: `${provider.interface_type} · ${provider.base_url_masked}`,
                }))}
                value={view.singleProviderId}
                onChange={view.setSingleProviderId}
              />
              {selectedProvider && (
                <div className="endpoint-probe-target-endpoint">
                  <span className="endpoint-probe-target-endpoint-label">端点</span>
                  <div className="endpoint-probe-target-endpoint-value">
                    <Tooltip
                      ariaLabel={`端点：${selectedProvider.base_url_masked}`}
                      content={selectedProvider.base_url_masked}
                    >
                      <span className="endpoint-probe-target-endpoint-url">
                        {selectedProvider.base_url_masked}
                      </span>
                    </Tooltip>
                    <Badge tone={selectedProvider.status === "online" ? "success" : "neutral"}>
                      {selectedProvider.interface_type}
                    </Badge>
                  </div>
                </div>
              )}
              <ModelField
                manual={manualModel || !view.singleProviderModels.length}
                model={view.singleProviderModel}
                models={view.singleProviderModels}
                scanning={view.scanningProviderId === view.singleProviderId}
                onManualChange={setManualModel}
                onModelChange={view.setSingleProviderModel}
                onScan={() => view.refreshProviderModels(view.singleProviderId)}
              />
            </>
          ) : (
            <InlineAlert tone="warning" title="暂无可测活服务商">
              请先导入服务商，或切换到“临时站点”直接测试。
            </InlineAlert>
          )}
        </div>
      ) : (
        <div className="endpoint-probe-field-stack">
          <div className="endpoint-probe-form-grid two">
            <Input
              label="站点名称（可选）"
              placeholder="例如：华东中转站"
              value={view.temporary.name}
              onChange={(event) => updateTemporary("name", event.target.value)}
            />
            <SelectField<EndpointProbeInterfaceType>
              label="接口类型"
              options={endpointProbeInterfaceOptions}
              value={view.temporary.interface_type}
              onChange={(value) => updateTemporary("interface_type", value, true)}
            />
          </div>
          <Input
            label="Base URL"
            placeholder="https://api.example.com/v1"
            value={view.temporary.base_url}
            onChange={(event) => updateTemporary("base_url", event.target.value, true)}
          />
          <Input
            autoComplete="off"
            label="API Key"
            placeholder="仅用于本次请求，不写入测活历史"
            type="password"
            value={view.temporary.api_key}
            onChange={(event) => updateTemporary("api_key", event.target.value, true)}
          />
          <ModelField
            manual={manualModel || !view.temporaryModels.length}
            model={view.temporary.model}
            models={view.temporaryModels}
            scanning={view.scanningTemporary}
            onManualChange={setManualModel}
            onModelChange={(value) => updateTemporary("model", value)}
            onScan={view.scanTemporaryModels}
          />
        </div>
      )}
    </section>
  );
}

type ModelFieldProps = {
  manual: boolean;
  model: string;
  models: EndpointProbeView["temporaryModels"];
  scanning: boolean;
  onManualChange: (manual: boolean) => void;
  onModelChange: (model: string) => void;
  onScan: () => void;
};

function ModelField({
  manual,
  model,
  models,
  scanning,
  onManualChange,
  onModelChange,
  onScan,
}: ModelFieldProps) {
  return (
    <div className="endpoint-probe-model-field">
      <div className="endpoint-probe-field-label">
        <span>模型</span>
        <div className="endpoint-probe-field-actions">
          {models.length > 0 && (
            <Button variant="ghost" onClick={() => onManualChange(!manual)}>
              {manual ? "使用模型列表" : "手动填写"}
            </Button>
          )}
          <Button
            icon={<RefreshCw size={14} />}
            loading={scanning}
            variant="ghost"
            onClick={onScan}
          >
            从 /models 同步
          </Button>
        </div>
      </div>
      {manual ? (
        <Input
          aria-label="模型名称"
          placeholder="例如：gpt-4.1-mini"
          value={model}
          onChange={(event) => onModelChange(event.target.value)}
        />
      ) : (
        <SelectField
          ariaLabel="选择模型"
          options={models.map((item) => ({
            label: item.name,
            value: item.name,
            description: endpointProbeModelDescription(item),
          }))}
          placeholder="请选择模型"
          value={model}
          onChange={onModelChange}
        />
      )}
    </div>
  );
}
