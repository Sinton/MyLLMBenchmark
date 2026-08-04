import { Button } from "../../components/ui/Button";
import { Card } from "../../components/ui/Card";
import { Download, Upload } from "../../components/ui/icons";
import { WorkspaceHeader } from "../../components/app-shell/WorkspaceHeader";
import { useNotification } from "../../components/ui/Notification";
import { DatasetDeleteDialog } from "../../features/datasets/components/DatasetDeleteDialog";
import { DatasetDetailDrawer } from "../../features/datasets/components/DatasetDetailDrawer";
import { DatasetImportDialog } from "../../features/datasets/components/DatasetImportDialog";
import { DatasetTable } from "../../features/datasets/components/DatasetTable";
import { DatasetTypeTabs } from "../../features/datasets/components/DatasetTypeTabs";
import { useDatasetsController } from "../../features/datasets/hooks/useDatasetsController";

export function Datasets() {
  const { notify } = useNotification();
  const controller = useDatasetsController();

  return (
    <div className="page">
      <WorkspaceHeader
        breadcrumb="样本管理"
        title="测试数据集"
        subtitle="查看、编辑和审计压测样本，让容量结论建立在可解释的业务 Prompt 上。"
        actions={
          <>
            <Button
              icon={<Download size={16} />}
              onClick={() =>
                notify({
                  title: "模板准备中",
                  description: "当前优先支持 JSONL、CSV、TXT 和 Excel 导入，模板导出会在后续补齐。",
                  tone: "success",
                })
              }
            >
              下载模板
            </Button>
            <Button
              icon={<Upload size={16} />}
              onClick={() => controller.setImportOpen(true)}
              variant="primary"
            >
              导入数据集
            </Button>
          </>
        }
      />

      <div className="dataset-layout">
        <DatasetTypeTabs
          value={controller.activeType}
          onChange={controller.setActiveType}
        />

        <Card title="数据集列表" eyebrow="样本目录">
          <DatasetTable
            datasets={controller.filtered}
            onDelete={controller.setDeleteTarget}
            onEdit={(dataset) => controller.openDetail(dataset, "edit")}
            onView={(dataset) => controller.openDetail(dataset, "view")}
          />
        </Card>
      </div>

      <DatasetImportDialog
        open={controller.importOpen}
        onClose={() => controller.setImportOpen(false)}
        onSubmit={controller.importDataset}
        submitting={controller.importPending}
      />

      <DatasetDetailDrawer
        appendSamples={controller.appendSamples}
        appendSamplesPending={controller.appendSamplesPending}
        batchDeleteSamples={controller.batchDeleteSamples}
        batchDeletePending={controller.batchDeletePending}
        createSample={controller.createSample}
        createSamplePending={controller.createSamplePending}
        dataset={controller.selectedDataset}
        deleteSample={controller.deleteSample}
        deleteSamplePending={controller.deleteSamplePending}
        editing={controller.detailMode === "edit"}
        onClose={controller.closeDetail}
        onEditingChange={(editing) =>
          controller.setDetailMode(editing ? "edit" : "view")
        }
        onPageChange={controller.setSamplePage}
        onPageSizeChange={controller.setSamplePageSize}
        onSearchChange={controller.setSampleKeyword}
        onExportDataset={controller.exportDataset}
        onUpdateDataset={controller.updateDataset}
        onValidateDataset={controller.validateDataset}
        sampleKeyword={controller.sampleKeyword}
        samplePage={controller.samplePageData}
        samplesFetching={controller.samplesFetching}
        samplesLoading={controller.samplesLoading}
        updateDatasetPending={controller.updateDatasetPending}
        updateSample={controller.updateSample}
        updateSamplePending={controller.updateSamplePending}
        exportDatasetPending={controller.exportDatasetPending}
        validateDatasetPending={controller.validateDatasetPending}
        validationResult={controller.validationResult}
      />

      <DatasetDeleteDialog
        dataset={controller.deleteTarget}
        deleting={controller.deleteDatasetPending}
        onClose={() => controller.setDeleteTarget(null)}
        onConfirm={controller.deleteDataset}
      />
    </div>
  );
}
