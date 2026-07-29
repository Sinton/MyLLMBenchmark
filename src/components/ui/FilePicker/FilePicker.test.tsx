import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { FilePicker } from "./FilePicker";

describe("FilePicker", () => {
  it("renders a consistent empty state and native file input", () => {
    const markup = renderToStaticMarkup(
      <FilePicker
        accept=".jsonl,.csv"
        file={null}
        label="数据集文件"
        onFileChange={() => undefined}
      />,
    );

    expect(markup).toContain("尚未选择文件");
    expect(markup).toContain("选择文件");
    expect(markup).toContain('type="file"');
    expect(markup).toContain('aria-label="数据集文件：选择文件"');
    expect(markup).toContain('accept=".jsonl,.csv"');
    expect(markup).toContain("hidden");
  });

  it("renders selected file metadata and a clear action", () => {
    const file = { name: "samples.jsonl", size: 1536 } as File;
    const markup = renderToStaticMarkup(
      <FilePicker file={file} onFileChange={() => undefined} />,
    );

    expect(markup).toContain("samples.jsonl");
    expect(markup).toContain("1.5 KB");
    expect(markup).toContain("重新选择");
    expect(markup).toContain('aria-label="清除文件 samples.jsonl"');
  });

  it("renders hint and error states", () => {
    const hintMarkup = renderToStaticMarkup(
      <FilePicker file={null} hint="最大 10MB" onFileChange={() => undefined} />,
    );
    const errorMarkup = renderToStaticMarkup(
      <FilePicker file={null} error="文件无效" onFileChange={() => undefined} />,
    );

    expect(hintMarkup).toContain("file-picker-hint");
    expect(errorMarkup).toContain("file-picker-error");
    expect(errorMarkup).toContain("has-error");
  });
});
