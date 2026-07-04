import { useEffect, useRef } from "react";
import type { QueryClient } from "@tanstack/react-query";
import { listenToEvent } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import type {
  BenchmarkTaskSummary,
  MetricsTick,
  ReportSummary,
  StageChangedEvent,
} from "../../../types/api";
import { debugRealtime, warnRealtime } from "../domain/realtimeDebug";

type UseBenchmarkEventsInput = {
  addLog: (message: string) => void;
  addTick: (tick: MetricsTick) => void;
  markTaskStopped: (taskId: string, title: string) => void;
  onMetricsTick?: (tick: MetricsTick) => void;
  queryClient: QueryClient;
  setCurrentStage: (stage: StageChangedEvent | null) => void;
  setGeneratedReport: (report: ReportSummary | null) => void;
  updateActiveTask: (task: BenchmarkTaskSummary) => void;
};

export function useBenchmarkEvents({
  addLog,
  addTick,
  markTaskStopped,
  onMetricsTick,
  queryClient,
  setCurrentStage,
  setGeneratedReport,
  updateActiveTask,
}: UseBenchmarkEventsInput) {
  const handlersRef = useRef({
    addLog,
    addTick,
    markTaskStopped,
    onMetricsTick,
    setCurrentStage,
    setGeneratedReport,
    updateActiveTask,
  });
  const hasReceivedTickRef = useRef(false);

  handlersRef.current = {
    addLog,
    addTick,
    markTaskStopped,
    onMetricsTick,
    setCurrentStage,
    setGeneratedReport,
    updateActiveTask,
  };

  useEffect(() => {
    const unsubs: Array<() => void> = [];
    let disposed = false;

    async function attach() {
      debugRealtime("event", "开始订阅 Tauri 实时事件", {
        events: [
          "benchmark:metrics_tick",
          "benchmark:stage_changed",
          "benchmark:task_completed",
          "benchmark:task_stopped",
          "benchmark:report_ready",
        ],
      });

      try {
        unsubs.push(
          await listenToEvent<MetricsTick>("benchmark:metrics_tick", (payload) => {
            const handlers = handlersRef.current;
            debugRealtime("event", "收到 benchmark:metrics_tick", {
              taskId: payload.task_id,
              elapsedSeconds: payload.elapsed_seconds,
              qps: payload.qps,
              latencyMs: payload.latency_ms,
              successRate: payload.success_rate,
            });

            handlers.onMetricsTick?.(payload);
            if (!hasReceivedTickRef.current) {
              handlers.addLog("已通过事件推送收到首批实时指标");
              hasReceivedTickRef.current = true;
            }
            handlers.addTick(payload);
          }),
        );
        unsubs.push(
          await listenToEvent<StageChangedEvent>("benchmark:stage_changed", (payload) => {
            const handlers = handlersRef.current;
            debugRealtime("event", "收到 benchmark:stage_changed", {
              taskId: payload.task_id,
              stageIndex: payload.stage_index,
              concurrency: payload.concurrency,
              stage: payload.stage,
            });
            handlers.setCurrentStage(payload);
            handlers.addLog(payload.message);
          }),
        );
        unsubs.push(
          await listenToEvent<BenchmarkTaskSummary>(
            "benchmark:task_completed",
            (payload) => {
              const handlers = handlersRef.current;
              debugRealtime("event", "收到 benchmark:task_completed", {
                taskId: payload.id,
                status: payload.status,
              });
              handlers.updateActiveTask(payload);
              handlers.addLog("压测任务已完成，可以生成报告");
              void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard() });
            },
          ),
        );
        unsubs.push(
          await listenToEvent<string>("benchmark:task_stopped", (taskId) => {
            const handlers = handlersRef.current;
            debugRealtime("event", "收到 benchmark:task_stopped", { taskId });
            handlers.markTaskStopped(taskId, "压测已停止");
            handlers.addLog("压测任务已停止");
            void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard() });
          }),
        );
        unsubs.push(
          await listenToEvent<ReportSummary>("benchmark:report_ready", (payload) => {
            const handlers = handlersRef.current;
            debugRealtime("event", "收到 benchmark:report_ready", {
              reportId: payload.id,
              taskId: payload.task_id,
            });
            handlers.setGeneratedReport(payload);
            handlers.addLog("测试报告已生成");
            void queryClient.invalidateQueries({ queryKey: queryKeys.reports() });
            void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard() });
          }),
        );
        debugRealtime("event", "Tauri 实时事件订阅成功");
      } catch (error) {
        const message = getErrorMessage(error);
        warnRealtime("event", "Tauri 实时事件订阅失败", { error: message });
        handlersRef.current.addLog(`实时指标通道订阅失败：${message}`);
      }

      if (disposed) {
        unsubs.forEach((unsub) => unsub());
      }
    }

    void attach();
    return () => {
      disposed = true;
      hasReceivedTickRef.current = false;
      debugRealtime("event", "清理 Tauri 实时事件订阅");
      unsubs.forEach((unsub) => unsub());
    };
  }, [queryClient]);
}

function getErrorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "未知错误";
}
