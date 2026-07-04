import type { ReactNode } from "react";

type StatusBarItemTone = "neutral" | "success" | "warning" | "danger";

type StatusBarItemProps = {
  label: string;
  value: ReactNode;
  tone?: StatusBarItemTone;
};

export function StatusBarItem({
  label,
  value,
  tone = "neutral",
}: StatusBarItemProps) {
  return (
    <div className={`status-bar-item status-bar-item-${tone}`}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
