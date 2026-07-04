import type { ComponentType } from "react";
import { NavLink } from "react-router-dom";

type ActivityRailItemProps = {
  to: string;
  label: string;
  shortLabel: string;
  icon: ComponentType<{ size?: number }>;
};

export function ActivityRailItem({
  to,
  label,
  shortLabel,
  icon: Icon,
}: ActivityRailItemProps) {
  return (
    <NavLink className="activity-rail-item" title={label} to={to}>
      <span className="activity-rail-indicator" aria-hidden="true" />
      <Icon size={19} />
      <span>{shortLabel}</span>
    </NavLink>
  );
}
