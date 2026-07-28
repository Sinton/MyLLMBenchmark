import { useEffect, useRef, type InputHTMLAttributes } from "react";

export type CheckboxProps = Omit<InputHTMLAttributes<HTMLInputElement>, "type"> & {
  indeterminate?: boolean;
};

export function Checkbox({
  checked,
  className = "",
  indeterminate = false,
  ...props
}: CheckboxProps) {
  const inputRef = useRef<HTMLInputElement | null>(null);

  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.indeterminate = indeterminate;
    }
  }, [indeterminate]);

  return (
    <input
      {...props}
      aria-checked={indeterminate ? "mixed" : props["aria-checked"]}
      checked={checked}
      className={`checkbox ${className}`.trim()}
      ref={inputRef}
      type="checkbox"
    />
  );
}
