import type { ReactNode } from "react";
import { Badge } from "../../../components/ui/Badge";
import { Card } from "../../../components/ui/Card";

type SettingsPanelProps = {
  icon: ReactNode;
  title: string;
  description: string;
  status?: string;
  children: ReactNode;
};

export function SettingsPanel({
  icon,
  title,
  description,
  status = "本地配置",
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
        <Badge tone="neutral">{status}</Badge>
      </div>
      <div className="settings-fields">{children}</div>
    </Card>
  );
}
