export type TabItem<T extends string = string> = {
  key: T;
  label: string;
};

type TabsProps<T extends string = string> = {
  items: ReadonlyArray<TabItem<T>>;
  value: T;
  onChange: (value: T) => void;
  className?: string;
  variant?: "segmented" | "line";
  ariaLabel?: string;
};

export function Tabs<T extends string = string>({
  items,
  value,
  onChange,
  className = "",
  variant = "segmented",
  ariaLabel,
}: TabsProps<T>) {
  return (
    <div
      aria-label={ariaLabel}
      className={`tabs tabs-${variant} ${className}`.trim()}
      role="tablist"
    >
      {items.map((item) => (
        <button
          aria-selected={value === item.key}
          className={value === item.key ? "active" : ""}
          key={item.key}
          onClick={() => onChange(item.key)}
          role="tab"
          type="button"
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}

