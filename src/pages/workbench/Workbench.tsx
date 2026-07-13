import { BenchmarkConfigPanel } from "../../features/workbench/components/BenchmarkConfigPanel";
import { EventLogPanel } from "../../features/workbench/components/EventLogPanel";
import { RealtimeTrendPanel } from "../../features/workbench/components/RealtimeTrendPanel";
import { RuntimeStatsPanel } from "../../features/workbench/components/RuntimeStatsPanel";
import { TaskSummaryPanel } from "../../features/workbench/components/TaskSummaryPanel";
import { WorkbenchHeader } from "../../features/workbench/components/WorkbenchHeader";
import { useWorkbenchController } from "../../features/workbench/hooks/useWorkbenchController";

export function Workbench() {
  const workbench = useWorkbenchController();

  return (
    <div className="page workbench-page">
      <WorkbenchHeader
        activeTask={workbench.activeTask}
        canGenerateReport={workbench.canGenerateReport}
        canStop={workbench.canStop}
        onGenerateReport={workbench.onGenerateReport}
        onStop={workbench.onStop}
        reportPending={workbench.reportPending}
        stopPending={workbench.stopPending}
      />

      <div className="workbench-layout">
        <BenchmarkConfigPanel
          canSubmitStart={workbench.canSubmitStart}
          datasets={workbench.datasets}
          form={workbench.form}
          isStaircase={workbench.isStaircase}
          onSubmit={workbench.onSubmit}
          providerModels={workbench.providerModels}
          providerDiagnostics={workbench.providerDiagnostics}
          providerDiagnosticsFetching={workbench.providerDiagnosticsFetching}
          providers={workbench.providers}
          selectedModel={workbench.selectedModel}
          selectedModelType={workbench.selectedModelType}
          selectedProvider={workbench.selectedProvider}
          setForm={workbench.setForm}
          startBlockReason={workbench.startBlockReason}
          startNotice={workbench.startNotice}
          startPending={workbench.startPending}
        />

        <div className="workbench-center">
          <RealtimeTrendPanel
            activeTask={workbench.activeTask}
            chartMetric={workbench.chartMetric}
            historyError={workbench.historyError}
            historyLoading={workbench.historyLoading}
            isHistoryView={workbench.isHistoryView}
            latestTick={workbench.latestTick}
            metricCards={workbench.liveMetricCards}
            onChartMetricChange={workbench.onChartMetricChange}
            startPending={workbench.startPending}
            tabs={workbench.liveChartTabs}
            ticks={workbench.ticks}
          />
          <EventLogPanel logs={workbench.logs} />
        </div>

        <div className="workbench-side">
          <TaskSummaryPanel
            dataset={workbench.selectedDataset}
            estimatedSeconds={workbench.estimatedSeconds}
            form={workbench.form}
            historyTask={workbench.isHistoryView ? workbench.activeTask : null}
            isStaircase={workbench.isStaircase}
            model={workbench.selectedModel}
            modelType={workbench.selectedModelType}
            provider={workbench.selectedProvider}
            stageSequence={workbench.stageSequence}
          />
          <RuntimeStatsPanel
            activeTask={workbench.activeTask}
            currentStage={workbench.currentStage}
            generatedReport={workbench.generatedReport}
            latestTick={workbench.latestTick}
          />
        </div>
      </div>
    </div>
  );
}
