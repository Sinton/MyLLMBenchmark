import { create } from "zustand";
import type { BenchmarkTaskSummary, MetricsTick, ReportSummary } from "../types/api";

export type WorkbenchState = {
  activeTask: BenchmarkTaskSummary | null;
  latestTick: MetricsTick | null;
  ticks: MetricsTick[];
  logs: string[];
  generatedReport: ReportSummary | null;
  setActiveTask: (task: BenchmarkTaskSummary | null) => void;
  updateActiveTask: (task: BenchmarkTaskSummary) => void;
  addTick: (tick: MetricsTick) => void;
  mergeTicks: (ticks: MetricsTick[]) => void;
  hydrateTask: (task: BenchmarkTaskSummary, ticks: MetricsTick[], message: string) => void;
  addLog: (message: string) => void;
  setGeneratedReport: (report: ReportSummary | null) => void;
  resetRun: () => void;
};

export const useWorkbenchStore = create<WorkbenchState>((set) => ({
  activeTask: null,
  latestTick: null,
  ticks: [],
  logs: [],
  generatedReport: null,
  setActiveTask: (task) =>
    set(() => ({
      activeTask: task,
      latestTick: null,
      ticks: [],
      logs: task ? [`${formatTime()} 任务已创建：${task.name}`] : [],
      generatedReport: null,
    })),
  updateActiveTask: (task) => set(() => ({ activeTask: task })),
  addTick: (tick) =>
    set((state) => {
      const byKey = new Map<string, MetricsTick>();
      for (const item of state.ticks) {
        byKey.set(`${item.task_id}:${item.elapsed_seconds}`, item);
      }
      byKey.set(`${tick.task_id}:${tick.elapsed_seconds}`, tick);
      const ticks = Array.from(byKey.values())
        .sort((a, b) => a.elapsed_seconds - b.elapsed_seconds)
        .slice(-80);
      return {
        latestTick: ticks.at(-1) ?? tick,
        ticks,
      };
    }),
  mergeTicks: (incomingTicks) =>
    set((state) => {
      if (!incomingTicks.length) return state;
      const byKey = new Map<string, MetricsTick>();
      for (const tick of state.ticks) {
        byKey.set(`${tick.task_id}:${tick.elapsed_seconds}`, tick);
      }
      for (const tick of incomingTicks) {
        byKey.set(`${tick.task_id}:${tick.elapsed_seconds}`, tick);
      }
      const ticks = Array.from(byKey.values())
        .sort((a, b) => a.elapsed_seconds - b.elapsed_seconds)
        .slice(-80);
      return {
        latestTick: ticks.at(-1) ?? state.latestTick,
        ticks,
      };
    }),
  hydrateTask: (task, incomingTicks, message) =>
    set(() => {
      const ticks = [...incomingTicks].sort(
        (a, b) => a.elapsed_seconds - b.elapsed_seconds,
      );
      return {
        activeTask: task,
        latestTick: ticks.at(-1) ?? null,
        ticks,
        logs: [`${formatTime()} ${message}`],
        generatedReport: null,
      };
    }),
  addLog: (message) =>
    set((state) => ({
      logs: [...state.logs.slice(-80), `${formatTime()} ${message}`],
    })),
  setGeneratedReport: (report) => set(() => ({ generatedReport: report })),
  resetRun: () =>
    set(() => ({
      activeTask: null,
      latestTick: null,
      ticks: [],
      logs: [],
      generatedReport: null,
    })),
}));

function formatTime() {
  return new Date().toLocaleTimeString("zh-CN", { hour12: false });
}
