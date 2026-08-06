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

const sizedColumns: Array<DataTableColumn<Row>> = [
  {
    key: "select",
    title: "选择",
    align: "center",
    fixed: "left",
    width: 44,
    render: () => "选中",
  },
  { key: "name", title: "名称", render: (row) => row.name },
  {
    key: "status",
    title: "状态",
    fixed: "right",
    width: 80,
    render: () => "完成",
  },
  {
    key: "actions",
    title: "操作",
    align: "center",
    fixed: "right",
    width: 96,
    render: () => "查看",
  },
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

describe("DataTable column layout", () => {
  it("renders semantic widths, horizontal scrolling, and center alignment", () => {
    const markup = renderToStaticMarkup(
      <DataTable
        columns={sizedColumns}
        getRowKey={(row) => row.id}
        rows={rows}
        scrollX={760}
      />,
    );

    expect(markup).toContain("table-wrap-scroll-x");
    expect(markup).toContain("table-layout-fixed");
    expect(markup).toContain("min-width:760px");
    expect(markup).toContain("<colgroup>");
    expect(markup).toContain("width:44px");
    expect(markup).toContain("is-center is-fixed-left is-fixed-edge");
  });

  it("calculates offsets for multiple fixed columns", () => {
    const markup = renderToStaticMarkup(
      <DataTable
        columns={sizedColumns}
        getRowKey={(row) => row.id}
        rows={rows}
        scrollX={760}
      />,
    );

    expect(markup).toContain("left:0");
    expect(markup).toContain("right:0");
    expect(markup).toContain("right:96px");
    expect(markup).toContain('class="is-fixed-right is-fixed-edge"');
  });
});

describe("DataTable selectable rows", () => {
  it("renders clickable rows with selected state and keyboard focus", () => {
    const markup = renderToStaticMarkup(
      <DataTable
        columns={columns}
        getRowAriaLabel={(row) => `查看 ${row.name}`}
        getRowKey={(row) => row.id}
        onRowClick={() => undefined}
        rows={rows}
        selectedRowKey="row-1"
      />,
    );

    expect(markup).toContain("table-row-clickable");
    expect(markup).toContain("table-row-selected");
    expect(markup).toContain('aria-label="查看 第一行"');
    expect(markup).toContain('aria-selected="true"');
    expect(markup).toContain('tabindex="0"');
  });
});
