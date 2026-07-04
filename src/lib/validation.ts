export type ValidationResult = { ok: true } | { ok: false; message: string };

export const valid: ValidationResult = { ok: true };

export function firstError(validations: ValidationResult[]): string | null {
  return validations.find((result) => !result.ok)?.message ?? null;
}

export function finiteNumber(label: string, value: number): ValidationResult {
  return Number.isFinite(value) ? valid : invalid(`${label}不是有效数字。`);
}

export function minNumber(label: string, value: number, min: number): ValidationResult {
  return value >= min ? valid : invalid(`${label}必须大于等于 ${min}。`);
}

export function greaterThan(label: string, value: number, min: number): ValidationResult {
  return value > min ? valid : invalid(`${label}必须大于 ${min}。`);
}

export function maxNumber(label: string, value: number, max: number): ValidationResult {
  return value <= max ? valid : invalid(`${label}不能大于 ${max}。`);
}

export function numberRange(
  label: string,
  value: number,
  min: number,
  max: number,
): ValidationResult {
  if (value < min || value > max) {
    return invalid(`${label}必须在 ${min} 到 ${max} 之间。`);
  }
  return valid;
}

export function invalid(message: string): ValidationResult {
  return { ok: false, message };
}
