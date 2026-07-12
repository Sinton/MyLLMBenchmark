import type { ReactNode } from "react";
import {
  Activity,
  Bot,
  Database,
  KeyRound,
  Link2,
} from "../../../components/common/icons";
import {
  getModelTypeDescription,
  getModelTypeLabel,
  MODEL_CAPABILITY_LABELS,
} from "../../../lib/modelTaxonomy";
import type { ModelSummary } from "../../../types/api";
import { capabilityNames } from "../domain/providerView";

export function ModelRow({ model }: { model: ModelSummary }) {
  const capabilities = capabilityNames(model);

  return (
    <div className="model-row">
      <div className="model-row-icon">
        <Bot size={16} />
      </div>
      <div className="model-row-main">
        <strong title={model.name}>{model.name}</strong>
        <span>{getModelTypeDescription(model.model_type)}</span>
        <div className="model-row-tags">
          <em>{getModelTypeLabel(model.model_type)}</em>
          {capabilities.map((capability) => (
            <em key={capability}>{MODEL_CAPABILITY_LABELS[capability]}</em>
          ))}
        </div>
      </div>
      <div className="model-row-meta">
        {model.recommended_concurrency
          ? `推荐并发 ${model.recommended_concurrency}`
          : "待压测"}
      </div>
    </div>
  );
}

export function InfoItem({
  icon,
  label,
  value,
}: {
  icon: ReactNode;
  label: string;
  value: string;
}) {
  return (
    <div className="info-item">
      <div className="info-icon">{icon}</div>
      <div>
        <span>{label}</span>
        <strong title={value}>{value}</strong>
      </div>
    </div>
  );
}

export const providerInfoIcons = {
  link: <Link2 size={16} />,
  key: <KeyRound size={16} />,
  models: <Database size={16} />,
  checkedAt: <Activity size={16} />,
};
