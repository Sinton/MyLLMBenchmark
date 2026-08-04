import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { ToastViewport, type ToastItem } from "./Toast";

const items: ToastItem[] = [
  {
    id: "toast-1",
    message: "测活批次已删除",
    tone: "success",
  },
];

describe("Toast", () => {
  it("renders a compact single-line message without a close action", () => {
    const markup = renderToStaticMarkup(
      <ToastViewport items={items} onDismiss={() => undefined} />,
    );

    expect(markup).toContain("测活批次已删除");
    expect(markup).toContain("toast-viewport");
    expect(markup).not.toContain("<strong");
    expect(markup).not.toContain("<button");
    expect(markup).toContain('role="status"');
  });
});
