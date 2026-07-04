import type { ReactNode } from "react";

type WorkspaceHeaderProps = {
  title: string;
  subtitle?: string;
  breadcrumb?: string;
  actions?: ReactNode;
};

export function WorkspaceHeader({
  title,
  subtitle,
  breadcrumb,
  actions,
}: WorkspaceHeaderProps) {
  return (
    <header className="workspace-header">
      <div className="workspace-header-copy">
        {breadcrumb && <span className="workspace-breadcrumb">{breadcrumb}</span>}
        <div>
          <h1>{title}</h1>
          {subtitle && <p>{subtitle}</p>}
        </div>
      </div>
      {actions && <div className="workspace-header-actions">{actions}</div>}
    </header>
  );
}
