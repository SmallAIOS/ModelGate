import { useQuery } from '@tanstack/react-query';

import { api } from '../api';
import type { HealthStatus, Model } from '../api';
import { ErrorBox } from '../components/ErrorBox';

export function OverviewScreen() {
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
        <dd>{models.data ? models.data.length : (health.data?.model_count ?? '…')}</dd>
      </dl>
    </section>
  );
}
