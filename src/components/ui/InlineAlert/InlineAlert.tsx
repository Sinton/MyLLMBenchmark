import type { ReactNode } from "react";
import { AlertCircle, CheckCircle2 } from "../icons";

type InlineAlertTone = "info" | "success" | "warning" | "danger";

type InlineAlertProps = {
  tone?: InlineAlertTone;
  title?: string;
  children: ReactNode;
};

export function InlineAlert({ tone = "info", title, children }: InlineAlertProps) {
  const Icon = tone === "success" ? CheckCircle2 : AlertCircle;

  return (
    <div className={`inline-alert inline-alert-${tone}`}>
      <Icon size={17} />
      <div className="inline-alert-body">
        {title && <strong>{title}</strong>}
        <div className="inline-alert-message">{children}</div>
      </div>
    </div>
  );
}

