import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useRef,
  useState,
  type FocusEvent,
  type ReactNode,
} from "react";
import { createPortal } from "react-dom";
import {
  AlertCircle,
  AlertTriangle,
  CheckCircle2,
  Info,
  X,
} from "../icons";
import { usePausableAutoDismiss } from "../feedbackTimer";

export type NotificationTone = "success" | "warning" | "danger" | "info";
export type NotificationPosition =
  | "top-right"
  | "top-left"
  | "bottom-right"
  | "bottom-left";

export type NotificationInput = {
  title: string;
  description?: string;
  tone?: NotificationTone;
  durationMs?: number | null;
};

export type NotificationItem = NotificationInput & {
  id: string;
};

type NotificationViewportProps = {
  items: NotificationItem[];
  position: NotificationPosition;
  onDismiss: (id: string) => void;
};

type NotificationContextValue = {
  dismissNotification: (id: string) => void;
  notify: (notification: NotificationInput) => string;
  position: NotificationPosition;
  setPosition: (position: NotificationPosition) => void;
};

const NotificationContext = createContext<NotificationContextValue | null>(null);

export function NotificationProvider({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<NotificationItem[]>([]);
  const [position, setPosition] =
    useState<NotificationPosition>("top-right");

  const dismissNotification = useCallback((id: string) => {
    setItems((current) => current.filter((item) => item.id !== id));
  }, []);

  const notify = useCallback((notification: NotificationInput) => {
    const id = crypto.randomUUID();
    setItems((current) => [{ id, ...notification }, ...current].slice(0, 3));
    return id;
  }, []);

  const value = useMemo<NotificationContextValue>(
    () => ({ dismissNotification, notify, position, setPosition }),
    [dismissNotification, notify, position],
  );

  const viewport = (
    <NotificationViewport
      items={items}
      position={position}
      onDismiss={dismissNotification}
    />
  );

  return (
    <NotificationContext.Provider value={value}>
      {children}
      {typeof document === "undefined"
        ? viewport
        : createPortal(viewport, document.body)}
    </NotificationContext.Provider>
  );
}

export function useNotification() {
  const context = useContext(NotificationContext);
  if (!context) {
    throw new Error(
      "useNotification must be used within NotificationProvider",
    );
  }
  return context;
}

export function NotificationViewport({
  items,
  position,
  onDismiss,
}: NotificationViewportProps) {
  if (!items.length) return null;

  return (
    <div
      className={`notification-viewport notification-viewport-${position}`}
      aria-label="应用通知"
      aria-live="polite"
      aria-relevant="additions"
    >
      {items.map((item) => (
        <NotificationNotice
          item={item}
          key={item.id}
          onDismiss={onDismiss}
        />
      ))}
    </div>
  );
}

function NotificationNotice({
  item,
  onDismiss,
}: {
  item: NotificationItem;
  onDismiss: (id: string) => void;
}) {
  const tone = item.tone ?? "info";
  const dismiss = useCallback(() => onDismiss(item.id), [item.id, onDismiss]);
  const autoDismiss = usePausableAutoDismiss(
    item.durationMs === undefined
      ? notificationDurationMs(tone)
      : item.durationMs,
    dismiss,
  );
  const pointerPaused = useRef(false);
  const focusPaused = useRef(false);

  const pauseForPointer = () => {
    pointerPaused.current = true;
    autoDismiss.pause();
  };

  const resumeAfterPointer = () => {
    pointerPaused.current = false;
    if (!focusPaused.current) autoDismiss.resume();
  };

  const pauseForFocus = () => {
    focusPaused.current = true;
    autoDismiss.pause();
  };

  const handleBlur = (event: FocusEvent<HTMLElement>) => {
    if (!event.currentTarget.contains(event.relatedTarget)) {
      focusPaused.current = false;
      if (!pointerPaused.current) autoDismiss.resume();
    }
  };

  return (
    <article
      className={`notification notification-${tone}`}
      onBlurCapture={handleBlur}
      onFocusCapture={pauseForFocus}
      onMouseEnter={pauseForPointer}
      onMouseLeave={resumeAfterPointer}
      role={tone === "danger" ? "alert" : "status"}
    >
      <span className="notification-icon" aria-hidden="true">
        <NotificationIcon tone={tone} />
      </span>
      <span className="notification-copy">
        <strong>{item.title}</strong>
        {item.description && <span>{item.description}</span>}
      </span>
      <button
        aria-label={`关闭通知：${item.title}`}
        className="notification-close"
        onClick={dismiss}
        type="button"
      >
        <X size={15} />
      </button>
    </article>
  );
}

function NotificationIcon({ tone }: { tone: NotificationTone }) {
  if (tone === "danger") return <AlertCircle size={18} />;
  if (tone === "warning") return <AlertTriangle size={18} />;
  if (tone === "success") return <CheckCircle2 size={18} />;
  return <Info size={18} />;
}

export function notificationDurationMs(tone: NotificationTone) {
  if (tone === "warning") return 6_000;
  if (tone === "danger") return 8_000;
  return 4_000;
}
