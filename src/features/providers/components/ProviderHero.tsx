import type { ReactNode } from "react";
import { useState } from "react";
import { Badge, statusLabel, statusTone } from "../../../components/common/Badge";
import { Button } from "../../../components/common/Button";
import { Card } from "../../../components/common/Card";
import { Dialog } from "../../../components/common/Dialog";
import {
  Activity,
  AlertCircle,
  AlertTriangle,
  Database,
  KeyRound,
  Link2,
  Network,
  Pencil,
  RefreshCw,
  Search,
  Settings2,
  Trash2,
} from "../../../components/common/icons";
import type { ProviderDiagnosticsResult, ProviderSummary } from "../../../types/api";
import { formatDate, getInitials } from "../domain/providerView";

type ProviderHeroProps = {
  canScan: boolean;
  deleteError?: unknown;
  deleting: boolean;
  diagnosticsError?: unknown;
  diagnosticsPending: boolean;
  diagnosticsResult: ProviderDiagnosticsResult | null;
  getErrorMessage: (error: unknown) => string;
  modelsFetching: boolean;
  scanError?: unknown;
  scanPending: boolean;
  selected: ProviderSummary;
  selectedModelCount: number;
  testError?: unknown;
  testPending: boolean;
  onDelete: () => Promise<void>;
  onDiagnose: () => void;
  onEdit: () => void;
  onScan: () => void;
  onTestConnection: () => void;
};

export function ProviderHero({
  canScan,
  deleteError,
  deleting,
  diagnosticsError,
  diagnosticsPending,
  diagnosticsResult,
  getErrorMessage,
  modelsFetching,
  scanError,
  scanPending,
  selected,
  selectedModelCount,
  testError,
  testPending,
  onDelete,
  onDiagnose,
  onEdit,
  onScan,
  onTestConnection,
}: ProviderHeroProps) {
  const [deleteOpen, setDeleteOpen] = useState(false);

  const confirmDelete = async () => {
    try {
      await onDelete();
      setDeleteOpen(false);
    } catch {
      // The mutation state is rendered in the dialog, so keep it open.
    }
  };

  return (
    <>
      <Card className="provider-hero">
        <div className="provider-hero-top">
          <div className="provider-title-block">
            <div className="provider-avatar large">{getInitials(selected.name)}</div>
            <div>
              <div className="eyebrow">当前服务商</div>
              <h2 title={selected.name}>{selected.name}</h2>
              <p>
                {selected.interface_type} 接口入口，用于连接测试、模型扫描和后续压测任务。
              </p>
            </div>
          </div>
          <div className="provider-action-stack">
            <Badge tone={statusTone(selected.status)}>{statusLabel(selected.status)}</Badge>
            <div className="provider-primary-actions">
              <Button icon={<Pencil size={15} />} onClick={onEdit}>
                编辑
              </Button>
              <Button
                disabled={testPending}
                icon={<RefreshCw className={testPending ? "spin" : ""} size={15} />}
                loading={testPending}
                onClick={onTestConnection}
              >
                {testPending ? "检测中" : "测试连接"}
              </Button>
              <Button
                disabled={!canScan || scanPending}
                icon={<Search className={scanPending ? "spin" : ""} size={15} />}
                loading={scanPending}
                onClick={onScan}
              >
                {scanPending ? "扫描中" : "扫描模型"}
              </Button>
              <Button
                disabled={diagnosticsPending}
                icon={<Network className={diagnosticsPending ? "spin" : ""} size={15} />}
                loading={diagnosticsPending}
                onClick={onDiagnose}
              >
                {diagnosticsPending ? "诊断中" : "兼容性诊断"}
              </Button>
            </div>
          </div>
        </div>

        {Boolean(scanError) && (
          <div className="connection-result danger">
            <AlertCircle size={18} />
            <div>
              <strong>模型扫描失败</strong>
              <span>{getErrorMessage(scanError)}</span>
            </div>
          </div>
        )}
        {Boolean(testError) && (
          <div className="connection-result danger">
            <AlertCircle size={18} />
            <div>
              <strong>连接测试失败</strong>
              <span>{getErrorMessage(testError)}</span>
            </div>
          </div>
        )}
        {diagnosticsResult?.provider_id === selected.id && (
          <ProviderDiagnosticsPanel result={diagnosticsResult} />
        )}
        {Boolean(diagnosticsError) && (
          <div className="connection-result danger">
            <AlertCircle size={18} />
            <div>
              <strong>兼容性诊断失败</strong>
              <span>{getErrorMessage(diagnosticsError)}</span>
            </div>
          </div>
        )}

        <div className="provider-detail-list">
          <DetailRow
            icon={<Link2 size={16} />}
            label="Base URL"
            title={selected.base_url_masked}
            value={selected.base_url_masked}
            wide
          />
          <DetailRow
            icon={<KeyRound size={16} />}
            label="API Key"
            title={selected.api_key_masked}
            value={selected.api_key_masked}
            wide
          />
          <DetailRow icon={<Settings2 size={16} />} label="接口类型" value={selected.interface_type} />
          <DetailRow
            icon={<Database size={16} />}
            label="模型数量"
            value={modelsFetching ? "刷新中" : `${selectedModelCount} 个`}
          />
          <DetailRow
            icon={<Activity size={16} />}
            label="最近检测"
            title={selected.last_checked_at ? formatDate(selected.last_checked_at) : "未检测"}
            value={selected.last_checked_at ? formatDate(selected.last_checked_at) : "未检测"}
          />
        </div>

        <div className="provider-danger-row">
          <div>
            <strong>危险操作</strong>
            <span>删除服务商会清理关联的压测任务、模型缓存和报告记录。</span>
          </div>
          <Button
            disabled={deleting}
            icon={<Trash2 size={15} />}
            loading={deleting}
            onClick={() => setDeleteOpen(true)}
            variant="danger"
          >
            删除服务商
          </Button>
        </div>
      </Card>

      <Dialog
        description="该操作会同步清理这个服务商下的模型缓存、压测任务和测试报告，删除后无法在当前应用内恢复。"
        footer={
          <>
            <Button disabled={deleting} onClick={() => setDeleteOpen(false)} variant="ghost">
              取消
            </Button>
            <Button
              disabled={deleting}
              icon={<Trash2 size={15} />}
              loading={deleting}
              onClick={confirmDelete}
              variant="danger"
            >
              确认删除
            </Button>
          </>
        }
        open={deleteOpen}
        title={`删除「${selected.name}」？`}
        onClose={() => !deleting && setDeleteOpen(false)}
      >
        <div className="provider-delete-dialog">
          <div className="provider-delete-icon">
            <AlertTriangle size={22} />
          </div>
          <div>
            <strong>{selected.name}</strong>
            <span>{selected.base_url_masked}</span>
          </div>
        </div>
        {Boolean(deleteError) && (
          <div className="connection-result danger provider-delete-error">
            <AlertCircle size={18} />
            <div>
              <strong>删除失败</strong>
              <span>{getErrorMessage(deleteError)}</span>
            </div>
          </div>
        )}
      </Dialog>
    </>
  );
}

