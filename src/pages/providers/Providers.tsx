import { Button } from "../../components/common/Button";
import { Card } from "../../components/common/Card";
import { Dialog } from "../../components/common/Dialog";
import { EmptyState } from "../../components/common/EmptyState";
import { Plus } from "../../components/common/icons";
import { PageHeader } from "../../components/common/PageHeader";
import { ProviderHero } from "../../features/providers/components/ProviderHero";
import { ProviderOnboarding } from "../../features/providers/components/ProviderOnboarding";
import { ProviderPanels } from "../../features/providers/components/ProviderPanels";
import { ProviderRail } from "../../features/providers/components/ProviderRail";
import { useProvidersController } from "../../features/providers/hooks/useProvidersController";

export function Providers() {
  const providersView = useProvidersController();

  return (
    <div className="page">
      <PageHeader
        eyebrow="服务商管理"
        title="模型服务商"
        description="管理 OpenAI、OpenAI-Response、Anthropic、Gemini 和 Jina Rerank 等服务入口。当前试点阶段会在本地保存并展示 API Key 明文。"
        actions={
          providersView.providers.length > 0 ? (
            <Button
              icon={<Plus size={16} />}
              onClick={() => providersView.setIsCreating(true)}
              variant="primary"
            >
              新增服务商
            </Button>
          ) : undefined
        }
      />

      <div className="provider-console">
        <ProviderRail
          providers={providersView.providers}
          selectedId={providersView.selected?.id}
          onSelect={providersView.setSelectedId}
        />

        <section className="provider-workspace">
          {providersView.showEmptyOnboarding && (
            <Card className="provider-empty-card">
              <EmptyState
                action={
                  <Button
                    icon={<Plus size={16} />}
                    onClick={() => providersView.setIsCreating(true)}
                    variant="primary"
                  >
                    新增服务商
                  </Button>
                }
                icon={<Plus size={22} />}
                title="先连接一个模型服务商"
                description="保存服务入口后，就可以测试连接、扫描模型，并在压测工作台中选择它作为测试对象。"
              />
            </Card>
          )}

          {providersView.selected && (
            <ProviderHero
              canScan={providersView.canScanSelected}
              connectionResult={providersView.connectionResult}
              deleteError={providersView.deleteError}
              deleting={providersView.deleting}
              diagnosticsError={providersView.diagnosticsError}
              diagnosticsPending={providersView.diagnosticsPending}
              diagnosticsResult={providersView.diagnosticsResult}
              getErrorMessage={providersView.getErrorMessage}
              modelsFetching={providersView.isModelsFetching}
              scanError={providersView.scanError}
              scanPending={providersView.isScanningCurrent}
              scanResult={providersView.scanResult}
              selected={providersView.selected}
              selectedModelCount={providersView.selectedModelCount}
              testError={providersView.testError}
              testPending={providersView.testPending}
              onDelete={providersView.onDelete}
              onDiagnose={providersView.onDiagnose}
              onEdit={() =>
                providersView.selected &&
                providersView.openEditProvider(providersView.selected)
              }
              onScan={providersView.onScan}
              onTestConnection={providersView.onTestConnection}
            />
          )}

          {providersView.selected && (
            <ProviderPanels
              canScan={providersView.canScanSelected}
              models={providersView.models}
              selectedModelCount={providersView.selectedModelCount}
              stats={providersView.capabilityStats}
            />
          )}
        </section>
      </div>

      <Dialog
        description={
          providersView.providerDrawerMode === "edit"
            ? "修改已有服务商配置。API Key 会回填当前保存值；清空后保存表示移除密钥。"
            : "填写服务名称、接口类型、Base URL 和 API Key。保存后可继续测试连接和扫描模型。"
        }
        open={providersView.showCreatePanel}
        title={
          providersView.providerDrawerMode === "edit"
            ? "编辑模型服务商"
            : "新增模型服务商"
        }
        variant="drawer"
        width="560px"
        onClose={() => {
          if (!providersView.createPending && !providersView.editPending) {
            providersView.closeProviderDrawer();
          }
        }}
      >
        <ProviderOnboarding
          form={providersView.form}
          mode={providersView.providerDrawerMode}
          saving={providersView.createPending || providersView.editPending}
          setForm={providersView.setForm}
          onCancel={providersView.closeProviderDrawer}
          onSubmit={providersView.submit}
        />
      </Dialog>
    </div>
  );
}
