import { useState } from "react";
import { WorkspaceHeader } from "../../components/app-shell/WorkspaceHeader";
import { Button } from "../../components/ui/Button";
import { Upload } from "../../components/ui/icons";
import { EndpointProbeConfiguration } from "../../features/endpoint-probe/components/EndpointProbeConfiguration";
import { EndpointProbeHistory } from "../../features/endpoint-probe/components/EndpointProbeHistory";
import { EndpointProbePromotionDialog } from "../../features/endpoint-probe/components/EndpointProbePromotionDialog";
import { EndpointProbeResults } from "../../features/endpoint-probe/components/EndpointProbeResults";
import { ProviderImportDialog } from "../../features/endpoint-probe/components/ProviderImportDialog";
import { useEndpointProbeView } from "../../features/endpoint-probe/hooks/useEndpointProbeView";

export function EndpointProbe() {
  const view = useEndpointProbeView();
  const [importOpen, setImportOpen] = useState(false);

  return (
    <div className="page endpoint-probe-page">
      <WorkspaceHeader
        breadcrumb="真实端点联调"
        title="站点测活"
        subtitle="用自定义 Prompt 检查中转站、API Key、协议和模型，并实时查看真实 SSE 响应。"
        actions={
          <Button icon={<Upload size={15} />} onClick={() => setImportOpen(true)}>
            导入服务商
          </Button>
        }
      />

      <div className="endpoint-probe-workspace">
        <EndpointProbeConfiguration view={view} />
        <EndpointProbeResults view={view} />
      </div>
      <EndpointProbeHistory view={view} />

      <EndpointProbePromotionDialog
        defaultKey={view.promotionDefaultKey}
        error={view.promotionError}
        pending={view.promotionPending}
        run={view.promotionRun}
        onClose={() => view.setPromotionRun(null)}
        onSubmit={view.submitPromotion}
      />
      <ProviderImportDialog
        open={importOpen}
        pending={view.importPending}
        result={view.importResult}
        onClose={() => setImportOpen(false)}
        onImport={view.importProviders}
      />
    </div>
  );
}
