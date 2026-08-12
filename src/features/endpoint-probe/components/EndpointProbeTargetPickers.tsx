import { useMemo, useState } from "react";
import { Badge } from "../../../components/ui/Badge";
import { Input } from "../../../components/ui/Input";
import { Popover } from "../../../components/ui/Popover";
import { Check, ChevronDown, Search } from "../../../components/ui/icons";
import { statusLabel, statusTone } from "../../../domain/statusPresentation";
import type { EndpointProbeModelOption, ProviderSummary } from "../../../types/api";
import { endpointProbeInterfaceLabel } from "../domain/endpointProbePresentation";

type EndpointProbeProviderPickerProps = {
  providers: ProviderSummary[];
  value: string;
  onChange: (providerId: string) => void;
};

export function EndpointProbeProviderPicker({
  providers,
  value,
  onChange,
}: EndpointProbeProviderPickerProps) {
  const [keyword, setKeyword] = useState("");
  const selected = providers.find((provider) => provider.id === value) ?? providers[0];
  const filtered = useMemo(
    () => filterProviders(providers, keyword),
    [keyword, providers],
  );

  return (
    <div className="endpoint-probe-picker-field">
      <span className="endpoint-probe-picker-label">服务商</span>
      <Popover
        className="endpoint-probe-provider-picker-popover"
        disabled={!providers.length}
        trigger={({ contentId, open, toggle }) => (
          <button
            aria-controls={open ? contentId : undefined}
            aria-expanded={open}
            aria-haspopup="listbox"
            className={`endpoint-probe-provider-trigger ${open ? "open" : ""}`}
            type="button"
            onClick={toggle}
          >
            {selected ? (
              <span className="endpoint-probe-provider-trigger-content">
                <span className="endpoint-probe-provider-trigger-title">
                  <strong>{selected.name}</strong>
                  <Badge tone={statusTone(selected.status)}>{statusLabel(selected.status)}</Badge>
                </span>
                <span className="endpoint-probe-provider-trigger-meta">
                  <small title={selected.base_url_masked}>{selected.base_url_masked}</small>
                  <em>{endpointProbeInterfaceLabel(selected.interface_type)}</em>
                </span>
              </span>
            ) : (
              <span className="endpoint-probe-provider-trigger-content">
                <strong>请选择服务商</strong>
                <small>导入或新增服务商后可用于测活</small>
              </span>
            )}
            <ChevronDown size={16} />
          </button>
        )}
      >
        {({ close }) => (
          <div className="endpoint-probe-picker-menu">
            <Input
              aria-label="搜索服务商"
              autoComplete="off"
              prefix={<Search size={14} />}
              placeholder="搜索名称、URL 或协议"
              value={keyword}
              onChange={(event) => setKeyword(event.target.value)}
            />
            <div className="endpoint-probe-picker-options" role="listbox">
              {filtered.length ? (
                filtered.map((provider) => (
                  <button
                    aria-selected={provider.id === value}
                    className={`endpoint-probe-provider-option ${
                      provider.id === value ? "selected" : ""
                    }`}
                    key={provider.id}
                    role="option"
                    type="button"
                    onClick={() => {
                      onChange(provider.id);
                      close();
                    }}
                  >
                    <span className="endpoint-probe-provider-option-main">
                      <strong>{provider.name}</strong>
                      <small title={provider.base_url_masked}>{provider.base_url_masked}</small>
                    </span>
                    <span className="endpoint-probe-provider-option-meta">
                      <Badge tone={statusTone(provider.status)}>
                        {statusLabel(provider.status)}
                      </Badge>
                      <em>{endpointProbeInterfaceLabel(provider.interface_type)}</em>
                    </span>
                    {provider.id === value && <Check size={15} />}
                  </button>
                ))
              ) : (
                <div className="endpoint-probe-picker-empty">没有匹配的服务商</div>
              )}
            </div>
          </div>
        )}
      </Popover>
    </div>
  );
}

type EndpointProbeModelPickerProps = {
  disabled?: boolean;
  models: EndpointProbeModelOption[];
  value: string;
  onChange: (model: string) => void;
};

export function EndpointProbeModelPicker({
  disabled = false,
  models,
  value,
  onChange,
}: EndpointProbeModelPickerProps) {
  const [keyword, setKeyword] = useState("");
  const selected = models.find((model) => model.name === value);
  const filtered = useMemo(
    () => filterModels(models, keyword),
    [keyword, models],
  );

  return (
    <Popover
      className="endpoint-probe-model-picker-popover"
      disabled={disabled || !models.length}
      trigger={({ contentId, open, toggle }) => (
        <button
          aria-controls={open ? contentId : undefined}
          aria-expanded={open}
          aria-haspopup="listbox"
          className={`endpoint-probe-model-trigger ${open ? "open" : ""}`}
          disabled={disabled || !models.length}
          type="button"
          onClick={toggle}
        >
          <span>
            <strong>{selected?.name ?? (value || "请选择模型")}</strong>
          </span>
          <ChevronDown size={16} />
        </button>
      )}
    >
      {({ close }) => (
        <div className="endpoint-probe-picker-menu">
          <Input
            aria-label="搜索模型"
            autoComplete="off"
            prefix={<Search size={14} />}
            placeholder="搜索模型名称"
            value={keyword}
            onChange={(event) => setKeyword(event.target.value)}
          />
          <div className="endpoint-probe-picker-options" role="listbox">
            {filtered.length ? (
              filtered.map((model) => (
                <button
                  aria-selected={model.name === value}
                  className={`endpoint-probe-model-option ${
                    model.name === value ? "selected" : ""
                  }`}
                  key={model.name}
                  role="option"
                  type="button"
                  onClick={() => {
                    onChange(model.name);
                    close();
                  }}
                >
                  <span>
                    <strong>{model.name}</strong>
                  </span>
                  {model.name === value && <Check size={15} />}
                </button>
              ))
            ) : (
              <div className="endpoint-probe-picker-empty">没有匹配的模型</div>
            )}
          </div>
        </div>
      )}
    </Popover>
  );
}

function filterProviders(providers: ProviderSummary[], keyword: string) {
  const normalized = keyword.trim().toLowerCase();
  if (!normalized) return providers;
  return providers.filter((provider) =>
    [
      provider.name,
      provider.base_url_masked,
      provider.interface_type,
      statusLabel(provider.status),
    ].some((value) => value.toLowerCase().includes(normalized)),
  );
}

function filterModels(models: EndpointProbeModelOption[], keyword: string) {
  const normalized = keyword.trim().toLowerCase();
  if (!normalized) return models;
  return models.filter((model) => model.name.toLowerCase().includes(normalized));
}
