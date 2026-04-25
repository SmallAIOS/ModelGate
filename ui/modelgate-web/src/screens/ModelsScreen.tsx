import { useQuery } from '@tanstack/react-query';

import { api } from '../api';
import type { Model } from '../api';
import { EmptyState, ErrorBox } from '../components/ErrorBox';
import { humanBytes } from '../utils';

export function ModelsScreen() {
  const models = useQuery<Model[]>({
    queryKey: ['models'],
    queryFn: () => api.listModels(),
  });

  if (models.isError) return <ErrorBox title="Could not list models" error={models.error} />;
  if (!models.data) return <p className="muted">Loading…</p>;
  if (models.data.length === 0) return <EmptyState label="No models registered" />;

  return (
    <section className="panel">
      <h2>Models</h2>
      <table className="table">
        <thead>
          <tr>
            <th>Name</th>
            <th>Format</th>
            <th>Size</th>
            <th>Status</th>
            <th>Registered</th>
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
            </tr>
          ))}
        </tbody>
      </table>
    </section>
  );
}
