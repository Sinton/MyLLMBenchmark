import { describe, expect, it } from "vitest";
import {
  DEFAULT_ENDPOINT_PROBE_PROMPT,
  DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID,
  createEndpointProbePromptTemplate,
  normalizeEndpointProbePromptTemplates,
  selectedEndpointProbePromptTemplate,
} from "./endpointProbePromptTemplates";

describe("endpoint probe prompt templates", () => {
  it("defaults to a single basic liveness template", () => {
    const config = normalizeEndpointProbePromptTemplates(undefined);

    expect(config.selected_id).toBe(DEFAULT_ENDPOINT_PROBE_PROMPT_TEMPLATE_ID);
    expect(config.items).toHaveLength(1);
    expect(selectedEndpointProbePromptTemplate(config).prompt).toBe(
      DEFAULT_ENDPOINT_PROBE_PROMPT,
    );
  });

  it("falls back to the first valid template when selected id is missing", () => {
    const config = normalizeEndpointProbePromptTemplates({
      selected_id: "missing",
      items: [
        { id: "", name: "无效", prompt: "会被过滤" },
        { id: "custom", name: "自定义", prompt: "hello" },
      ],
    });

    expect(config.selected_id).toBe("custom");
    expect(config.items).toEqual([{ id: "custom", name: "自定义", prompt: "hello" }]);
  });

  it("creates the next custom template name", () => {
    const template = createEndpointProbePromptTemplate(
      [
        { id: "one", name: "自定义模板 1", prompt: "a" },
        { id: "two", name: "基础测活", prompt: "b" },
      ],
      "新的 Prompt",
    );

    expect(template.name).toBe("自定义模板 2");
    expect(template.prompt).toBe("新的 Prompt");
  });
});
