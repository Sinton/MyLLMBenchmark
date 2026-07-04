import type { ReactNode } from "react";

export type DataTableColumn<T> = {
  key: string;
  title: ReactNode;
  render: (row: T) => ReactNode;
  align?: "left" | "right";
};

type DataTableProps<T> = {
  columns: Array<DataTableColumn<T>>;
  rows: T[];
  getRowKey: (row: T) => string | number;
  className?: string;
  empty?: ReactNode;
};

export function DataTable<T>({
  columns,
  rows,
  getRowKey,
  className = "",
  empty,
}: DataTableProps<T>) {
  if (!rows.length && empty) {
    return <>{empty}</>;
  }

  return (
    <div className={`table-wrap ${className}`.trim()}>
      <table>
        <thead>
          <tr>
            {columns.map((column) => (
              <th className={column.align === "right" ? "is-right" : ""} key={column.key}>
                {column.title}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={getRowKey(row)}>
              {columns.map((column) => (
                <td className={column.align === "right" ? "is-right" : ""} key={column.key}>
                  {column.render(row)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

