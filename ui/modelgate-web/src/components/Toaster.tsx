import { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';
import type { ReactNode } from 'react';

export type Toast = {
  kind: 'success' | 'error' | 'info';
  message: string;
  /** Optional secondary line — usually a remediation clause. */
  detail?: string;
};

type Entry = Toast & { id: number };

type ToastContextValue = {
  show: (toast: Toast) => void;
};

const ToastContext = createContext<ToastContextValue | undefined>(undefined);

const MAX_VISIBLE = 4;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [entries, setEntries] = useState<Entry[]>([]);

  const show = useCallback((toast: Toast) => {
    const id = Date.now() + Math.random();
    setEntries((prev) => {
      const next = [...prev, { ...toast, id }];
      // Cap at MAX_VISIBLE. Drop the oldest when we overflow.
      return next.length > MAX_VISIBLE ? next.slice(next.length - MAX_VISIBLE) : next;
    });
    const dismissAfter = toast.kind === 'error' ? 8000 : 5000;
    window.setTimeout(() => {
      setEntries((prev) => prev.filter((e) => e.id !== id));
    }, dismissAfter);
  }, []);

  const dismiss = useCallback((id: number) => {
    setEntries((prev) => prev.filter((e) => e.id !== id));
  }, []);

  const value = useMemo(() => ({ show }), [show]);

  return (
    <ToastContext.Provider value={value}>
      {children}
      <Toaster entries={entries} onDismiss={dismiss} />
    </ToastContext.Provider>
  );
}

export function useToast(): ToastContextValue {
  const ctx = useContext(ToastContext);
  if (!ctx) {
    throw new Error('useToast must be used inside <ToastProvider>');
  }
  return ctx;
}

function Toaster({
  entries,
  onDismiss,
}: {
  entries: Entry[];
  onDismiss: (id: number) => void;
}) {
  // aria-live="polite" so screen readers announce new toasts without
  // stealing focus.
  return (
    <div className="toaster" role="status" aria-live="polite">
      {entries.map((entry) => (
        <ToastItem key={entry.id} entry={entry} onDismiss={() => onDismiss(entry.id)} />
      ))}
    </div>
  );
}

function ToastItem({ entry, onDismiss }: { entry: Entry; onDismiss: () => void }) {
  // Local mount animation hook: the .toast--visible class fades in once
  // the element is in the DOM, and clicking dismisses early.
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    const id = window.requestAnimationFrame(() => setVisible(true));
    return () => window.cancelAnimationFrame(id);
  }, []);
  return (
    <button
      type="button"
      className={`toast toast--${entry.kind}${visible ? ' toast--visible' : ''}`}
      onClick={onDismiss}
    >
      <span className="toast__message">{entry.message}</span>
      {entry.detail && <span className="toast__detail">{entry.detail}</span>}
    </button>
  );
}
