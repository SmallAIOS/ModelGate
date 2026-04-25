import { GateApiError } from '../api';

export function ErrorBox({ title, error }: { title: string; error: unknown }) {
  const message =
    error instanceof GateApiError
      ? `${error.kind}: ${error.message}`
      : error instanceof Error
        ? error.message
        : String(error);
  return (
    <section className="panel panel--error">
      <h3>{title}</h3>
      <p className="muted">{message}</p>
    </section>
  );
}

export function EmptyState({ label }: { label: string }) {
  return (
    <p className="muted">
      <em>{label}</em>
    </p>
  );
}
