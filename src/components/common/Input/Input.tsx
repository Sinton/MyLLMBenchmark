import type { InputHTMLAttributes, ReactNode } from "react";

type InputProps = Omit<InputHTMLAttributes<HTMLInputElement>, "prefix"> & {
  label?: string;
  hint?: string;
  error?: string;
  prefix?: ReactNode;
  suffix?: ReactNode;
};

export function Input({
  label,
  hint,
  error,
  prefix,
  suffix,
  className = "",
  ...props
}: InputProps) {
  return (
    <label className={`input-field ${className}`.trim()}>
      {label && <span className="input-label">{label}</span>}
      <span className={`input-shell ${error ? "has-error" : ""}`}>
        {prefix && <span className="input-affix">{prefix}</span>}
        <input {...props} />
        {suffix && <span className="input-affix">{suffix}</span>}
      </span>
      {(error || hint) && <span className={error ? "input-error" : "input-hint"}>{error ?? hint}</span>}
    </label>
  );
}

