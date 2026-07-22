import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { DataTable, type DataTableColumn } from "./DataTable";

type Row = {
  id: string;
  name: string;
};

const rows: Row[] = [{ id: "row-1", name: "第一行" }];
const columns: Array<DataTableColumn<Row>> = [
  { key: "name", title: "名称", render: (row) => row.name },
];

function renderTable(expandedRowKey: string | null, rowExpandable = true) {
  return renderToStaticMarkup(
    <DataTable
      columns={columns}
      expandable={{
        expandedRowKey,
        expandedRowRender: (row) => <div>详情：{row.name}</div>,
        onExpandedRowChange: () => undefined,
        rowExpandable: () => rowExpandable,
      }}
      getRowKey={(row) => row.id}
      rows={rows}
    />,
  );
}

describe("DataTable expandable rows", () => {
  it("renders a collapsed row with accessible expansion state", () => {
    const markup = renderTable(null);

    expect(markup).toContain('aria-expanded="false"');
    expect(markup).toContain("展开行详情");
    expect(markup).not.toContain("详情：第一行");
  });

  it("renders expanded content across the expansion and data columns", () => {
    const markup = renderTable("row-1");

    expect(markup).toContain('aria-expanded="true"');
    expect(markup).toContain('colSpan="2"');
    expect(markup).toContain("详情：第一行");
  });

  it("does not render an expansion button for a non-expandable row", () => {
    const markup = renderTable(null, false);

    expect(markup).not.toContain("展开行详情");
    expect(markup).not.toContain('aria-expanded="false"');
  });
});
