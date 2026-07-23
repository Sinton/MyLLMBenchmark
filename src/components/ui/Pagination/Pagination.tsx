import { Button } from "../Button";
import { SelectField } from "../SelectField";

const DEFAULT_PAGE_SIZE_OPTIONS = [20, 50, 100, 200];

const toPageSizeOptions = (sizes: number[]) =>
  sizes.map((size) => ({
    label: `${size} 条 / 页`,
    value: String(size),
  }));

type PaginationProps = {
  page: number;
  pageSize: number;
  total: number;
  onPageChange: (page: number) => void;
  onPageSizeChange: (pageSize: number) => void;
  disabled?: boolean;
  itemLabel?: string;
  pageSizeOptions?: number[];
};

export function Pagination({
  page,
  pageSize,
  total,
  onPageChange,
  onPageSizeChange,
  disabled = false,
  itemLabel = "样本",
  pageSizeOptions = DEFAULT_PAGE_SIZE_OPTIONS,
}: PaginationProps) {
  const totalPages = Math.max(1, Math.ceil(total / pageSize));
  const safePage = Math.min(Math.max(page, 1), totalPages);
  const start = total === 0 ? 0 : (safePage - 1) * pageSize + 1;
  const end = Math.min(total, safePage * pageSize);
  const pageSizeSelectOptions = toPageSizeOptions(pageSizeOptions);

  return (
    <div className="pagination" aria-label="分页">
      <div className="pagination-summary">
        <strong>{total.toLocaleString("zh-CN")}</strong>
        <span>
          条{itemLabel}
          {total > 0 && `，当前 ${start.toLocaleString("zh-CN")}-${end.toLocaleString("zh-CN")}`}
        </span>
      </div>

      <div className="pagination-controls">
        <Button
          disabled={disabled || safePage <= 1}
          variant="ghost"
          onClick={() => onPageChange(safePage - 1)}
        >
          上一页
        </Button>
        <span className="pagination-page">
          {safePage.toLocaleString("zh-CN")} / {totalPages.toLocaleString("zh-CN")}
        </span>
        <Button
          disabled={disabled || safePage >= totalPages}
          variant="ghost"
          onClick={() => onPageChange(safePage + 1)}
        >
          下一页
        </Button>
        <SelectField
          disabled={disabled}
          options={pageSizeSelectOptions}
          value={String(pageSize)}
          onChange={(value) => onPageSizeChange(Number(value))}
        />
      </div>
    </div>
  );
}
