import { lazy, Suspense } from "react";
import { Navigate, Route, Routes } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api } from "../api/client";
import { queryKeys } from "../api/queryKeys";
import {
  DesktopShell,
  type DesktopNavItem,
} from "../components/app-shell/DesktopShell";
import {
  Activity,
  Building2,
  Database,
  FileText,
  LayoutDashboard,
  Network,
  Settings as SettingsIcon,
} from "../components/ui/icons";
import { LoadingBlock } from "../components/ui/LoadingBlock";
import { StatusBarItem } from "../components/app-shell/StatusBarItem";
import { useWorkbenchStore } from "../stores/workbenchStore";

const Dashboard = lazy(() =>
  import("../pages/dashboard/Dashboard").then((module) => ({
    default: module.Dashboard,
  })),
);
const Providers = lazy(() =>
  import("../pages/providers/Providers").then((module) => ({
    default: module.Providers,
  })),
);
const Datasets = lazy(() =>
  import("../pages/datasets/Datasets").then((module) => ({
    default: module.Datasets,
  })),
);
const Workbench = lazy(() =>
  import("../pages/workbench/Workbench").then((module) => ({
    default: module.Workbench,
  })),
);
const Reports = lazy(() =>
  import("../pages/reports/Reports").then((module) => ({ default: module.Reports })),
);
const SiteProbe = lazy(() =>
  import("../pages/site-probe/SiteProbe").then((module) => ({
    default: module.SiteProbe,
  })),
);
const SettingsRoute = lazy(() =>
  import("../pages/settings/Settings").then((module) => ({
    default: module.Settings,
  })),
);

const navItems: DesktopNavItem[] = [
  { to: "/dashboard", label: "启动中心", shortLabel: "中心", icon: LayoutDashboard },
  { to: "/providers", label: "模型服务商", shortLabel: "服务", icon: Building2 },
  { to: "/datasets", label: "测试数据集", shortLabel: "数据", icon: Database },
  { to: "/workbench", label: "压测工作台", shortLabel: "压测", icon: Activity },
  { to: "/reports", label: "测试报告", shortLabel: "报告", icon: FileText },
  { to: "/site-probe", label: "站点测活", shortLabel: "测活", icon: Network },
  { to: "/settings", label: "系统设置", shortLabel: "设置", icon: SettingsIcon },
];

export function App() {
  const activeTask = useWorkbenchStore((state) => state.activeTask);
  const latestTick = useWorkbenchStore((state) => state.latestTick);
  const configQuery = useQuery({
    queryKey: queryKeys.appConfig(),
    queryFn: api.getAppConfig,
    staleTime: 60_000,
  });

  return (
    <DesktopShell
      navItems={navItems}
      statusLeft={
        <>
          <StatusBarItem label="运行" tone="success" value="本地桌面" />
          <StatusBarItem
            label="数据"
            value={dataModeLabel(configQuery.data?.data_mode)}
          />
          <StatusBarItem
            label="引擎"
            tone={
              configQuery.data?.benchmark_engine === "openai_compatible"
                ? "success"
                : "neutral"
            }
            value={engineLabel(configQuery.data?.benchmark_engine)}
          />
        </>
      }
      statusRight={
        <>
          <StatusBarItem
            label="任务"
            tone={activeTask ? taskTone(activeTask.status) : "neutral"}
            value={activeTask ? taskLabel(activeTask.status) : "空闲"}
          />
          <StatusBarItem
            label="实时"
            tone={latestTick ? "success" : "neutral"}
            value={latestTick ? `第 ${latestTick.elapsed_seconds} 秒` : "待命"}
          />
        </>
      }
    >
      <Suspense fallback={<LoadingBlock text="正在打开工作区..." />}>
        <Routes>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/providers" element={<Providers />} />
          <Route path="/datasets" element={<Datasets />} />
          <Route path="/workbench" element={<Workbench />} />
          <Route path="/reports" element={<Reports />} />
          <Route path="/site-probe" element={<SiteProbe />} />
          <Route path="/settings" element={<SettingsRoute />} />
        </Routes>
      </Suspense>
    </DesktopShell>
  );
}

function dataModeLabel(value?: string) {
  if (value === "sqlite") return "SQLite";
  if (value === "mock") return "Mock";
  return "读取中";
}

function engineLabel(value?: string) {
  if (value === "openai_compatible") return "OpenAI Compatible";
  if (value === "mock") return "Mock";
  return "读取中";
}

function taskLabel(status: string) {
  const labels: Record<string, string> = {
    running: "运行中",
    stopping: "停止中",
    completed: "已完成",
    cancelled: "已取消",
    failed: "失败",
  };
  return labels[status] ?? status;
}

function taskTone(status: string) {
  if (status === "running" || status === "completed") return "success";
  if (status === "stopping") return "warning";
  if (status === "failed") return "danger";
  return "neutral";
}
