export type StatusTone =
  | "success"
  | "warning"
  | "danger"
  | "info"
  | "running"
  | "neutral";

export function statusTone(status: string): StatusTone {
  switch (status) {
    case "online":
    case "completed":
    case "ready":
      return "success";
    case "running":
    case "stopping":
      return "running";
    case "failed":
    case "error":
      return "danger";
    case "cancelled":
    case "unchecked":
    case "offline":
      return "neutral";
    default:
      return "info";
  }
}

export function statusLabel(status: string) {
  const labels: Record<string, string> = {
    online: "在线",
    offline: "离线",
    error: "异常",
    unchecked: "未检测",
    running: "运行中",
    stopping: "停止中",
    completed: "已完成",
    cancelled: "已取消",
    failed: "失败",
  };
  return labels[status] ?? status;
}
