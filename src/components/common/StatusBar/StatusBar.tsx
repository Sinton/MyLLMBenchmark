import type { ReactNode } from "react";

type StatusBarProps = {
  left?: ReactNode;
  right?: ReactNode;
};

export function StatusBar({ left, right }: StatusBarProps) {
  return (
    <footer className="desktop-status-bar">
      <div className="desktop-status-group">{left}</div>
      <div className="desktop-status-group">{right}</div>
    </footer>
  );
}
