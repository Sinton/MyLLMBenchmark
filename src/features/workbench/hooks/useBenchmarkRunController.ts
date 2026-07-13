import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type Dispatch,
  type SetStateAction,
} from "react";
import { useQueryClient } from "@tanstack/react-query";
import { useBenchmarkActions } from "./useBenchmarkActions";
import { useBenchmarkEvents } from "./useBenchmarkEvents";
import { useBenchmarkPolling } from "./useBenchmarkPolling";
import type { WorkbenchState } from "../../../stores/workbenchStore";
import type {
  BenchmarkTaskSummary,
  MetricsTick,
  ReportSummary,
  StageChangedEvent,
} from "../../../types/api";
import { isRunning } from "../domain/benchmarkPlan";
import { debugRealtime } from "../domain/realtimeDebug";
import type { StartNotice, WorkbenchForm } from "../types";

const EVENT_FALLBACK_TIMEOUT_MS = 2500;

type UseBenchmarkRunControllerInput = {
  activeTask: BenchmarkTaskSummary | null;
  addLog: WorkbenchState["addLog"];
  addTick: WorkbenchState["addTick"];
  mergeTicks: WorkbenchState["mergeTicks"];
  estimatedSeconds: number;
  form: WorkbenchForm;
  isStaircase: boolean;
  selectedModelType: string;
  setActiveTask: WorkbenchState["setActiveTask"];
  setCurrentStage: Dispatch<SetStateAction<StageChangedEvent | null>>;
  setGeneratedReport: (report: ReportSummary | null) => void;
  setStartNotice: (notice: StartNotice | null) => void;
  startBlockReason: string | null;
  updateActiveTask: WorkbenchState["updateActiveTask"];
};

export function useBenchmarkRunController({
  activeTask,
  addLog,
  addTick,
  mergeTicks,
  estimatedSeconds,
  form,
  isStaircase,
  selectedModelType,
  setActiveTask,
  setCurrentStage,
  setGeneratedReport,
  setStartNotice,
  startBlockReason,
  updateActiveTask,
}: UseBenchmarkRunControllerInput) {
  const queryClient = useQueryClient();
  const [fallbackPollingEnabled, setFallbackPollingEnabled] = useState(false);
  const eventTickTaskRef = useRef<string | null>(null);
  const fallbackLogTaskRef = useRef<string | null>(null);
  const activeTaskId = activeTask?.id ?? null;
  const activeTaskStatus = activeTask?.status ?? null;
  const actionStore = useMemo(
    () => ({
      addLog,
      setActiveTask,
      setGeneratedReport,
      updateActiveTask,
    }),
    [addLog, setActiveTask, setGeneratedReport, updateActiveTask],
  );

  const actions = useBenchmarkActions({
    activeTask,
    estimatedSeconds,
    form,
    isStaircase,
    selectedModelType,
    setCurrentStage,
    setStartNotice,
    startBlockReason,
    store: actionStore,
  });

  const handleMetricsTick = useCallback(
    (tick: MetricsTick) => {
      eventTickTaskRef.current = tick.task_id;

      if (tick.task_id !== activeTaskId) return;

      if (fallbackPollingEnabled) {
        debugRealtime("event", "已收到事件推送指标，关闭状态同步兜底", {
          taskId: tick.task_id,
          elapsedSeconds: tick.elapsed_seconds,
        });
        addLog("实时事件推送已恢复，已停止状态同步兜底");
        setFallbackPollingEnabled(false);
      }
    },
    [activeTaskId, addLog, fallbackPollingEnabled],
  );

  useEffect(() => {
    if (!activeTaskId || !isRunning(activeTaskStatus ?? undefined)) {
      setFallbackPollingEnabled(false);
      return;
    }

    eventTickTaskRef.current = null;
    fallbackLogTaskRef.current = null;
    setFallbackPollingEnabled(false);
    debugRealtime("event", "等待事件推送首个指标", {
      taskId: activeTaskId,
      timeoutMs: EVENT_FALLBACK_TIMEOUT_MS,
    });

    const timer = window.setTimeout(() => {
      if (eventTickTaskRef.current === activeTaskId) return;

      debugRealtime("polling", "事件推送超时，启用状态同步兜底", {
        taskId: activeTaskId,
        timeoutMs: EVENT_FALLBACK_TIMEOUT_MS,
      });
      if (fallbackLogTaskRef.current !== activeTaskId) {
        addLog("未在 2.5 秒内收到事件推送指标，已启用状态同步兜底");
        fallbackLogTaskRef.current = activeTaskId;
      }
      setFallbackPollingEnabled(true);
    }, EVENT_FALLBACK_TIMEOUT_MS);

    return () => {
      window.clearTimeout(timer);
    };
  }, [activeTaskId, activeTaskStatus, addLog]);

  useBenchmarkEvents({
    activeTaskId,
    addLog,
    addTick,
    enabled: Boolean(activeTaskId && isRunning(activeTaskStatus ?? undefined)),
    markTaskStopped: actions.markTaskStopped,
    onMetricsTick: handleMetricsTick,
    queryClient,
    setCurrentStage,
    setGeneratedReport,
    updateActiveTask,
  });

  useBenchmarkPolling({
    activeTask,
    addLog,
    enabled: fallbackPollingEnabled,
    mergeTicks,
    updateActiveTask,
  });

  return actions;
}
