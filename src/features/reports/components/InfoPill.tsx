import type { ReactNode } from "react";

type InfoPillProps = {
  icon: ReactNode;
  label: string;
  value: string;
};

export function InfoPill({ icon, label, value }: InfoPillProps) {
  return (
    <div className="info-pill">
      {icon}
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  );
}
