import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { Checkbox } from "./Checkbox";

describe("Checkbox", () => {
  it("renders a standard checked checkbox", () => {
    const markup = renderToStaticMarkup(
      <Checkbox aria-label="选择样本" checked readOnly />,
    );

    expect(markup).toContain('type="checkbox"');
    expect(markup).toContain('class="checkbox"');
    expect(markup).toContain("checked");
  });

  it("exposes a mixed state for partial selection", () => {
    const markup = renderToStaticMarkup(
      <Checkbox aria-label="选择当前页全部样本" indeterminate readOnly />,
    );

    expect(markup).toContain('aria-checked="mixed"');
  });

  it("forwards the disabled state", () => {
    const markup = renderToStaticMarkup(
      <Checkbox aria-label="选择样本" disabled readOnly />,
    );

    expect(markup).toContain("disabled");
  });
});
