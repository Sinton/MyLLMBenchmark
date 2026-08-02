import { Button } from "../../../components/ui/Button";
import { InlineAlert } from "../../../components/ui/InlineAlert";
import { Input } from "../../../components/ui/Input";
import { ListChecks, Pencil, RefreshCw } from "../../../components/ui/icons";
import { SelectField } from "../../../components/ui/SelectField";
import type { SiteProbeModelOption } from "../../../types/api";
import { siteProbeModelDescription } from "../domain/siteProbePresentation";

type SiteProbeModelFieldProps = {
  manualEntry: boolean;
  model: string;
  models: SiteProbeModelOption[];
  scanError: unknown;
  scanMessage: string | null;
  scanning: boolean;
  onManualEntryChange: (manual: boolean) => void;
  onModelChange: (model: string) => void;
  onScan: () => void;
};

export function SiteProbeModelField({
  manualEntry,
  model,
  models,
  scanError,
  scanMessage,
  scanning,
  onManualEntryChange,
  onModelChange,
  onScan,
}: SiteProbeModelFieldProps) {
  const hasModels = models.length > 0;

  return (
    <section className="site-probe-model-field">
      <div className="site-probe-field-heading">
        <span>模型</span>
        <Button
          className="site-probe-model-mode"
          disabled={scanning || (manualEntry && !hasModels)}
          icon={manualEntry ? <ListChecks size={14} /> : <Pencil size={14} />}
          type="button"
          variant="ghost"
          onClick={() => onManualEntryChange(!manualEntry)}
        >
          {manualEntry ? "使用模型列表" : "手动填写"}
        </Button>
      </div>

      <div className="site-probe-model-row">
        {manualEntry ? (
          <Input
            aria-label="模型名称"
            placeholder="输入中转站实际暴露的模型 ID"
            value={model}
            onChange={(event) => onModelChange(event.target.value)}
          />
        ) : (
          <SelectField
            ariaLabel="模型"
            disabled={!hasModels || scanning}
            placeholder={scanning ? "正在获取模型..." : "先获取模型列表"}
            value={model}
            options={models.map((item) => ({
              value: item.name,
              label: item.name,
              description: siteProbeModelDescription(item),
            }))}
            onChange={onModelChange}
          />
        )}
        <Button
          className="site-probe-model-scan"
          icon={<RefreshCw className={scanning ? "spin" : ""} size={15} />}
          loading={scanning}
          title="从 /models 获取模型列表"
          type="button"
          onClick={onScan}
        >
          {hasModels ? "重新获取" : "获取模型"}
        </Button>
      </div>

      {!manualEntry && !hasModels && !scanError && !scanMessage && (
        <p className="site-probe-model-hint">
          填写 Base URL 和 API Key 后，从站点的 /models 接口读取可用模型。
        </p>
      )}
      {scanMessage && !scanError && (
        <p className="site-probe-model-feedback">{scanMessage}</p>
      )}
      {Boolean(scanError) && (
        <InlineAlert tone="danger" title="模型列表获取失败">
          {scanError instanceof Error ? scanError.message : String(scanError)}
        </InlineAlert>
      )}
    </section>
  );
}
