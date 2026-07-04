import { useEffect, useRef } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import type { WorkbenchState } from "../../../stores/workbenchStore";
import type { BenchmarkTaskSummary } from "../../../types/api";
import { isRunning } from "../domain/benchmarkPlan";
import { debugRealtime, warnRealtime } from "../domain/realtimeDebug";

const POLLING_INTERVAL_MS = 2000;

type UseBenchmarkPollingInput = {
  activeTask: BenchmarkTaskSummary | null;
  addLog: WorkbenchState["addLog"];
  enabled: boolean;
  mergeTicks: WorkbenchState["mergeTicks"];
  updateActiveTask: WorkbenchState["updateActiveTask"];
};

export function useBenchmarkPolling({
  activeTask,
  addLog,
  enabled,
  mergeTicks,
  updateActiveTask,
}: UseBenchmarkPollingInput) {
  const queryClient = useQueryClient();
  const loggedFirstTickForTaskRef = useRef<string | null>(null);
  const loggedErrorForTaskRef = useRef<string | null>(null);
  const taskId = activeTask?.id ?? null;
  const taskStatus = activeTask?.status ?? null;

  useEffect(() => {
    if (!enabled || !taskId || !isRunning(taskStatus ?? undefined)) return;

    let cancelled = false;

    debugRealtime("polling", "启动状态同步兜底", {
      taskId,
      status: taskStatus,
      intervalMs: POLLING_INTERVAL_MS,
    });

    async function poll() {
      if (!taskId || cancelled) return;

      try {
        const [ticks, task] = await Promise.all([
          api.listBenchmarkTicks(taskId),
          api.getBenchmarkTask(taskId),
        ]);

        if (cancelled) return;

        debugRealtime("polling", "兜底同步返回任务状态和指标", {
          taskId,
          taskStatus: task.status,
          tickCount: ticks.length,
          latestElapsedSeconds: ticks.at(-1)?.elapsed_seconds ?? null,
        });

        if (ticks.length > 0) {
          mergeTicks(ticks);
          if (loggedFirstTickForTaskRef.current !== taskId) {
            addLog("已通过状态同步兜底获取实时指标");
            loggedFirstTickForTaskRef.current = taskId;
          }
        }

        if (task.status !== taskStatus) {
          updateActiveTask(task);
          if (task.status === "completed") {
            addLog("压测任务已完成，可以生成报告");
            void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard() });
          }
        }
      } catch (error) {
        const message = getErrorMessage(error);
        warnRealtime("polling", "状态同步兜底失败", {
          taskId,
          error: message,
        });
        if (loggedErrorForTaskRef.current === taskId) return;
        loggedErrorForTaskRef.current = taskId;
        addLog(`实时指标状态同步失败：${message}`);
      }
    }

    void poll();
    const timer = window.setInterval(() => {
      void poll();
    }, POLLING_INTERVAL_MS);

    return () => {
      cancelled = true;
      debugRealtime("polling", "停止状态同步兜底", { taskId });
      window.clearInterval(timer);
    };
  }, [
    addLog,
    enabled,
    mergeTicks,
    queryClient,
    taskId,
    taskStatus,
    updateActiveTask,
  ]);
}

function getErrorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "未知错误";
}
