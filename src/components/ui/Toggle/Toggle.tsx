type ToggleProps = {
  checked: boolean;
  onChange: (checked: boolean) => void;
  label?: string;
  disabled?: boolean;
  ariaLabel?: string;
  className?: string;
};

export function Toggle({
  checked,
  onChange,
  label,
  disabled = false,
  ariaLabel,
  className = "",
}: ToggleProps) {
  const accessibleLabel = label ?? ariaLabel ?? "切换开关";

  return (
    <label className={`toggle-row ${className}`.trim()}>
      <button
        aria-checked={checked}
        aria-label={label ? undefined : accessibleLabel}
        className={`toggle ${checked ? "checked" : ""}`}
        disabled={disabled}
        onClick={() => onChange(!checked)}
        role="switch"
        type="button"
      >
        <span aria-hidden="true" className="toggle-thumb" />
      </button>
      {label && <span>{label}</span>}
    </label>
  );
}
