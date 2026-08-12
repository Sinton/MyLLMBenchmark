import type { CSSProperties, ReactNode } from "react";
import { useEffect, useId, useLayoutEffect, useRef, useState } from "react";
import { createPortal } from "react-dom";

type PopoverProps = {
  trigger: (state: { contentId: string; open: boolean; toggle: () => void }) => ReactNode;
  children: ReactNode | ((state: { close: () => void }) => ReactNode);
  className?: string;
  disabled?: boolean;
  align?: "start" | "end";
};

type PopoverPosition = {
  left: number;
  top: number;
  triggerWidth: number;
};

export function Popover({
  trigger,
  children,
  className = "",
  disabled = false,
  align = "start",
}: PopoverProps) {
  const contentId = useId();
  const [open, setOpen] = useState(false);
  const [position, setPosition] = useState<PopoverPosition | null>(null);
  const rootRef = useRef<HTMLDivElement | null>(null);
  const contentRef = useRef<HTMLDivElement | null>(null);

  useLayoutEffect(() => {
    if (!open || disabled) return;

    const updatePosition = () => {
      const triggerRect = rootRef.current?.getBoundingClientRect();
      if (!triggerRect) return;

      const margin = 12;
      const contentRect = contentRef.current?.getBoundingClientRect();
      const contentWidth = contentRect?.width || triggerRect.width;
      const contentHeight = contentRect?.height || 0;
      const spaceBelow = window.innerHeight - triggerRect.bottom - margin;
      const placeAbove =
        contentHeight > 0 && spaceBelow < contentHeight && triggerRect.top > spaceBelow;

      const rawLeft =
        align === "end" ? triggerRect.right - contentWidth : triggerRect.left;
      const left = Math.max(
        margin,
        Math.min(rawLeft, window.innerWidth - margin - contentWidth),
      );
      const top = placeAbove
        ? Math.max(margin, triggerRect.top - contentHeight - 6)
        : Math.min(window.innerHeight - margin, triggerRect.bottom + 6);

      setPosition({ left, top, triggerWidth: triggerRect.width });
    };

    updatePosition();
    window.addEventListener("resize", updatePosition);
    window.addEventListener("scroll", updatePosition, true);
    return () => {
      window.removeEventListener("resize", updatePosition);
      window.removeEventListener("scroll", updatePosition, true);
    };
  }, [align, disabled, open]);

  useEffect(() => {
    if (!open) return;

    const onPointerDown = (event: PointerEvent) => {
      const target = event.target as Node;
      if (!rootRef.current?.contains(target) && !contentRef.current?.contains(target)) {
        setOpen(false);
      }
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpen(false);
      }
    };

    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [open]);

  const close = () => setOpen(false);

  return (
    <div className={`popover-root ${className}`.trim()} ref={rootRef}>
      {trigger({
        contentId,
        open,
        toggle: () => !disabled && setOpen((current) => !current),
      })}
      {open &&
        !disabled &&
        createPortal(
          <div
            className={`popover-content ${className}`.trim()}
            id={contentId}
            ref={contentRef}
            style={buildPopoverStyle(position)}
          >
            {typeof children === "function" ? children({ close }) : children}
          </div>,
          document.body,
        )}
    </div>
  );
}

function buildPopoverStyle(position: PopoverPosition | null): CSSProperties {
  return {
    left: position?.left ?? 0,
    top: position?.top ?? 0,
    "--popover-trigger-width": `${position?.triggerWidth ?? 0}px`,
  } as CSSProperties;
}
