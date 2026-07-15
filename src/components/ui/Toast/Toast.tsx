import { createContext, useContext, useMemo, useState, type ReactNode } from "react";
import { CheckCircle2, AlertCircle } from "../icons";

export type ToastItem = {
  id: string;
  title: string;
  description?: string;
  tone?: "success" | "danger" | "info";
};

type ToastViewportProps = {
  items: ToastItem[];
  onDismiss: (id: string) => void;
};

type ToastContextValue = {
  pushToast: (toast: Omit<ToastItem, "id">) => void;
};

const ToastContext = createContext<ToastContextValue | null>(null);

export function ToastProvider({ children }: { children: ReactNode }) {
  const [items, setItems] = useState<ToastItem[]>([]);
  const value = useMemo<ToastContextValue>(
    () => ({
      pushToast: (toast) => {
        const item = { id: crypto.randomUUID(), ...toast };
        setItems((current) => [item, ...current].slice(0, 3));
      },
    }),
    [],
  );

  return (
    <ToastContext.Provider value={value}>
      {children}
      <ToastViewport
        items={items}
        onDismiss={(id) => setItems((current) => current.filter((item) => item.id !== id))}
      />
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
    <div className="toast-viewport" aria-live="polite">
      {items.map((item) => (
        <button
          className={`toast toast-${item.tone ?? "info"}`}
          key={item.id}
          onClick={() => onDismiss(item.id)}
          type="button"
        >
          {item.tone === "danger" ? <AlertCircle size={18} /> : <CheckCircle2 size={18} />}
          <span>
            <strong>{item.title}</strong>
            {item.description && <em>{item.description}</em>}
          </span>
        </button>
      ))}
    </div>
  );
}

