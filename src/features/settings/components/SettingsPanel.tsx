import type { ReactNode } from "react";
import { Card } from "../../../components/ui/Card";

type SettingsPanelProps = {
  icon: ReactNode;
  title: string;
  description: string;
  children: ReactNode;
};

export function SettingsPanel({
  icon,
  title,
  description,
  children,
}: SettingsPanelProps) {
  return (
    <Card className="settings-card">
      <div className="settings-block">
        <div className="settings-icon">{icon}</div>
        <div>
          <h2>{title}</h2>
          <p>{description}</p>
        </div>
      </div>
      <div className="settings-section-fields">{children}</div>
    </Card>
  );
}

type SettingRowProps = {
  label: string;
  description?: string;
  children: ReactNode;
};

export function SettingRow({ label, description, children }: SettingRowProps) {
  return (
    <div className="setting-row">
      <div className="setting-row-copy">
        <strong>{label}</strong>
        {description && <span>{description}</span>}
      </div>
      <div className="setting-row-control">{children}</div>
    </div>
  );
}
