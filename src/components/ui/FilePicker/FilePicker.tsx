import { useId, useRef } from "react";
import { Button } from "../Button";
import { FileText, FolderOpen, X } from "../icons";
import { Tooltip } from "../Tooltip";

export type FilePickerProps = {
  accept?: string;
  chooseText?: string;
  className?: string;
  disabled?: boolean;
  error?: string;
  file: File | null;
  hint?: string;
  label?: string;
  onFileChange: (file: File | null) => void;
  placeholder?: string;
};

export function FilePicker({
  accept,
  chooseText,
  className = "",
  disabled = false,
  error,
  file,
  hint,
  label,
  onFileChange,
  placeholder = "尚未选择文件",
}: FilePickerProps) {
  const descriptionId = useId();
  const inputRef = useRef<HTMLInputElement | null>(null);
  const description = error ?? hint;
  const actionText = chooseText ?? (file ? "重新选择" : "选择文件");

  const openFileDialog = () => {
    if (!disabled) inputRef.current?.click();
  };

  const clearFile = () => {
    if (inputRef.current) inputRef.current.value = "";
    onFileChange(null);
  };

  return (
    <div className={`file-picker ${className}`.trim()}>
      {label && (
        <span className="file-picker-label">
          {label}
        </span>
      )}
      <div
        className={`file-picker-shell ${file ? "has-file" : ""} ${error ? "has-error" : ""} ${disabled ? "is-disabled" : ""}`.trim()}
      >
        <FileText aria-hidden="true" className="file-picker-file-icon" size={16} />
        <span className="file-picker-summary">
          <strong title={file?.name}>{file?.name ?? placeholder}</strong>
          {file && <small>{formatFileSize(file.size)}</small>}
        </span>
        {file && (
          <Tooltip content="清除已选文件" triggerFocusable={false}>
            <Button
              aria-label={`清除文件 ${file.name}`}
              className="file-picker-clear"
              disabled={disabled}
              icon={<X aria-hidden="true" size={14} />}
              type="button"
              variant="ghost"
              onClick={clearFile}
            />
          </Tooltip>
        )}
        <Button
          aria-describedby={description ? descriptionId : undefined}
          aria-label={label ? `${label}：${actionText}` : actionText}
          className="file-picker-choose"
          disabled={disabled}
          icon={<FolderOpen aria-hidden="true" size={15} />}
          type="button"
          variant="ghost"
          onClick={openFileDialog}
        >
          {actionText}
        </Button>
        <input
          accept={accept}
          disabled={disabled}
          hidden
          ref={inputRef}
          type="file"
          onChange={(event) => {
            onFileChange(event.target.files?.[0] ?? null);
            event.target.value = "";
          }}
        />
      </div>
      {description && (
        <span
          className={error ? "file-picker-error" : "file-picker-hint"}
          id={descriptionId}
          role={error ? "alert" : undefined}
        >
          {description}
        </span>
      )}
    </div>
  );
}

function formatFileSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${formatSizeValue(bytes / 1024)} KB`;
  return `${formatSizeValue(bytes / (1024 * 1024))} MB`;
}

function formatSizeValue(value: number) {
  return new Intl.NumberFormat("zh-CN", { maximumFractionDigits: 1 }).format(value);
}
