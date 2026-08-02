import { KeyboardEvent, useId, useRef } from "react";
import { Check, ChevronDown } from "../icons";
import { Popover } from "../Popover";

export type PickerOption<T extends string = string> = {
  label: string;
  value: T;
  description?: string;
};

type PickerProps<T extends string = string> = {
  label?: string;
  ariaLabel?: string;
  value: T;
  options: Array<PickerOption<T>>;
  onChange: (value: T, option: PickerOption<T>) => void;
  disabled?: boolean;
  placeholder?: string;
};

export function Picker<T extends string = string>({
  label,
  ariaLabel,
  value,
  options,
  onChange,
  disabled = false,
  placeholder = "请选择",
}: PickerProps<T>) {
  const labelId = useId();
  const menuRef = useRef<HTMLDivElement | null>(null);
  const selected = options.find((option) => option.value === value);

  return (
    <div className="select-field">
      {label && (
        <span className="select-label" id={labelId}>
          {label}
        </span>
      )}
      <Popover
        className="select-popover"
        disabled={disabled}
        trigger={({ contentId, open, toggle }) => (
          <button
            aria-controls={open ? contentId : undefined}
            aria-expanded={open}
            aria-haspopup="listbox"
            aria-label={label ? undefined : ariaLabel ?? placeholder}
            aria-labelledby={label ? labelId : undefined}
            className={`select-trigger ${open ? "open" : ""}`}
            disabled={disabled}
            onClick={() => {
              toggle();
              window.setTimeout(() => focusSelectedOption(menuRef.current));
            }}
            onKeyDown={(event) => {
              if (event.key === "ArrowDown" || event.key === "ArrowUp") {
                event.preventDefault();
                if (!open) toggle();
                window.setTimeout(() => focusSelectedOption(menuRef.current));
              }
            }}
            type="button"
          >
            <span className={selected ? "" : "placeholder"}>
              {selected?.label ?? placeholder}
            </span>
            <ChevronDown size={16} />
          </button>
        )}
      >
        {({ close }) => (
        <div
          className="select-menu"
          ref={menuRef}
          role="listbox"
          onKeyDown={(event) => handleListboxKeyDown(event, close)}
        >
          {options.map((option) => (
            <button
              aria-selected={option.value === value}
              className={`select-option ${option.value === value ? "selected" : ""}`}
              key={option.value}
              onClick={() => {
                onChange(option.value, option);
                close();
              }}
              role="option"
              tabIndex={option.value === value ? 0 : -1}
              type="button"
            >
              <span>
                <strong>{option.label}</strong>
                {option.description && <em>{option.description}</em>}
              </span>
              {option.value === value && <Check size={15} />}
            </button>
          ))}
        </div>
        )}
      </Popover>
    </div>
  );
}

function handleListboxKeyDown(event: KeyboardEvent<HTMLDivElement>, close: () => void) {
  const options = Array.from(
    event.currentTarget.querySelectorAll<HTMLButtonElement>('[role="option"]'),
  );
  const index = options.findIndex((option) => option === document.activeElement);

  if (event.key === "Escape") {
    event.preventDefault();
    close();
    return;
  }
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    const direction = event.key === "ArrowDown" ? 1 : -1;
    const nextIndex = index < 0
      ? 0
      : (index + direction + options.length) % options.length;
    options[nextIndex]?.focus();
  }
  if (event.key === "Home") {
    event.preventDefault();
    options[0]?.focus();
  }
  if (event.key === "End") {
    event.preventDefault();
    options.at(-1)?.focus();
  }
}

function focusSelectedOption(menu: HTMLDivElement | null) {
  const selected =
    menu?.querySelector<HTMLButtonElement>('[role="option"][aria-selected="true"]') ??
    menu?.querySelector<HTMLButtonElement>('[role="option"]');
  selected?.focus();
}

