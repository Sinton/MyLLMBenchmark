import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import {
  NotificationViewport,
  notificationDurationMs,
  type NotificationItem,
} from "./Notification";

const items: NotificationItem[] = [
  {
    id: "notice-1",
    title: "模型列表已更新",
    description: "已扫描到 4 个模型",
    tone: "success",
  },
];

describe("Notification", () => {
  it("renders a titled notice in the configured corner", () => {
    const markup = renderToStaticMarkup(
      <NotificationViewport
        items={items}
        position="bottom-left"
        onDismiss={() => undefined}
      />,
    );

    expect(markup).toContain("notification-viewport-bottom-left");
    expect(markup).toContain("模型列表已更新");
    expect(markup).toContain("已扫描到 4 个模型");
    expect(markup).toContain("关闭通知：模型列表已更新");
    expect(markup).toContain('role="status"');
  });

  it("uses longer durations for warnings and errors", () => {
    expect(notificationDurationMs("success")).toBe(4_000);
    expect(notificationDurationMs("info")).toBe(4_000);
    expect(notificationDurationMs("warning")).toBe(6_000);
    expect(notificationDurationMs("danger")).toBe(8_000);
  });
});
