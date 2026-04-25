import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';

import { api } from '../api';
import type { Model } from '../api';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { EmptyState, ErrorBox } from '../components/ErrorBox';
import { useRemoveModel } from '../hooks/mutations';
import { humanBytes } from '../utils';
import { RegisterModelDialog } from './RegisterModelDialog';

export function ModelsScreen() {
  const models = useQuery<Model[]>({
    queryKey: ['models'],
    queryFn: () => api.listModels(),
  });
  const remove = useRemoveModel();
  const [pendingRemove, setPendingRemove] = useState<string | null>(null);
  const [registerOpen, setRegisterOpen] = useState(false);

  if (models.isError) return <ErrorBox title="Could not list models" error={models.error} />;

  return (
    <section className="panel">
      <header className="panel__header">
        <h2>Models</h2>
        <button
          type="button"
          className="btn btn--primary"
          onClick={() => setRegisterOpen(true)}
        >
          Register model
        </button>
      </header>

      {!models.data ? (
        <p className="muted">Loading…</p>
      ) : models.data.length === 0 ? (
        <EmptyState label="No models registered" />
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>Name</th>
              <th>Format</th>
              <th>Size</th>
              <th>Status</th>
              <th>Registered</th>
              <th aria-label="Actions" />
            </tr>
          </thead>
          <tbody>
            {models.data.map((m) => (
              <tr key={m.name}>
                <td>{m.name}</td>
                <td>{m.format}</td>
                <td>{humanBytes(m.size_bytes)}</td>
                <td>{m.status}</td>
                <td>{m.registered_at}</td>
                <td>
                  <button
                    type="button"
                    className="btn"
                    onClick={() => setPendingRemove(m.name)}
                    disabled={remove.isPending && remove.variables === m.name}
                  >
                    Remove
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      <RegisterModelDialog open={registerOpen} onClose={() => setRegisterOpen(false)} />
      <ConfirmDialog
        open={pendingRemove !== null}
        title="Remove model"
        body={
          <>
            Remove <code>{pendingRemove}</code> from this ModelGate instance? This
            cannot be undone.
          </>
        }
        cliEquivalent={pendingRemove ? `smctl gate models remove ${pendingRemove}` : undefined}
        confirmLabel="Remove model"
        destructive
        busy={remove.isPending}
        onCancel={() => setPendingRemove(null)}
        onConfirm={() => {
          if (!pendingRemove) return;
          remove.mutate(pendingRemove, {
            onSettled: () => setPendingRemove(null),
          });
        }}
      />
    </section>
  );
}
