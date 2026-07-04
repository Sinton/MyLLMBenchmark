import type { ReactNode } from "react";
import { WorkspaceHeader } from "../WorkspaceHeader";

type PageHeaderProps = {
  eyebrow: string;
  title: string;
  description?: string;
  actions?: ReactNode;
};

export function PageHeader({
  eyebrow,
  title,
  description,
  actions,
}: PageHeaderProps) {
  return (
    <WorkspaceHeader
      actions={actions}
      breadcrumb={eyebrow}
      subtitle={description}
      title={title}
    />
  );
}
