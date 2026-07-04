import { useId, useState, type ReactNode } from "react";
import { ChevronDown } from "../icons";

type DisclosureProps = {
  title: string;
  description?: string;
  defaultOpen?: boolean;
  children: ReactNode;
  className?: string;
};

export function Disclosure({
  title,
  description,
  defaultOpen = false,
  children,
  className = "",
}: DisclosureProps) {
  const [open, setOpen] = useState(defaultOpen);
  const panelId = useId();
  const classNames = ["disclosure", open ? "disclosure-open" : "", className]
    .filter(Boolean)
    .join(" ");

  return (
    <section className={classNames}>
      <button
        aria-controls={panelId}
        aria-expanded={open}
        className="disclosure-trigger"
        type="button"
        onClick={() => setOpen((current) => !current)}
      >
        <span>
          <strong>{title}</strong>
          {description && <em>{description}</em>}
        </span>
        <ChevronDown size={17} />
      </button>
      {open && (
        <div className="disclosure-panel" id={panelId}>
          {children}
        </div>
      )}
    </section>
  );
}
