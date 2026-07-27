import {
  Fragment,
  useId,
  type CSSProperties,
  type MouseEvent,
  type ReactNode,
} from "react";
import { ChevronDown } from "../icons";

export type DataTableRowKey = string | number;

type DataTableColumnBase<T> = {
  key: string;
  title: ReactNode;
  render: (row: T) => ReactNode;
  align?: "left" | "center" | "right";
};

type DataTableColumnSizing =
  | {
      fixed: "left" | "right";
      width: number;
    }
  | {
      fixed?: undefined;
      width?: number;
    };

export type DataTableColumn<T> = DataTableColumnBase<T> & DataTableColumnSizing;

export type DataTableExpandable<T> = {
  expandedRowKey: DataTableRowKey | null;
  expandedRowRender: (row: T) => ReactNode;
  onExpandedRowChange: (key: DataTableRowKey | null) => void;
  rowExpandable?: (row: T) => boolean;
  expandOnRowClick?: boolean;
};

type DataTableProps<T> = {
  columns: Array<DataTableColumn<T>>;
  rows: T[];
  getRowKey: (row: T) => DataTableRowKey;
  className?: string;
  empty?: ReactNode;
  expandable?: DataTableExpandable<T>;
  scrollX?: number;
};

export function DataTable<T>({
  columns,
  rows,
  getRowKey,
  className = "",
  empty,
  expandable,
  scrollX,
}: DataTableProps<T>) {
  const tableId = useId().replaceAll(":", "");

  if (!rows.length && empty) {
    return <>{empty}</>;
  }

  const columnLayout = buildColumnLayout(columns);
  const hasSizedColumns = columns.some((column) => column.width !== undefined);
  const wrapperClassName = [
    "table-wrap",
    scrollX !== undefined ? "table-wrap-scroll-x" : "",
    className,
  ]
    .filter(Boolean)
    .join(" ");
  const tableClassName = hasSizedColumns ? "table-layout-fixed" : undefined;
  const tableStyle = scrollX === undefined ? undefined : { minWidth: scrollX };

  return (
    <div className={wrapperClassName}>
      <table className={tableClassName} style={tableStyle}>
        {hasSizedColumns && (
          <colgroup>
            {expandable && <col style={{ width: 38 }} />}
            {columns.map((column) => (
              <col key={column.key} style={buildColumnWidthStyle(column)} />
            ))}
          </colgroup>
        )}
        <thead>
          <tr>
            {expandable && (
              <th aria-label="展开详情" className="table-expand-column" />
            )}
            {columns.map((column, columnIndex) => (
              <th
                className={buildColumnClassName(column, columnIndex, columnLayout)}
                key={column.key}
                style={buildColumnCellStyle(column, columnLayout[columnIndex])}
              >
                {column.title}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row, rowIndex) => {
            const rowKey = getRowKey(row);
            const canExpand = Boolean(
              expandable && (expandable.rowExpandable?.(row) ?? true),
            );
            const isExpanded = canExpand && expandable?.expandedRowKey === rowKey;
            const expandedContentId = `${tableId}-expanded-${rowIndex}-${normalizeId(rowKey)}`;
            const toggleExpanded = () => {
              if (!expandable || !canExpand) return;
              expandable.onExpandedRowChange(isExpanded ? null : rowKey);
            };
            const rowClassName = [
              canExpand ? "table-row-expandable" : "",
              isExpanded ? "table-row-expanded" : "",
            ]
              .filter(Boolean)
              .join(" ");

            return (
              <Fragment key={rowKey}>
                <tr
                  className={rowClassName}
                  onClick={(event) => {
                    if (!expandable?.expandOnRowClick || isInteractiveTarget(event)) {
                      return;
                    }
                    toggleExpanded();
                  }}
                >
                  {expandable && (
                    <td className="table-expand-cell">
                      {canExpand && (
                        <button
                          aria-controls={expandedContentId}
                          aria-expanded={isExpanded}
                          aria-label={isExpanded ? "收起行详情" : "展开行详情"}
                          className="table-expand-button"
                          title={isExpanded ? "收起详情" : "展开详情"}
                          type="button"
                          onClick={(event) => {
                            event.stopPropagation();
                            toggleExpanded();
                          }}
                        >
                          <ChevronDown aria-hidden="true" size={16} />
                        </button>
                      )}
                    </td>
                  )}
                  {columns.map((column, columnIndex) => (
                    <td
                      className={buildColumnClassName(
                        column,
                        columnIndex,
                        columnLayout,
                      )}
                      key={column.key}
                      style={buildColumnCellStyle(
                        column,
                        columnLayout[columnIndex],
                      )}
                    >
                      {column.render(row)}
                    </td>
                  ))}
                </tr>
                {isExpanded && expandable && (
                  <tr className="table-expanded-row">
                    <td colSpan={columns.length + 1}>
                      <div className="table-expanded-content" id={expandedContentId}>
                        {expandable.expandedRowRender(row)}
                      </div>
                    </td>
                  </tr>
                )}
              </Fragment>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

function normalizeId(key: DataTableRowKey) {
  return String(key).replace(/[^a-zA-Z0-9_-]/g, "-");
}

type ColumnLayout = {
  fixedOffset?: number;
  isFixedEdge: boolean;
};

function buildColumnLayout<T>(columns: Array<DataTableColumn<T>>): ColumnLayout[] {
  const layout = columns.map<ColumnLayout>(() => ({ isFixedEdge: false }));
  let leftOffset = 0;
  let leftEdgeIndex = -1;

  columns.forEach((column, index) => {
    if (column.fixed !== "left") return;
    layout[index].fixedOffset = leftOffset;
    leftOffset += column.width;
    leftEdgeIndex = index;
  });

  let rightOffset = 0;
  let rightEdgeIndex = -1;
  for (let index = columns.length - 1; index >= 0; index -= 1) {
    const column = columns[index];
    if (column.fixed !== "right") continue;
    layout[index].fixedOffset = rightOffset;
    rightOffset += column.width;
    rightEdgeIndex = index;
  }

  if (leftEdgeIndex >= 0) layout[leftEdgeIndex].isFixedEdge = true;
  if (rightEdgeIndex >= 0) layout[rightEdgeIndex].isFixedEdge = true;
  return layout;
}

function buildColumnClassName<T>(
  column: DataTableColumn<T>,
  columnIndex: number,
  layout: ColumnLayout[],
) {
  return [
    column.align === "center" ? "is-center" : "",
    column.align === "right" ? "is-right" : "",
    column.fixed ? `is-fixed-${column.fixed}` : "",
    column.fixed && layout[columnIndex].isFixedEdge ? "is-fixed-edge" : "",
  ]
    .filter(Boolean)
    .join(" ");
}

function buildColumnWidthStyle<T>(
  column: DataTableColumn<T>,
): CSSProperties | undefined {
  return column.width === undefined ? undefined : { width: column.width };
}

function buildColumnCellStyle<T>(
  column: DataTableColumn<T>,
  layout: ColumnLayout,
): CSSProperties | undefined {
  if (!column.fixed) return undefined;

  const sizing = {
    width: column.width,
    minWidth: column.width,
    maxWidth: column.width,
  };
  return column.fixed === "left"
    ? { ...sizing, left: layout.fixedOffset }
    : { ...sizing, right: layout.fixedOffset };
}

function isInteractiveTarget(event: MouseEvent<HTMLTableRowElement>) {
  const target = event.target;
  return (
    target instanceof Element &&
    Boolean(target.closest("button, a, input, select, textarea, [role='button']"))
  );
}
