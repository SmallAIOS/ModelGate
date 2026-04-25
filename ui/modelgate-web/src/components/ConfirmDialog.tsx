import { useEffect, useId, useRef } from 'react';
import type { ReactNode } from 'react';

export type ConfirmDialogProps = {
  open: boolean;
  title: string;
  body: ReactNode;
  /** Equivalent CLI command, rendered as a <code> block. */
  cliEquivalent?: string;
  confirmLabel: string;
  cancelLabel?: string;
  /** When true, renders the confirm button with destructive styling. */
  destructive?: boolean;
  /** When true, the confirm button enters a loading state and is disabled. */
  busy?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
};

export function ConfirmDialog({
  open,
  title,
  body,
  cliEquivalent,
  confirmLabel,
  cancelLabel = 'Cancel',
  destructive = false,
  busy = false,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  const titleId = useId();
  const confirmRef = useRef<HTMLButtonElement>(null);

  // Focus the confirm button on open and bind Escape -> cancel for the
  // duration the dialog is visible.
  useEffect(() => {
    if (!open) return;
    confirmRef.current?.focus();
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !busy) onCancel();
    };
    document.addEventListener('keydown', onKey);
    return () => document.removeEventListener('keydown', onKey);
  }, [open, busy, onCancel]);

  if (!open) return null;

  return (
    <div
      className="dialog__backdrop"
      onClick={(e) => {
        if (e.target === e.currentTarget && !busy) onCancel();
      }}
    >
      <div
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
      >
        <h3 id={titleId} className="dialog__title">
          {title}
        </h3>
        <div className="dialog__body">{body}</div>
        {cliEquivalent && (
          <pre className="dialog__cli">
            <code>{cliEquivalent}</code>
          </pre>
        )}
        <div className="dialog__actions">
          <button type="button" className="btn" onClick={onCancel} disabled={busy}>
            {cancelLabel}
          </button>
          <button
            type="button"
            ref={confirmRef}
            className={`btn ${destructive ? 'btn--destructive' : 'btn--primary'}`}
            onClick={onConfirm}
            disabled={busy}
          >
            {busy ? 'Working…' : confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
