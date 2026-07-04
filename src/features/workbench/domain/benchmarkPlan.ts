import type { BenchmarkTaskSummary } from "../../../types/api";

export function isRunning(status?: string) {
  return status === "running" || status === "stopping";
}

export function getStartBlockReason({
  activeTask,
  datasetsCount,
  form,
  modelsCount,
  providersCount,
}: {
  activeTask: BenchmarkTaskSummary | null;
  datasetsCount: number;
  form: {
    provider_id: string;
    model_id: string;
    dataset_id: string;
  };
  modelsCount: number;
  providersCount: number;
}) {
  if (isRunning(activeTask?.status)) {
    return "当前已有压测任务运行中，请先停止或等待任务完成。";
  }
  if (!providersCount || !form.provider_id) {
    return "请先在模型服务商页面新增服务商。";
  }
  if (!modelsCount || !form.model_id) {
    return "当前服务商还没有可用模型，请先在模型服务商页面完成连接测试并扫描模型。";
  }
  if (!datasetsCount || !form.dataset_id) {
    return "请先选择一个测试数据集。";
  }
  return null;
}

export function getErrorMessage(error: unknown) {
  if (error instanceof Error) return error.message;
  return typeof error === "string" ? error : "未知错误，请查看运行日志。";
}

export function normalizeStopResult(
  result: unknown,
  requestedTaskId: string,
): { taskId: string; stopped: boolean } {
  const payload = result as Partial<{
    task_id: string;
    taskId: string;
    stopped: boolean;
  }>;
  return {
    taskId: payload.task_id ?? payload.taskId ?? requestedTaskId,
    stopped: payload.stopped ?? true,
  };
}

export function buildFallbackPlan({
  endConcurrency,
  mode,
  stageSampleRounds,
  startConcurrency,
  stepStrategy,
  stepValue,
  warmupRounds,
}: {
  endConcurrency: number;
  mode: string;
  stageSampleRounds: number;
  startConcurrency: number;
  stepStrategy: string;
  stepValue: number;
  warmupRounds: number;
}) {
  const isStaircase = mode === "阶梯加压";
  const stages = isStaircase
    ? buildStageSequence(startConcurrency, endConcurrency, stepStrategy, stepValue)
    : [Math.max(1, Number(endConcurrency) || 1)];

  return {
    stages,
    stageSampleRounds: Math.max(1, Number(stageSampleRounds) || 4),
    warmupRounds: Math.max(0, Number(warmupRounds) || 0),
  };
}

export function buildStageSequence(
  startValue: number,
  endValue: number,
  strategy: string,
  stepValue: number,
) {
  const start = Math.max(1, Number(startValue) || 1);
  const end = Math.max(start, Number(endValue) || start);
  const step = Math.max(1, Number(stepValue) || 1);
  const stages: number[] = [];
  let current = start;

  while (current <= end && stages.length < 16) {
    stages.push(current);
    current = strategy === "linear" ? current + step : current * Math.max(2, step);
  }

  if (!stages.includes(end)) {
    stages.push(end);
  }

  return stages;
}
