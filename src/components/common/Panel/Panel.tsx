import type { ReactNode } from "react";

type PanelProps = {
  title?: string;
  eyebrow?: string;
  toolbar?: ReactNode;
  children: ReactNode;
  className?: string;
  density?: "normal" | "compact";
};

export function Panel({
  title,
  eyebrow,
  toolbar,
  children,
  className = "",
  density = "normal",
}: PanelProps) {
  const panelClassName = ["panel", `panel-${density}`, className]
    .filter(Boolean)
    .join(" ");

  return (
    <section className={panelClassName}>
      {(title || eyebrow || toolbar) && (
        <div className="panel-header">
          <div>
            {eyebrow && <div className="eyebrow">{eyebrow}</div>}
            {title && <h2>{title}</h2>}
          </div>
          {toolbar}
        </div>
      )}
      {children}
    </section>
  );
}
