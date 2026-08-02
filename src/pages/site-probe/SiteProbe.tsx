import { WorkspaceHeader } from "../../components/app-shell/WorkspaceHeader";
import { SiteProbeForm } from "../../features/site-probe/components/SiteProbeForm";
import { SiteProbeHistory } from "../../features/site-probe/components/SiteProbeHistory";
import { SiteProbeResult } from "../../features/site-probe/components/SiteProbeResult";
import { useSiteProbeView } from "../../features/site-probe/hooks/useSiteProbeView";

export function SiteProbe() {
  const view = useSiteProbeView();

  return (
    <div className="page site-probe-page">
      <WorkspaceHeader
        breadcrumb="真实端点联调"
        title="站点测活"
        subtitle="面向 new-api 等中转站的单站单模探测，支持 Chat Completions、Responses 和 Claude Messages。"
      />

      <div className="site-probe-layout">
        <SiteProbeForm
          form={view.form}
          manualModelEntry={view.manualModelEntry}
          modelOptions={view.modelOptions}
          modelScanError={view.modelScanError}
          modelScanMessage={view.modelScanMessage}
          running={view.running}
          scanningModels={view.scanningModels}
          setForm={view.setForm}
          onConnectionConfigChange={view.resetModelScan}
          onManualModelEntryChange={view.setManualModelEntry}
          onScanModels={view.scanModels}
          onSubmit={view.submit}
        />
        <SiteProbeResult
          detail={view.activeRun}
          error={view.detailError}
          loading={view.detailLoading}
        />
        <SiteProbeHistory
          deletingRunId={view.deletingRunId}
          error={view.historyError}
          history={view.history}
          keyword={view.keyword}
          loading={view.historyLoading}
          page={view.page}
          pageSize={view.pageSize}
          selectedRunId={view.selectedRunId}
          statusFilter={view.statusFilter}
          onDelete={view.deleteRun}
          onKeywordChange={view.setKeyword}
          onPageChange={view.setPage}
          onPageSizeChange={view.setPageSize}
          onSelect={view.setSelectedRunId}
          onStatusFilterChange={view.setStatusFilter}
        />
      </div>
    </div>
  );
}
