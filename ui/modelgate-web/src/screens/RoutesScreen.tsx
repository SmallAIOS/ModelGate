import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';

import { api } from '../api';
import type { Model, Route } from '../api';
import { EmptyState, ErrorBox } from '../components/ErrorBox';
import { useSetRoute } from '../hooks/mutations';

export function RoutesScreen() {
  const routes = useQuery<Route[]>({
    queryKey: ['routes'],
    queryFn: () => api.listRoutes(),
  });
  const models = useQuery<Model[]>({
    queryKey: ['models'],
    queryFn: () => api.listModels(),
  });
  const setRoute = useSetRoute();
  const [formOpen, setFormOpen] = useState(false);

  return (
    <section className="panel">
      <header className="panel__header">
        <h2>Routes</h2>
        <button
          type="button"
          className="btn btn--primary"
          onClick={() => setFormOpen((v) => !v)}
          disabled={!models.data || models.data.length === 0}
        >
          {formOpen ? 'Cancel' : 'Set route'}
        </button>
      </header>

      {formOpen && (
        <SetRouteForm
          models={models.data ?? []}
          busy={setRoute.isPending}
          onSubmit={(model, endpoint) =>
            setRoute.mutate({ model, endpoint }, { onSuccess: () => setFormOpen(false) })
          }
        />
      )}

      {routes.isError ? (
        <ErrorBox title="Could not list routes" error={routes.error} />
      ) : !routes.data ? (
        <p className="muted">Loading…</p>
      ) : routes.data.length === 0 ? (
        <EmptyState label="No routes configured" />
      ) : (
        <table className="table">
          <thead>
            <tr>
              <th>Model</th>
              <th>Endpoint</th>
              <th>Active</th>
              <th>Requests</th>
            </tr>
          </thead>
          <tbody>
            {routes.data.map((r) => (
              <tr key={`${r.model}-${r.endpoint}`}>
                <td>{r.model}</td>
                <td>{r.endpoint}</td>
                <td>{r.active ? 'yes' : 'no'}</td>
                <td>{r.request_count}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}

function SetRouteForm({
  models,
  busy,
  onSubmit,
}: {
  models: Model[];
  busy: boolean;
  onSubmit: (model: string, endpoint: string) => void;
}) {
  const [model, setModel] = useState<string>(models[0]?.name ?? '');
  const [endpoint, setEndpoint] = useState<string>('');

  const valid = model.length > 0 && endpoint.trim().length > 0;

  return (
    <form
      className="form"
      onSubmit={(e) => {
        e.preventDefault();
        if (!valid) return;
        onSubmit(model, endpoint.trim());
      }}
    >
      <label className="form__row">
        <span>Model</span>
        <select value={model} onChange={(e) => setModel(e.target.value)} disabled={busy}>
          {models.map((m) => (
            <option key={m.name} value={m.name}>
              {m.name}
            </option>
          ))}
        </select>
      </label>
      <label className="form__row">
        <span>Endpoint</span>
        <input
          type="text"
          value={endpoint}
          onChange={(e) => setEndpoint(e.target.value)}
          placeholder="/v1/chat/completions"
          disabled={busy}
        />
      </label>
      <div className="form__actions">
        <button type="submit" className="btn btn--primary" disabled={!valid || busy}>
          {busy ? 'Saving…' : 'Save route'}
        </button>
      </div>
    </form>
  );
}
