import { useState } from "react";
import { Badge } from "../../../components/ui/Badge";
import { Button } from "../../../components/ui/Button";
import { Checkbox } from "../../../components/ui/Checkbox";
import { DataTable, type DataTableColumn } from "../../../components/ui/DataTable";
import { EmptyState } from "../../../components/ui/EmptyState";
import { Input } from "../../../components/ui/Input";
import { Network, Plus, RefreshCw } from "../../../components/ui/icons";
import type { ProviderSummary } from "../../../types/api";
import type { useEndpointProbeView } from "../hooks/useEndpointProbeView";

type EndpointProbeView = ReturnType<typeof useEndpointProbeView>;

export function EndpointProbeBatchTargets({ view }: { view: EndpointProbeView }) {
  const [expandedProviderId, setExpandedProviderId] = useState<string | null>(
    view.providers[0]?.id ?? null,
  );
  const [manualModels, setManualModels] = useState<Record<string, string>>({});

  const columns: Array<DataTableColumn<ProviderSummary>> = [
    {
      key: "provider",
      title: "服务商",
      render: (provider) => (
        <div className="endpoint-probe-provider-cell">
          <strong>{provider.name}</strong>
          <span>{provider.base_url_masked}</span>
        </div>
      ),
    },
    {
      key: "interface",
      title: "协议",
      width: 138,
      render: (provider) => provider.interface_type,
    },
    {
      key: "selected",
      title: "已选模型",
      align: "center",
      width: 92,
      render: (provider) => (
        <Badge tone={(view.batchModels[provider.id]?.length ?? 0) ? "success" : "neutral"}>
          {view.batchModels[provider.id]?.length ?? 0}
        </Badge>
      ),
    },
    {
      key: "sync",
      title: "同步",
      align: "center",
      width: 74,
      render: (provider) => (
        <Button
          aria-label={`同步 ${provider.name} 模型`}
          icon={<RefreshCw size={14} />}
          loading={view.scanningProviderId === provider.id}
          variant="ghost"
          onClick={() => view.refreshProviderModels(provider.id)}
        />
      ),
    },
  ];

  return (
    <section className="endpoint-probe-config-section endpoint-probe-batch-targets">
      <div className="endpoint-probe-section-title">
        <span>服务商与模型</span>
        <small>已明确选择 {view.selectedRunCount} 个请求，最多 200 个</small>
      </div>
      <DataTable
        className="endpoint-probe-provider-table"
        columns={columns}
        empty={
          <EmptyState
            compact
            icon={<Network size={20} />}
            title="没有可测活的服务商"
            description="请先通过右上角导入服务商，或在模型服务商页新增连接。"
          />
        }
        expandable={{
          expandedRowKey: expandedProviderId,
          expandOnRowClick: true,
          onExpandedRowChange: (key) => {
            const providerId = key ? String(key) : null;
            setExpandedProviderId(providerId);
            if (providerId) view.ensureProviderModels(providerId);
          },
          expandedRowRender: (provider) => {
            const models = view.providerModels[provider.id] ?? [];
            const selected = view.batchModels[provider.id] ?? [];
            const allChecked = models.length > 0 && models.every((model) => selected.includes(model.name));
            return (
              <div className="endpoint-probe-model-choices">
                <div className="endpoint-probe-model-choice-head">
                  <label>
                    <Checkbox
                      checked={allChecked}
                      indeterminate={!allChecked && selected.length > 0}
                      onChange={(event) => {
                        for (const model of models) {
                          view.toggleBatchModel(provider.id, model.name, event.target.checked);
                        }
                      }}
                    />
                    <span>选择当前模型列表</span>
                  </label>
                  <span>{models.length ? `共 ${models.length} 个模型` : "模型列表为空，可手动填写"}</span>
                </div>
                {models.length > 0 && (
                  <div className="endpoint-probe-model-choice-grid">
                    {models.map((model) => (
                      <label className="endpoint-probe-model-choice" key={model.name}>
                        <Checkbox
                          checked={selected.includes(model.name)}
                          onChange={(event) =>
                            view.toggleBatchModel(provider.id, model.name, event.target.checked)
                          }
                        />
                        <span>
                          <strong>{model.name}</strong>
                        </span>
                      </label>
                    ))}
                  </div>
                )}
                <div className="endpoint-probe-manual-model-row">
                  <Input
                    aria-label={`${provider.name} 手动模型名称`}
                    placeholder="手动补充模型名"
                    value={manualModels[provider.id] ?? ""}
                    onChange={(event) =>
                      setManualModels((current) => ({
                        ...current,
                        [provider.id]: event.target.value,
                      }))
                    }
                  />
                  <Button
                    icon={<Plus size={14} />}
                    onClick={() => {
                      view.addManualProviderModel(provider.id, manualModels[provider.id] ?? "");
                      setManualModels((current) => ({ ...current, [provider.id]: "" }));
                    }}
                  >
                    添加并勾选
                  </Button>
                </div>
              </div>
            );
          },
        }}
        getRowKey={(provider) => provider.id}
        rows={view.providers}
        scrollX={680}
      />
    </section>
  );
}
