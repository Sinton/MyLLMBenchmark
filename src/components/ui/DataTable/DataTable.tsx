import { Fragment, useId, type MouseEvent, type ReactNode } from "react";
import { ChevronDown } from "../icons";

export type DataTableRowKey = string | number;

export type DataTableColumn<T> = {
  key: string;
  title: ReactNode;
  render: (row: T) => ReactNode;
  align?: "left" | "right";
};

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
};

export function DataTable<T>({
  columns,
  rows,
  getRowKey,
  className = "",
  empty,
  expandable,
}: DataTableProps<T>) {
  const tableId = useId().replaceAll(":", "");

  if (!rows.length && empty) {
    return <>{empty}</>;
  }

  return (
    <div className={`table-wrap ${className}`.trim()}>
      <table>
        <thead>
          <tr>
            {expandable && (
              <th aria-label="展开详情" className="table-expand-column" />
            )}
            {columns.map((column) => (
              <th
                className={column.align === "right" ? "is-right" : ""}
                key={column.key}
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
                  {columns.map((column) => (
                    <td
                      className={column.align === "right" ? "is-right" : ""}
                      key={column.key}
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

function isInteractiveTarget(event: MouseEvent<HTMLTableRowElement>) {
  const target = event.target;
  return (
    target instanceof Element &&
    Boolean(target.closest("button, a, input, select, textarea, [role='button']"))
  );
}
