import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { DatasetSamplePreview } from "../../../types/api";
import { DatasetSampleExpandedRow } from "./DatasetSampleExpandedRow";
import { DatasetSampleList } from "./DatasetSampleList";

const sample: DatasetSamplePreview = {
  id: "sample-1",
  sample_index: 0,
  prompt: "请根据业务背景生成一份完整分析。",
  prompt_preview: "请根据业务背景生成一份完整分析。",
  estimated_tokens: 18,
};

describe("DatasetSampleList", () => {
  it("renders page selection in the header and expandable sample rows", () => {
    const markup = renderToStaticMarkup(
      <DatasetSampleList
        datasetId="dataset-1"
        onBatchDelete={() => undefined}
        onCreate={async () => undefined}
        onDelete={() => undefined}
        onPageChange={() => undefined}
        onPageSizeChange={() => undefined}
        onSearchChange={() => undefined}
        onUpdate={async () => undefined}
        page={1}
        pageSize={20}
        samples={[sample]}
        search=""
        total={1}
      />,
    );

    expect(markup).toContain('aria-label="选择当前页全部样本"');
    expect(markup).not.toContain("全选本页");
    expect(markup).toContain('aria-expanded="false"');
    expect(markup).toContain('aria-label="编辑第 1 条样本"');
    expect(markup).toContain('aria-label="删除第 1 条样本"');
  });

  it("renders the complete prompt and copy action in an expanded row", () => {
    const markup = renderToStaticMarkup(
      <DatasetSampleExpandedRow sample={sample} />,
    );

    expect(markup).toContain("完整 Prompt");
    expect(markup).toContain(sample.prompt);
    expect(markup).toContain('aria-label="复制第 1 条样本 Prompt"');
  });
});
