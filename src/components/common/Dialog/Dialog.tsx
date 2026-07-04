import { useEffect, useId, useRef } from "react";
import type { CSSProperties, ReactNode } from "react";
import { X } from "../icons";

type DialogProps = {
  open: boolean;
  title: string;
  description?: string;
  children: ReactNode;
  className?: string;
  footer?: ReactNode;
  variant?: "modal" | "drawer";
  width?: string;
  onClose: () => void;
};

export function Dialog({
  open,
  title,
  description,
  children,
  className = "",
  footer,
  variant = "modal",
  width,
  onClose,
}: DialogProps) {
  const titleId = useId();
  const descriptionId = useId();
  const panelRef = useRef<HTMLElement | null>(null);
  const onCloseRef = useRef(onClose);

  useEffect(() => {
    onCloseRef.current = onClose;
  }, [onClose]);

  useEffect(() => {
    if (!open) return;
    const restoreTarget = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;

    const focusTimer = window.setTimeout(() => {
      getInitialFocusElement(panelRef.current)?.focus();
      if (!panelRef.current?.contains(document.activeElement)) {
        panelRef.current?.focus();
      }
    });

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onCloseRef.current();
        return;
      }
      if (event.key === "Tab") {
        trapFocus(event, panelRef.current);
      }
    };

    document.addEventListener("keydown", onKeyDown);
    return () => {
      window.clearTimeout(focusTimer);
      document.removeEventListener("keydown", onKeyDown);
      restoreTarget?.focus();
    };
  }, [open]);

  if (!open) return null;

  const panelClassName = [
    "dialog-panel",
    `dialog-panel-${variant}`,
    className,
  ]
    .filter(Boolean)
    .join(" ");
  const panelStyle = width
    ? ({ "--dialog-width": width } as CSSProperties)
    : undefined;

  return (
    <div
      className={`dialog-backdrop dialog-backdrop-${variant}`}
      role="presentation"
      onMouseDown={onClose}
    >
      <section
        aria-describedby={description ? descriptionId : undefined}
        aria-labelledby={titleId}
        aria-modal="true"
        className={panelClassName}
        ref={panelRef}
        role="dialog"
        style={panelStyle}
        tabIndex={-1}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <header className="dialog-header">
          <div>
            <h2 id={titleId}>{title}</h2>
            {description && <p id={descriptionId}>{description}</p>}
          </div>
          <button
            aria-label="关闭"
            className="dialog-close"
            onClick={onClose}
            title="关闭"
            type="button"
          >
            <X size={17} />
          </button>
        </header>
        <div className="dialog-body">{children}</div>
        {footer && <footer className="dialog-footer">{footer}</footer>}
      </section>
    </div>
  );
}

function trapFocus(event: KeyboardEvent, root: HTMLElement | null) {
  const focusable = getFocusableElements(root);
  if (!focusable.length) {
    event.preventDefault();
    root?.focus();
    return;
  }

  const first = focusable[0];
  const last = focusable[focusable.length - 1];
  const active = document.activeElement;

  if (event.shiftKey && active === first) {
    event.preventDefault();
    last.focus();
  } else if (!event.shiftKey && active === last) {
    event.preventDefault();
    first.focus();
  }
}

function getFocusableElements(root: HTMLElement | null) {
  if (!root) return [];
  return Array.from(
    root.querySelectorAll<HTMLElement>(
      'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
    ),
  ).filter((element) => !element.hasAttribute("hidden"));
}

function getInitialFocusElement(root: HTMLElement | null) {
  if (!root) return null;
  return (
    root.querySelector<HTMLElement>("[data-autofocus]") ??
    root.querySelector<HTMLElement>(
      'input:not([disabled]), textarea:not([disabled]), select:not([disabled])',
    ) ??
    getFocusableElements(root)[0] ??
    null
  );
}

