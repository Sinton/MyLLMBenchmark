import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { Tooltip } from "./Tooltip";

describe("Tooltip trigger focus", () => {
  it("keeps a non-interactive trigger keyboard focusable by default", () => {
    const markup = renderToStaticMarkup(
      <Tooltip content="指标说明">
        <span>说明</span>
      </Tooltip>,
    );

    expect(markup).toContain('tabindex="0"');
  });

  it("removes the wrapper from tab order for an interactive child", () => {
    const markup = renderToStaticMarkup(
      <Tooltip content="编辑样本" triggerFocusable={false}>
        <button aria-label="编辑样本" type="button" />
      </Tooltip>,
    );

    expect(markup).toContain('tabindex="-1"');
    expect(markup).toContain('aria-label="编辑样本"');
  });
});
