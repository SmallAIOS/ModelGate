import { useEffect, useState } from 'react';
import { useQuery } from '@tanstack/react-query';

import { api, GateApiError } from './api';
import type { HealthStatus, Model, Route } from './api';

// --- Hash router ---

type Tab = 'overview' | 'models' | 'policy' | 'terminal';

function parseTab(hash: string): Tab {
  const cleaned = hash.replace(/^#\/?/, '');
  if (cleaned === 'models' || cleaned === 'policy' || cleaned === 'terminal') {
    return cleaned;
  }
  return 'overview';
}

function useHashRoute(): [Tab, (t: Tab) => void] {
  const [tab, setTab] = useState<Tab>(() => parseTab(window.location.hash));
  useEffect(() => {
    const onChange = () => setTab(parseTab(window.location.hash));
    window.addEventListener('hashchange', onChange);
    return () => window.removeEventListener('hashchange', onChange);
  }, []);
  const navigate = (t: Tab) => {
    window.location.hash = `#/${t}`;
    setTab(t);
  };
  return [tab, navigate];
}

// --- Error box ---

function ErrorBox({ title, error }: { title: string; error: unknown }) {
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

function EmptyState({ label }: { label: string }) {
  return (
    <p className="muted">
      <em>{label}</em>
    </p>
  );
}

// --- Screens ---

function OverviewScreen() {
  const health = useQuery<HealthStatus>({
    queryKey: ['health'],
    queryFn: () => api.health(),
  });
  const models = useQuery<Model[]>({
    queryKey: ['models'],
    queryFn: () => api.listModels(),
  });

  if (health.isError) return <ErrorBox title="ModelGate unreachable" error={health.error} />;

  return (
    <section className="panel">
      <h2>Overview</h2>
      <dl className="kv">
        <dt>Status</dt>
        <dd>{health.data ? health.data.status : '…'}</dd>
        <dt>Version</dt>
        <dd>{health.data ? health.data.version : '…'}</dd>
        <dt>Uptime</dt>
        <dd>{health.data ? `${health.data.uptime_secs}s` : '…'}</dd>
        <dt>Models loaded</dt>
        <dd>{models.data ? models.data.length : health.data?.model_count ?? '…'}</dd>
      </dl>
    </section>
  );
}

function ModelsScreen() {
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

function RoutesScreen() {
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

function PolicyScreen() {
  return (
    <section className="panel">
      <h2>Policy</h2>
      <EmptyState label="Cedar policy viewer is not yet available" />
      <p className="muted">
        The ModelGate policy endpoint is not exposed. Run{' '}
        <code>smctl gate logs</code> for now.
      </p>
    </section>
  );
}

function TerminalScreen() {
  return (
    <section className="panel">
      <h2>Terminal</h2>
      <p className="muted">
        An embedded terminal is deferred to a follow-up change. Open a real
        terminal and run <code>smctl gate logs --follow</code>.
      </p>
    </section>
  );
}

// --- Shell ---

function App() {
  const [tab, navigate] = useHashRoute();
  return (
    <div className="shell">
      <header className="shell__top">
        <strong>ModelGate</strong>
      </header>
      <nav className="shell__rail">
        {(['overview', 'models', 'policy', 'terminal'] as Tab[]).map((t) => (
          <button
            key={t}
            type="button"
            className={t === tab ? 'rail__item rail__item--active' : 'rail__item'}
            onClick={() => navigate(t)}
          >
            {labelFor(t)}
          </button>
        ))}
      </nav>
      <main className="shell__main">
        {tab === 'overview' && <OverviewScreen />}
        {tab === 'models' && <ModelsScreen />}
        {tab === 'policy' && (
          <>
            <PolicyScreen />
            <RoutesScreen />
          </>
        )}
        {tab === 'terminal' && <TerminalScreen />}
      </main>
    </div>
  );
}

function labelFor(t: Tab): string {
  switch (t) {
    case 'overview':
      return 'Overview';
    case 'models':
      return 'Models';
    case 'policy':
      return 'Policy';
    case 'terminal':
      return 'Terminal';
  }
}

function humanBytes(n: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let v = n;
  let i = 0;
  while (v >= 1024 && i + 1 < units.length) {
    v /= 1024;
    i += 1;
  }
  return i === 0 ? `${n} ${units[0]}` : `${v.toFixed(1)} ${units[i]}`;
}

export default App;
