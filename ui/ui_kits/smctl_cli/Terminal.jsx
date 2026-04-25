/* global React */
const { useState, useEffect, useRef } = React;

// ============================================================================
// smctl CLI — UI kit components
// A high-fidelity recreation of the smctl terminal experience.
// Pure-cosmetic components: they render output; they don't really shell out.
// ============================================================================

// --- Atoms -----------------------------------------------------------------

const Gray   = ({ children }) => <span style={{ color: 'var(--term-muted)' }}>{children}</span>;
const Blue   = ({ children }) => <span style={{ color: 'var(--term-ion)' }}>{children}</span>;
const Green  = ({ children }) => <span style={{ color: 'var(--term-ok)' }}>{children}</span>;
const Red    = ({ children }) => <span style={{ color: 'var(--term-err)' }}>{children}</span>;
const Amber  = ({ children }) => <span style={{ color: 'var(--term-warn)' }}>{children}</span>;
const Bold   = ({ children }) => <span style={{ color: 'var(--term-fg)', fontWeight: 600 }}>{children}</span>;
const Dim    = ({ children }) => <span style={{ color: 'var(--term-dim)' }}>{children}</span>;

// A terminal line: prompt or output. Children are pre-formatted.
const Line = ({ children }) => (
  <div style={{ whiteSpace: 'pre', minHeight: '1.55em' }}>{children}</div>
);

const Spacer = () => <div style={{ height: '0.55em' }} />;

// A user-typed prompt line (what the user entered).
const Prompt = ({ user = 'e@airgap', cwd = '~/work/smallaios', children }) => (
  <Line>
    <Gray>{user}</Gray>{' '}<Blue>{cwd}</Blue>{' '}<Gray>$</Gray>{' '}
    <span style={{ color: 'var(--term-fg)' }}>{children}</span>
  </Line>
);

// Blinking caret shown on the current prompt.
const Caret = () => (
  <span style={{
    display: 'inline-block',
    width: '0.55em',
    height: '1.05em',
    background: 'var(--term-ok)',
    verticalAlign: '-0.2em',
    animation: 'smctl-blink 1s steps(2) infinite',
    marginLeft: '1px',
  }} />
);

// --- Output blocks ---------------------------------------------------------

// `smctl workspace status` output
function WorkspaceStatus({ repos }) {
  return (
    <>
      {repos.map((r, i) => {
        const isLast = i === repos.length - 1;
        const branch = String(r.branch).padEnd(10, ' ');
        const name = String(r.name).padEnd(11, ' ');
        return (
          <Line key={r.name}>
            <Dim>{isLast ? '└─' : '├─'}</Dim>{' '}
            {name}<Blue>{branch}</Blue>{'  '}
            {r.clean
              ? <Green>✓ clean  </Green>
              : <Red>✗ dirty  </Red>}
            <Gray>
              {r.clean
                ? `+${r.ahead}/-${r.behind}`
                : `${r.dirty} file${r.dirty === 1 ? '' : 's'}`}
            </Gray>
          </Line>
        );
      })}
    </>
  );
}

// `smctl spec ff <name>` validation output
function SpecValidation({ rows }) {
  return (
    <>
      {rows.map((row) => (
        <Line key={row.file}>
          {row.ok
            ? <Green>✓</Green>
            : <Amber>!</Amber>}
          {'  '}
          <span style={{ color: 'var(--term-fg)' }}>{row.file.padEnd(14, ' ')}</span>
          {'  '}
          <Gray>{row.note}</Gray>
        </Line>
      ))}
    </>
  );
}

// `smctl build --parallel` output — a simple progress block
function BuildOutput({ steps }) {
  return (
    <>
      {steps.map((s) => (
        <Line key={s.name}>
          {s.state === 'ok'      && <Green>✓</Green>}
          {s.state === 'run'     && <Amber>·</Amber>}
          {s.state === 'err'     && <Red>✗</Red>}
          {s.state === 'queue'   && <Dim>○</Dim>}
          {'  '}
          <span style={{ color: 'var(--term-fg)' }}>{s.name.padEnd(22, ' ')}</span>
          {'  '}
          <Gray>{s.note}</Gray>
        </Line>
      ))}
    </>
  );
}

// `smctl gate status` key/value block
function GateStatus({ items }) {
  return (
    <>
      {items.map((it) => (
        <Line key={it.k}>
          <Gray>{it.k.padEnd(18, ' ')}</Gray>{'  '}
          {it.tone === 'ok'   && <Green>{it.v}</Green>}
          {it.tone === 'warn' && <Amber>{it.v}</Amber>}
          {it.tone === 'err'  && <Red>{it.v}</Red>}
          {!it.tone           && <span style={{ color: 'var(--term-fg)' }}>{it.v}</span>}
        </Line>
      ))}
    </>
  );
}

// A destructive confirmation prompt: prints a [y/N] line and accepts 'y'/'n'
function ConfirmLine({ text, onAnswer, answered }) {
  return (
    <Line>
      <Bold>{text}</Bold>{' '}<Gray>[y/N]</Gray>{' '}
      {answered
        ? <span style={{ color: 'var(--term-fg)' }}>{answered}</span>
        : (
          <>
            <button
              onClick={() => onAnswer('y')}
              style={termBtnStyle}
            >y</button>
            <button
              onClick={() => onAnswer('n')}
              style={{ ...termBtnStyle, marginLeft: 6 }}
            >N</button>
          </>
        )}
    </Line>
  );
}

const termBtnStyle = {
  font: 'inherit',
  color: 'var(--term-fg)',
  background: 'var(--term-bg-row)',
  border: '1px solid var(--term-border)',
  borderRadius: 3,
  padding: '1px 8px',
  cursor: 'pointer',
  fontFamily: 'inherit',
};

Object.assign(window, {
  Gray, Blue, Green, Red, Amber, Bold, Dim,
  Line, Spacer, Prompt, Caret,
  WorkspaceStatus, SpecValidation, BuildOutput, GateStatus, ConfirmLine,
});
