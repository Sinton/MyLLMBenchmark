import type { ReactNode } from "react";

type CardProps = {
  title?: string;
  eyebrow?: string;
  action?: ReactNode;
  children: ReactNode;
  className?: string;
  interactive?: boolean;
};

export function Card({
  title,
  eyebrow,
  action,
  children,
  className = "",
  interactive = false,
}: CardProps) {
  const cardClassName = ["card", interactive ? "card-interactive" : "", className]
    .filter(Boolean)
    .join(" ");

  return (
    <section className={cardClassName}>
      {(title || eyebrow || action) && (
        <div className="card-header">
          <div>
            {eyebrow && <div className="eyebrow">{eyebrow}</div>}
            {title && <h2>{title}</h2>}
          </div>
          {action}
        </div>
      )}
      {children}
    </section>
  );
}

