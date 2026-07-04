import { Input } from "../../../components/common/Input";

type SecretInputProps = {
  hint?: string;
  label: string;
  onChange: (value: string) => void;
  placeholder?: string;
  value: string;
};

export function SecretInput({
  hint,
  label,
  onChange,
  placeholder,
  value,
}: SecretInputProps) {
  return (
    <Input
      autoComplete="off"
      className="secret-field-label"
      hint={hint}
      label={label}
      placeholder={placeholder}
      type="text"
      value={value}
      onChange={(event) => onChange(event.target.value)}
    />
  );
}