function ProviderDiagnosticsPanel({ result }: { result: ProviderDiagnosticsResult }) {
  const tone =
    result.status === "passed"
      ? "success"
      : result.status === "unsupported" || result.status === "failed"
        ? "danger"
        : "info";
  const title =
    result.status === "passed"
      ? "兼容性诊断通过"
      : result.status === "unsupported"
        ? "当前接口未启用真实引擎"
        : result.status === "failed"
          ? "兼容性诊断未通过"
          : "兼容性诊断存在警告";

  return (
    <div className={`provider-diagnostics ${tone}`}>
      <div className="provider-diagnostics-header">
        <div>
          <strong>{title}</strong>
          <span>
            引擎 {result.engine_mode} · {formatDate(result.checked_at)}
          </span>
        </div>
        <Badge tone={tone === "danger" ? "danger" : tone === "success" ? "success" : "info"}>
          {result.status}
        </Badge>
      </div>
      <div className="provider-diagnostics-grid">
        {result.endpoints.map((endpoint) => (
          <div
            key={`${endpoint.method}-${endpoint.path}-${endpoint.name}`}
            className={`diagnostic-endpoint ${endpoint.ok ? "ok" : "failed"}`}
          >
            <div className="diagnostic-endpoint-top">
              <span>{endpoint.name}</span>
              <Badge tone={endpoint.ok ? "success" : "danger"}>
                {endpoint.ok ? "通过" : endpoint.error_kind ?? "失败"}
              </Badge>
            </div>
            <strong>
              {endpoint.method} {endpoint.path}
            </strong>
            <p>{endpoint.message}</p>
            <em>
              {endpoint.http_status ? `HTTP ${endpoint.http_status}` : "本地检查"}
              {endpoint.latency_ms ? ` · ${endpoint.latency_ms}ms` : ""}
            </em>
          </div>
        ))}
      </div>
      <div className="provider-diagnostics-recommendations">
        {result.recommendations.map((item) => (
          <span key={item}>{item}</span>
        ))}
      </div>
    </div>
  );
}

type DetailRowProps = {
  action?: ReactNode;
  icon: ReactNode;
  label: string;
  title?: string;
  value: string;
  wide?: boolean;
};

function DetailRow({ action, icon, label, title, value, wide = false }: DetailRowProps) {
  return (
    <div className={`provider-detail-row ${wide ? "wide" : ""}`.trim()}>
      <div className="info-icon">{icon}</div>
      <div className="provider-detail-copy">
        <span>{label}</span>
        <strong title={title ?? value}>{value}</strong>
      </div>
      {action}
    </div>
  );
}
