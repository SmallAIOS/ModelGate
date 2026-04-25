export function TerminalScreen() {
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
