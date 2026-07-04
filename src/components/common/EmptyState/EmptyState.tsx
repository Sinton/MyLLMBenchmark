import type { ReactNode } from "react";

type EmptyStateProps = {
  icon?: ReactNode;
  title: string;
  description: string;
  action?: ReactNode;
  compact?: boolean;
};

export function EmptyState({
  icon,
  title,
  description,
  action,
  compact = false,
}: EmptyStateProps) {
  return (
    <div className={`empty-state ${compact ? "compact" : ""}`.trim()}>
      {icon && <div className="empty-state-icon">{icon}</div>}
      <strong>{title}</strong>
      <span>{description}</span>
      {action && <div className="empty-state-action">{action}</div>}
    </div>
  );
}

