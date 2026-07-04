import type { Dispatch, FormEvent, SetStateAction } from "react";
import { useCallback } from "react";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "../../../api/client";
import { queryKeys } from "../../../api/queryKeys";
import type { WorkbenchState } from "../../../stores/workbenchStore";
import type { BenchmarkTaskSummary, StageChangedEvent } from "../../../types/api";
import {
  getErrorMessage,
  isRunning,
  normalizeStopResult,
} from "../domain/benchmarkPlan";
import { buildBenchmarkStartInput } from "../domain/startInput";
import type { StartNotice, WorkbenchForm } from "../types";

type UseBenchmarkActionsInput = {
  activeTask: BenchmarkTaskSummary | null;
  estimatedSeconds: number;
  form: WorkbenchForm;
  isStaircase: boolean;
  selectedModelType: string;
  setCurrentStage: Dispatch<SetStateAction<StageChangedEvent | null>>;
  setStartNotice: (notice: StartNotice | null) => void;
  startBlockReason: string | null;
  store: Pick<
    WorkbenchState,
    | "addLog"
    | "setActiveTask"
    | "setGeneratedReport"
    | "updateActiveTask"
  >;
};

export function useBenchmarkActions({
  activeTask,
  estimatedSeconds,
  form,
  isStaircase,
  selectedModelType,
  setCurrentStage,
  setStartNotice,
  startBlockReason,
  store,
}: UseBenchmarkActionsInput) {
  const queryClient = useQueryClient();

  const markTaskStopped = useCallback(
    (taskId: string, title: string) => {
      const task = activeTask;
      if (task && (!taskId || task.id === taskId)) {
        store.updateActiveTask({ ...task, status: "cancelled" });
      }
      setCurrentStage((current) => ({
        task_id: taskId || task?.id || "",
        stage: "task_stopped",
        message: "压测任务已停止，实时指标已冻结。",
        stage_index: current?.stage_index ?? null,
        stage_total: current?.stage_total ?? null,
        concurrency: current?.concurrency ?? null,
      }));
      setStartNotice({
        tone: "success",
        title,
        message: "任务已取消，当前图表保留停止前最后一次指标结果。",
      });
    },
    [activeTask, setCurrentStage, setStartNotice, store],
  );

  const startMutation = useMutation({
    mutationFn: api.startBenchmark,
    onSuccess: (task) => {
      setStartNotice({
        tone: "success",
        title: "压测任务已启动",
        message: "正在等待第一批实时指标，通常 1 秒内会刷新图表。",
      });
      store.setActiveTask(task);
      store.addLog("后端已启动压测任务");
    },
    onError: (error) => {
      setStartNotice({
        tone: "danger",
        title: "压测启动失败",
        message: getErrorMessage(error),
      });
      store.addLog(`压测启动失败：${getErrorMessage(error)}`);
    },
  });

  const stopMutation = useMutation({
    mutationFn: api.stopBenchmark,
    onMutate: (taskId) => {
      const task = activeTask;
      if (task?.id === taskId) {
        store.updateActiveTask({ ...task, status: "stopping" });
      }
      setStartNotice({
        tone: "info",
        title: "正在停止压测",
        message: "停止请求已发出，正在等待后端确认并释放任务资源。",
      });
      store.addLog("正在停止压测任务");
    },
    onSuccess: (result, requestedTaskId) => {
      const normalized = normalizeStopResult(result, requestedTaskId);
      if (normalized.stopped) {
        markTaskStopped(normalized.taskId, "停止请求已确认");
        store.addLog("停止请求已确认");
      } else {
        markTaskStopped(normalized.taskId, "停止状态已同步");
        setStartNotice({
          tone: "success",
          title: "未找到运行中的任务",
          message: "后端没有返回可停止的任务，已将前端状态从停止中恢复为已取消。",
        });
        store.addLog("停止请求未命中运行中的任务");
      }
      void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard() });
    },
    onError: (error) => {
      const task = activeTask;
      if (task?.status === "stopping") {
        store.updateActiveTask({ ...task, status: "running" });
      }
      setStartNotice({
        tone: "danger",
        title: "停止失败",
        message: getErrorMessage(error),
      });
      store.addLog(`停止失败：${getErrorMessage(error)}`);
    },
  });

  const reportMutation = useMutation({
    mutationFn: api.generateReport,
    onSuccess: (report) => {
      store.setGeneratedReport(report);
      void queryClient.invalidateQueries({ queryKey: queryKeys.reports() });
      void queryClient.invalidateQueries({ queryKey: queryKeys.dashboard() });
    },
  });

  const submit = (event: FormEvent) => {
    event.preventDefault();
    if (startBlockReason) {
      setStartNotice({
        tone: "danger",
        title: "还不能开始压测",
        message: startBlockReason,
      });
      store.addLog(`配置未完成：${startBlockReason}`);
      return;
    }

    const startInput = buildBenchmarkStartInput({
      estimatedSeconds,
      form,
      isStaircase,
      selectedModelType,
    });
    if (!startInput.ok) {
      setStartNotice({
        tone: "danger",
        title: "压测配置无效",
        message: startInput.message,
      });
      store.addLog(`配置无效：${startInput.message}`);
      return;
    }

    setCurrentStage(null);
    setStartNotice({
      tone: "info",
      title: "正在启动压测任务",
      message: "配置已提交，正在创建任务和建立实时指标通道。",
    });
    store.addLog("正在提交压测任务配置");
    startMutation.mutate(startInput.input);
  };

  return {
    canSubmitStart: !isRunning(activeTask?.status) && !startMutation.isPending,
    canGenerateReport: Boolean(activeTask?.id) && activeTask?.status === "completed",
    canStop:
      Boolean(activeTask?.id) &&
      activeTask?.status === "running" &&
      !stopMutation.isPending,
    markTaskStopped,
    reportMutation,
    startMutation,
    stopMutation,
    submit,
  };
}
