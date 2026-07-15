import type { ReactNode } from "react";
import { getMetricHelp } from "../../../domain/metricGlossary";
import { Info } from "../../ui/icons";
import { Tooltip } from "../../ui/Tooltip";

type MetricHelpProps = {
  children: ReactNode;
  helpKey?: string;
  description?: string;
  ariaLabel?: string;
};

export function MetricHelp({ children, helpKey, description, ariaLabel }: MetricHelpProps) {
  const content = description ?? getMetricHelp(helpKey);
  if (!content) {
    return <>{children}</>;
  }

  return (
    <span className="metric-help">
      <span>{children}</span>
      <Tooltip
        ariaLabel={ariaLabel ?? "指标说明"}
        content={content}
        placement="top"
      >
        <span className="metric-help-icon" aria-hidden="true">
          <Info size={13} />
        </span>
      </Tooltip>
    </span>
  );
}
