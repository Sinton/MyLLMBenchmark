import { useMemo, useState } from "react";
import { Card } from "../../../components/ui/Card";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { Input } from "../../../components/ui/Input";
import { Search } from "../../../components/ui/icons";
import type { ProviderSummary } from "../../../types/api";
import { getInitials } from "../domain/providerView";

type ProviderRailProps = {
  providers: ProviderSummary[];
  selectedId?: string;
  onSelect: (id: string) => void;
};

export function ProviderRail({ providers, selectedId, onSelect }: ProviderRailProps) {
  const [keyword, setKeyword] = useState("");
  const visibleProviders = useMemo(() => {
    const normalized = keyword.trim().toLowerCase();
    if (!normalized) return providers;
    return providers.filter((provider) =>
      [provider.name, provider.base_url_masked, provider.interface_type]
        .join(" ")
        .toLowerCase()
        .includes(normalized),
    );
  }, [keyword, providers]);

  return (
    <aside className="provider-rail">
      <Card
        title="服务商列表"
        eyebrow="连接入口"
        action={<span className="rail-count">{visibleProviders.length}</span>}
      >
        <Input
          aria-label="筛选服务商"
          className="provider-search-field"
          onChange={(event) => setKeyword(event.target.value)}
          placeholder="按名称、Base URL 或接口类型筛选"
          prefix={<Search size={15} />}
          value={keyword}
        />
        <div className="provider-list">
          {visibleProviders.map((provider) => (
            <button
              className={`provider-item ${selectedId === provider.id ? "active" : ""}`}
              key={provider.id}
              onClick={() => onSelect(provider.id)}
              type="button"
            >
              <div className="provider-avatar">{getInitials(provider.name)}</div>
              <div className="provider-item-body">
                <div className="provider-item-title">
                  <strong title={provider.name}>{provider.name}</strong>
                </div>
                <span title={provider.base_url_masked}>{provider.base_url_masked}</span>
                <div className="provider-item-meta">
                  <span>{provider.interface_type}</span>
                  <span>{provider.model_count} 模型</span>
                </div>
              </div>
            </button>
          ))}
          {!providers.length && (
            <InlineAlert title="还没有服务商">请先新增一个服务商连接。</InlineAlert>
          )}
          {Boolean(providers.length) && !visibleProviders.length && (
            <InlineAlert title="没有匹配结果">换一个关键词试试。</InlineAlert>
          )}
        </div>
      </Card>
    </aside>
  );
}
