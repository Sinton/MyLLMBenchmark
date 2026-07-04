export type TabItem<T extends string = string> = {
  key: T;
  label: string;
};

type TabsProps<T extends string = string> = {
  items: ReadonlyArray<TabItem<T>>;
  value: T;
  onChange: (value: T) => void;
  className?: string;
};

export function Tabs<T extends string = string>({
  items,
  value,
  onChange,
  className = "",
}: TabsProps<T>) {
  return (
    <div className={`segment ${className}`.trim()}>
      {items.map((item) => (
        <button
          className={value === item.key ? "active" : ""}
          key={item.key}
          onClick={() => onChange(item.key)}
          type="button"
        >
          {item.label}
        </button>
      ))}
    </div>
  );
}

