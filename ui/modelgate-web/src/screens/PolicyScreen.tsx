import { EmptyState } from '../components/ErrorBox';

export function PolicyScreen() {
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
