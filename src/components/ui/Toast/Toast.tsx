import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { createPortal } from "react-dom";
import {
  AlertCircle,
  AlertTriangle,
  CheckCircle2,
  Info,
} from "../icons";
import { usePausableAutoDismiss } from "../feedbackTimer";

export type ToastTone = "success" | "warning" | "danger" | "info";

export type ToastInput = {
  message: string;
  tone?: ToastTone;
  durationMs?: number;
};

export type ToastItem = ToastInput & {
  id: string;
};

type ToastViewportProps = {
  items: ToastItem[];
  onDismiss: (id: string) => void;
};

type ToastContextValue = {
  showToast: (toast: ToastInput) => string;
};

const ToastContext = createContext<ToastContextValue | null>(null);

export function ToastProvider({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<ToastItem[]>([]);
  const dismissToast = useCallback((id: string) => {
    setItems((current) => current.filter((item) => item.id !== id));
  }, []);
  const showToast = useCallback((toast: ToastInput) => {
    const id = crypto.randomUUID();
    setItems((current) => [{ id, ...toast }, ...current].slice(0, 3));
    return id;
  }, []);
  const value = useMemo<ToastContextValue>(() => ({ showToast }), [showToast]);
  const viewport = <ToastViewport items={items} onDismiss={dismissToast} />;

  return (
    <ToastContext.Provider value={value}>
      {children}
      {typeof document === "undefined"
        ? viewport
        : createPortal(viewport, document.body)}
    </ToastContext.Provider>
  );
}

export function useToast() {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within ToastProvider");
  }
  return context;
}

export function ToastViewport({ items, onDismiss }: ToastViewportProps) {
  if (!items.length) return null;

  return (
    <div
      className="toast-viewport"
      aria-label="快捷提示"
      aria-live="polite"
      aria-relevant="additions"
    >
      {items.map((item) => (
        <ToastNotice item={item} key={item.id} onDismiss={onDismiss} />
      ))}
    </div>
  );
}

function ToastNotice({
  item,
  onDismiss,
}: {
  item: ToastItem;
  onDismiss: (id: string) => void;
}) {
  const tone = item.tone ?? "info";
  const dismiss = useCallback(() => onDismiss(item.id), [item.id, onDismiss]);
  const autoDismiss = usePausableAutoDismiss(
    Math.max(1_000, item.durationMs ?? 3_000),
    dismiss,
  );

  return (
    <div
      className={`toast toast-${tone}`}
      onMouseEnter={autoDismiss.pause}
      onMouseLeave={autoDismiss.resume}
      role={tone === "danger" ? "alert" : "status"}
    >
      <span className="toast-icon" aria-hidden="true">
        <ToastIcon tone={tone} />
      </span>
      <span>{item.message}</span>
    </div>
  );
}

function ToastIcon({ tone }: { tone: ToastTone }) {
  if (tone === "danger") return <AlertCircle size={16} />;
  if (tone === "warning") return <AlertTriangle size={16} />;
  if (tone === "success") return <CheckCircle2 size={16} />;
  return <Info size={16} />;
}
