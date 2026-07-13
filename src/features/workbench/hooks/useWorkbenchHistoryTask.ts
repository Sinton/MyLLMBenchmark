import { useEffect, useRef } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import type { WorkbenchState } from "../../../stores/workbenchStore";
import type { StartNotice } from "../types";

type UseWorkbenchHistoryTaskInput = {
  taskId: string | null;
  hydrateTask: WorkbenchState["hydrateTask"];
  resetRun: WorkbenchState["resetRun"];
  setCurrentStage: (stage: null) => void;
  setStartNotice: (notice: StartNotice | null) => void;
};

export function useWorkbenchHistoryTask({
  taskId,
  hydrateTask,
  resetRun,
  setCurrentStage,
  setStartNotice,
}: UseWorkbenchHistoryTaskInput) {
  const normalizedTaskId = taskId?.trim() ?? "";
  const hydratedKeyRef = useRef<string | null>(null);
  const errorKeyRef = useRef<string | null>(null);

  const taskQuery = useQuery({
    queryKey: queryKeys.benchmarkTask(normalizedTaskId),
    queryFn: () => api.getBenchmarkTask(normalizedTaskId),
    enabled: Boolean(normalizedTaskId),
    retry: false,
  });

  const ticksQuery = useQuery({
    queryKey: queryKeys.benchmarkTicks(normalizedTaskId),
    queryFn: () => api.listBenchmarkTicks(normalizedTaskId),
    enabled: Boolean(normalizedTaskId),
    retry: false,
  });

  useEffect(() => {
    if (!normalizedTaskId) return;
    resetRun();
    setCurrentStage(null);
    setStartNotice({
      tone: "info",
      title: "正在加载历史任务",
      message: "正在读取任务摘要和持久化指标。",
    });
  }, [normalizedTaskId, resetRun, setCurrentStage, setStartNotice]);

  useEffect(() => {
    if (!normalizedTaskId) {
      hydratedKeyRef.current = null;
      errorKeyRef.current = null;
      return;
    }

    const error = taskQuery.error ?? ticksQuery.error;
    if (error) {
      if (errorKeyRef.current === normalizedTaskId) return;
      errorKeyRef.current = normalizedTaskId;
      setStartNotice({
        tone: "danger",
        title: "历史任务加载失败",
        message: getErrorMessage(error),
      });
      return;
    }

    if (!taskQuery.data || !ticksQuery.data) return;

    const task = taskQuery.data;
    const ticks = ticksQuery.data;
    const latestElapsed = ticks.at(-1)?.elapsed_seconds ?? "none";
    const hydrateKey = `${task.id}:${task.status}:${ticks.length}:${latestElapsed}`;
    if (hydratedKeyRef.current === hydrateKey) return;
    hydratedKeyRef.current = hydrateKey;
    errorKeyRef.current = null;

    const message = ticks.length
      ? `已加载历史任务：${task.name}，共 ${ticks.length} 条指标。`
      : `已加载历史任务：${task.name}，但没有可回放指标。`;

    hydrateTask(task, ticks, message);
    setCurrentStage(null);
    setStartNotice({
      tone: ticks.length ? "success" : "info",
      title: ticks.length ? "历史任务已加载" : "历史任务没有指标",
      message: ticks.length
        ? "正在展示后端持久化的压测指标；事件日志只包含当前会话信息。"
        : "这个任务没有持久化 tick，可能是任务初始化失败、旧版本任务，或 Mock 内存数据已丢失。",
    });
  }, [
    hydrateTask,
    normalizedTaskId,
    setCurrentStage,
    setStartNotice,
    taskQuery.data,
    taskQuery.error,
    ticksQuery.data,
    ticksQuery.error,
  ]);

  return {
    historyError: taskQuery.error ?? ticksQuery.error ?? null,
    historyLoading:
      Boolean(normalizedTaskId) && (taskQuery.isLoading || ticksQuery.isLoading),
    historyTaskId: normalizedTaskId || null,
    isHistoryView: Boolean(normalizedTaskId),
  };
}

function getErrorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  return typeof error === "string" ? error : "任务不存在或当前数据源不可用。";
}
