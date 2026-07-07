import type { ReactNode } from "react";
import { Search } from "../icons";

type CommandToolbarProps = {
  title: string;
  subtitle?: string;
  actions?: ReactNode;
};

export function CommandToolbar({
  title,
  subtitle,
  actions,
}: CommandToolbarProps) {
  return (
    <header className="command-toolbar">
      <div className="command-toolbar-title">
        <strong>{title}</strong>
        {subtitle && <em>{subtitle}</em>}
      </div>
      <div className="command-toolbar-search">
        <Search size={15} />
        <span>搜索服务商、模型、数据集、报告</span>
        <kbd>Ctrl K</kbd>
      </div>
      {actions && <div className="command-toolbar-actions">{actions}</div>}
    </header>
  );
}
