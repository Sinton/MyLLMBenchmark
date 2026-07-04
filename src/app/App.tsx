import { lazy, Suspense } from "react";
import { Link, Navigate, Route, Routes, useLocation } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { api } from "../api/client";
import { queryKeys } from "../api/queryKeys";
import { Button } from "../components/common/Button";
import { DesktopShell, type DesktopNavItem } from "../components/common/DesktopShell";
import {
  Activity,
  Building2,
  Database,
  FileText,
  LayoutDashboard,
  Plus,
  Rocket,
  Settings as SettingsIcon,
} from "../components/common/icons";
import { LoadingBlock } from "../components/common/LoadingBlock";
import { StatusBarItem } from "../components/common/StatusBarItem";
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
  { to: "/settings", label: "系统设置", shortLabel: "设置", icon: SettingsIcon },
];

const moduleMeta = [
  {
    path: "/dashboard",
    title: "启动中心",
    subtitle: "本地模型压测工作台概览",
  },
  {
    path: "/providers",
    title: "模型服务商",
    subtitle: "连接、检测和管理模型入口",
  },
  {
    path: "/datasets",
    title: "测试数据集",
    subtitle: "维护 Prompt 样本和负载数据",
  },
  {
    path: "/workbench",
    title: "压测工作台",
    subtitle: "配置任务、观察实时指标、生成报告",
  },
  {
    path: "/reports",
    title: "测试报告",
    subtitle: "阅读容量结论和阶段证据链",
  },
  {
    path: "/settings",
    title: "系统设置",
    subtitle: "管理本地数据源和压测引擎",
  },
];

export function App() {
  const location = useLocation();
  const activeTask = useWorkbenchStore((state) => state.activeTask);
  const latestTick = useWorkbenchStore((state) => state.latestTick);
  const configQuery = useQuery({
    queryKey: queryKeys.appConfig(),
    queryFn: api.getAppConfig,
    staleTime: 60_000,
  });
  const activeModule =
    moduleMeta.find((item) => location.pathname.startsWith(item.path)) ??
    moduleMeta[0];

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
            value={latestTick ? `第 ${latestTick.elapsed_seconds} 轮` : "待命"}
          />
        </>
      }
      toolbarActions={
        <>
          <Link to="/providers">
            <Button icon={<Plus size={15} />}>服务商</Button>
          </Link>
          <Link to="/workbench">
            <Button icon={<Rocket size={15} />} variant="primary">
              开始压测
            </Button>
          </Link>
        </>
      }
      toolbarSubtitle={activeModule.subtitle}
      toolbarTitle={activeModule.title}
    >
      <Suspense fallback={<LoadingBlock text="正在打开工作区..." />}>
        <Routes>
          <Route path="/" element={<Navigate to="/dashboard" replace />} />
          <Route path="/dashboard" element={<Dashboard />} />
          <Route path="/providers" element={<Providers />} />
          <Route path="/datasets" element={<Datasets />} />
          <Route path="/workbench" element={<Workbench />} />
          <Route path="/reports" element={<Reports />} />
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
