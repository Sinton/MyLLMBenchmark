import { Picker } from "../Picker";

export type SelectOption<T extends string = string> = {
  label: string;
  value: T;
  description?: string;
};

type SelectFieldProps<T extends string = string> = {
  label?: string;
  value: T;
  options: Array<SelectOption<T>>;
  onChange: (value: T) => void;
  disabled?: boolean;
  placeholder?: string;
};

export function SelectField<T extends string = string>({
  label,
  value,
  options,
  onChange,
  disabled = false,
  placeholder = "请选择",
}: SelectFieldProps<T>) {
  return (
    <Picker
      disabled={disabled}
      label={label}
      onChange={(nextValue) => onChange(nextValue)}
      options={options}
      placeholder={placeholder}
      value={value}
    />
  );
}

