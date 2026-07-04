import type { ReactNode } from "react";
import {
  AlertCircle,
  Bot,
  CheckCircle2,
  Database,
  KeyRound,
  Link2,
  Activity,
} from "../../../components/common/icons";
import {
  getModelTypeDescription,
  getModelTypeLabel,
  MODEL_CAPABILITY_LABELS,
} from "../../../lib/modelTaxonomy";
import type {
  ModelSummary,
  ProviderConnectionResult,
  ProviderModelScanResult,
} from "../../../types/api";
import { capabilityNames, formatDate } from "../domain/providerView";

export function ConnectionResult({ result }: { result: ProviderConnectionResult }) {
  return (
    <div className={`connection-result ${result.ok ? "success" : "danger"}`}>
      {result.ok ? <CheckCircle2 size={18} /> : <AlertCircle size={18} />}
      <div>
        <strong>{result.ok ? "连接测试通过" : "连接测试失败"}</strong>
        <span>
          {result.message} · {formatDate(result.checked_at)}
        </span>
      </div>
    </div>
  );
}

export function ScanResult({ result }: { result: ProviderModelScanResult }) {
  return (
    <div className="connection-result success">
      <CheckCircle2 size={18} />
      <div>
        <strong>模型扫描完成</strong>
        <span>
          {result.message} · {formatDate(result.scanned_at)}
        </span>
      </div>
    </div>
  );
}

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
