import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { Pagination } from "./Pagination";

function renderPagination(pageSize: number, pageSizeOptions?: number[]) {
  return renderToStaticMarkup(
    <Pagination
      itemLabel="请求"
      page={1}
      pageSize={pageSize}
      pageSizeOptions={pageSizeOptions}
      total={120}
      onPageChange={() => undefined}
      onPageSizeChange={() => undefined}
    />,
  );
}

describe("Pagination page size options", () => {
  it("uses the default page-size options", () => {
    const markup = renderPagination(100);

    expect(markup).toContain("100 条 / 页");
  });

  it("uses custom page-size options when provided", () => {
    const markup = renderPagination(50, [20, 50]);

    expect(markup).toContain("50 条 / 页");
    expect(markup).not.toContain("100 条 / 页");
  });
});
