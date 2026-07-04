import { useMemo, useState } from "react";
import {
  buildStageSequence,
  getStartBlockReason,
} from "../domain/benchmarkPlan";
import {
  getChartTabs,
  getLiveMetricCards,
} from "../domain/metricDefinitions";
import { useBenchmarkRunController } from "./useBenchmarkRunController";
import { useWorkbenchData } from "./useWorkbenchData";
import { useWorkbenchFormSync } from "./useWorkbenchFormSync";
import {
  defaultWorkbenchForm,
  type ChartMetric,
  type StartNotice,
} from "../types";
import { useWorkbenchStore } from "../../../stores/workbenchStore";
import type { StageChangedEvent } from "../../../types/api";

export function useWorkbenchController() {
  const [chartMetric, setChartMetric] = useState<ChartMetric>("latency");
  const [form, setForm] = useState(defaultWorkbenchForm);
  const [currentStage, setCurrentStage] = useState<StageChangedEvent | null>(null);
  const [startNotice, setStartNotice] = useState<StartNotice | null>(null);

  const {
    activeTask,
    latestTick,
    ticks,
    logs,
    generatedReport,
    setActiveTask,
    updateActiveTask,
    addTick,
    mergeTicks,
    addLog,
    setGeneratedReport,
  } = useWorkbenchStore();

  const data = useWorkbenchData(form, activeTask);
  const startBlockReason = getStartBlockReason({
    activeTask,
    datasetsCount: data.datasets.length,
    form,
    modelsCount: data.providerModels.length,
    providersCount: data.providers.length,
  });
  const isStaircase = form.mode === "阶梯加压";
  const stageSequence = useMemo(
    () =>
      buildStageSequence(
        form.start_concurrency,
        form.end_concurrency,
        form.step_strategy,
        form.step_value,
      ),
    [
      form.end_concurrency,
      form.start_concurrency,
      form.step_strategy,
      form.step_value,
    ],
  );
  const estimatedSeconds = isStaircase
    ? stageSequence.length *
      (Number(form.stage_sample_rounds) + Number(form.warmup_rounds))
    : Number(form.duration_seconds);

  const liveMetricCards = getLiveMetricCards(data.activeModelType, latestTick);
  const liveChartTabs = getChartTabs(data.activeModelType);
  const effectiveChartMetric = liveChartTabs.some((tab) => tab.key === chartMetric)
    ? chartMetric
    : liveChartTabs[0].key;

  const runController = useBenchmarkRunController({
    activeTask,
    addLog,
    addTick,
    mergeTicks,
    estimatedSeconds,
    form,
    isStaircase,
    selectedModelType: data.selectedModelType,
    setActiveTask,
    setCurrentStage,
    setGeneratedReport,
    setStartNotice,
    startBlockReason,
    updateActiveTask,
  });

  useWorkbenchFormSync({
    datasets: data.datasets,
    form,
    providerModels: data.providerModels,
    providers: data.providers,
    selectedModelType: data.selectedModelType,
    setForm,
  });

  return {
    activeTask,
    canGenerateReport: runController.canGenerateReport,
    canStop: runController.canStop,
    canSubmitStart: runController.canSubmitStart,
    chartMetric: effectiveChartMetric,
    currentStage,
    datasets: data.datasets,
    estimatedSeconds,
    form,
    generatedReport,
    isStaircase,
    latestTick,
    liveChartTabs,
    liveMetricCards,
    logs,
    onChartMetricChange: setChartMetric,
    onGenerateReport: () =>
      activeTask?.id && runController.reportMutation.mutate(activeTask.id),
    onStop: () => activeTask?.id && runController.stopMutation.mutate(activeTask.id),
    onSubmit: runController.submit,
    providerModels: data.providerModels,
    providerDiagnostics: data.providerDiagnostics,
    providerDiagnosticsFetching: data.providerDiagnosticsFetching,
    providers: data.providers,
    reportPending: runController.reportMutation.isPending,
    selectedDataset: data.selectedDataset,
    selectedModel: data.selectedModel,
    selectedModelType: data.selectedModelType,
    selectedProvider: data.selectedProvider,
    setForm,
    stageSequence,
    startBlockReason,
    startNotice,
    startPending: runController.startMutation.isPending,
    stopPending: runController.stopMutation.isPending,
    ticks,
  };
}
