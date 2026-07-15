import type { TextareaHTMLAttributes } from "react";

type TextareaProps = TextareaHTMLAttributes<HTMLTextAreaElement> & {
  label?: string;
  hint?: string;
  error?: string;
};

export function Textarea({
  label,
  hint,
  error,
  className = "",
  ...props
}: TextareaProps) {
  return (
    <label className={`textarea-field ${className}`.trim()}>
      {label && <span className="textarea-label">{label}</span>}
      <span className={`textarea-shell ${error ? "has-error" : ""}`}>
        <textarea {...props} />
      </span>
      {(error || hint) && (
        <span className={error ? "textarea-error" : "textarea-hint"}>
          {error ?? hint}
        </span>
      )}
    </label>
  );
}
