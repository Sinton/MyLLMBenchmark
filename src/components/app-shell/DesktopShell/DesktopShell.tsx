import type { ComponentType, ReactNode } from "react";
import { ActivityRailItem } from "../ActivityRailItem";
import { CommandToolbar } from "../CommandToolbar";
import { StatusBar } from "../StatusBar";
import { WindowTitleBar } from "../WindowTitleBar";

export type DesktopNavItem = {
  to: string;
  label: string;
  shortLabel: string;
  icon: ComponentType<{ size?: number }>;
};

type DesktopShellProps = {
  navItems: DesktopNavItem[];
  toolbarTitle: string;
  toolbarSubtitle?: string;
  toolbarActions?: ReactNode;
  statusLeft?: ReactNode;
  statusRight?: ReactNode;
  children: ReactNode;
};

export function DesktopShell({
  navItems,
  toolbarTitle,
  toolbarSubtitle,
  toolbarActions,
  statusLeft,
  statusRight,
  children,
}: DesktopShellProps) {
  return (
    <div className="desktop-shell">
      <WindowTitleBar />
      <aside className="activity-rail" aria-label="主导航">
        <div
          aria-label="MyLLMBenchmark"
          className="activity-brand"
          role="img"
          title="MyLLMBenchmark"
        >
          <img
            alt=""
            aria-hidden="true"
            draggable={false}
            src="/logo.png"
          />
        </div>
        <nav className="activity-nav">
          {navItems.map((item) => (
            <ActivityRailItem key={item.to} {...item} />
          ))}
        </nav>
      </aside>

      <section className="desktop-main">
        <CommandToolbar
          actions={toolbarActions}
          subtitle={toolbarSubtitle}
          title={toolbarTitle}
        />
        <main className="workspace">{children}</main>
        <StatusBar left={statusLeft} right={statusRight} />
      </section>
    </div>
  );
}
