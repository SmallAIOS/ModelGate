import { useQuery } from '@tanstack/react-query';

import { api } from '../api';
import type { Route } from '../api';
import { EmptyState, ErrorBox } from '../components/ErrorBox';

export function RoutesScreen() {
  const routes = useQuery<Route[]>({
    queryKey: ['routes'],
    queryFn: () => api.listRoutes(),
  });

  if (routes.isError) return <ErrorBox title="Could not list routes" error={routes.error} />;
  if (!routes.data) return <p className="muted">Loading…</p>;
  if (routes.data.length === 0) return <EmptyState label="No routes configured" />;

  return (
    <section className="panel">
      <h2>Routes</h2>
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
    </section>
  );
}
