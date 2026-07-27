import type { CSSProperties, ReactNode } from "react";
import { useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

type TooltipPlacement = "top" | "bottom" | "left" | "right";

type TooltipProps = {
  children: ReactNode;
  content: ReactNode;
  placement?: TooltipPlacement;
  disabled?: boolean;
  ariaLabel?: string;
  triggerFocusable?: boolean;
};

type TooltipPosition = {
  left: number;
  top: number;
};

export function Tooltip({
  children,
  content,
  placement = "top",
  disabled = false,
  ariaLabel,
  triggerFocusable = true,
}: TooltipProps) {
  const tooltipId = useId();
  const [open, setOpen] = useState(false);
  const [position, setPosition] = useState<TooltipPosition | null>(null);
  const triggerRef = useRef<HTMLSpanElement | null>(null);
  const contentRef = useRef<HTMLDivElement | null>(null);

  useLayoutEffect(() => {
    if (!open || disabled) return;

    const updatePosition = () => {
      const triggerRect = triggerRef.current?.getBoundingClientRect();
      const contentRect = contentRef.current?.getBoundingClientRect();
      if (!triggerRect || !contentRect) return;

      const gap = 8;
      const margin = 10;
      let left = triggerRect.left + triggerRect.width / 2 - contentRect.width / 2;
      let top = triggerRect.top - contentRect.height - gap;

      if (placement === "bottom") {
        top = triggerRect.bottom + gap;
      }
      if (placement === "left") {
        left = triggerRect.left - contentRect.width - gap;
        top = triggerRect.top + triggerRect.height / 2 - contentRect.height / 2;
      }
      if (placement === "right") {
        left = triggerRect.right + gap;
        top = triggerRect.top + triggerRect.height / 2 - contentRect.height / 2;
      }

      setPosition({
        left: Math.max(margin, Math.min(left, window.innerWidth - contentRect.width - margin)),
        top: Math.max(margin, Math.min(top, window.innerHeight - contentRect.height - margin)),
      });
    };

    updatePosition();
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);
    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [disabled, open, placement]);

  useEffect(() => {
    if (!open) return;

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
        triggerRef.current?.focus();
      }
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [open]);

  const show = () => !disabled && setOpen(true);
  const hide = () => setOpen(false);

  return (
    <>
      <span
        aria-label={ariaLabel}
        aria-describedby={open ? tooltipId : undefined}
        className="tooltip-trigger"
        onBlur={hide}
        onFocus={show}
        onMouseEnter={show}
        onMouseLeave={hide}
        ref={triggerRef}
        tabIndex={triggerFocusable ? 0 : -1}
      >
        {children}
      </span>
      {open &&
        !disabled &&
        createPortal(
          <div
            className="tooltip-content"
            id={tooltipId}
            ref={contentRef}
            role="tooltip"
            style={buildStyle(position)}
          >
            {content}
          </div>,
          document.body,
        )}
    </>
  );
}

function buildStyle(position: TooltipPosition | null): CSSProperties {
  return {
    left: position?.left ?? 0,
    top: position?.top ?? 0,
  };
}
