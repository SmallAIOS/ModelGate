import { useRef, useState } from 'react';

import { useRegisterModel } from '../hooks/mutations';
import { humanBytes } from '../utils';

export function RegisterModelDialog({
  open,
  onClose,
}: {
  open: boolean;
  onClose: () => void;
}) {
  const [file, setFile] = useState<File | null>(null);
  const [progress, setProgress] = useState<{ sent: number; total: number } | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const register = useRegisterModel();

  if (!open) return null;

  const submit = () => {
    if (!file) return;
    setProgress({ sent: 0, total: file.size });
    register.mutate(
      {
        file,
        onProgress: (sent, total) => setProgress({ sent, total }),
      },
      {
        onSuccess: () => {
          reset();
          onClose();
        },
        onError: () => {
          // Toast is shown by the hook; keep the dialog open so the
          // operator can pick a different file or retry.
          setProgress(null);
        },
      },
    );
  };

  const reset = () => {
    setFile(null);
    setProgress(null);
    if (fileInputRef.current) fileInputRef.current.value = '';
  };

  const cancel = () => {
    if (register.isPending) return;
    reset();
    onClose();
  };

  return (
    <div
      className="dialog__backdrop"
      onClick={(e) => {
        if (e.target === e.currentTarget) cancel();
      }}
    >
      <div className="dialog" role="dialog" aria-modal="true" aria-labelledby="register-title">
        <h3 id="register-title" className="dialog__title">
          Register model
        </h3>
        <div className="dialog__body">
          <p>
            Upload a model file (.onnx, .gguf, …). The model name is derived from
            the filename.
          </p>
          <input
            ref={fileInputRef}
            type="file"
            onChange={(e) => setFile(e.target.files?.[0] ?? null)}
            disabled={register.isPending}
          />
          {file && (
            <p className="muted" style={{ marginTop: 'var(--space-2)' }}>
              Selected: <code>{file.name}</code> ({humanBytes(file.size)})
            </p>
          )}
          {progress && (
            <ProgressBar sent={progress.sent} total={progress.total} />
          )}
        </div>
        <div className="dialog__actions">
          <button
            type="button"
            className="btn"
            onClick={cancel}
            disabled={register.isPending}
          >
            Cancel
          </button>
          <button
            type="button"
            className="btn btn--primary"
            onClick={submit}
            disabled={!file || register.isPending}
          >
            {register.isPending ? 'Uploading…' : 'Upload'}
          </button>
        </div>
      </div>
    </div>
  );
}

function ProgressBar({ sent, total }: { sent: number; total: number }) {
  const pct = total === 0 ? 0 : Math.min(100, Math.round((sent / total) * 100));
  return (
    <div style={{ marginTop: 'var(--space-2)' }}>
      <div
        className="progress"
        role="progressbar"
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={pct}
      >
        <div className="progress__fill" style={{ width: `${pct}%` }} />
      </div>
      <p className="muted" style={{ marginTop: '4px', fontSize: 'var(--fs-xs)' }}>
        {humanBytes(sent)} / {humanBytes(total)} ({pct}%)
      </p>
    </div>
  );
}
