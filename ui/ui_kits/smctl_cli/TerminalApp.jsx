/* global React, Prompt, Line, Spacer, Caret,
   WorkspaceStatus, SpecValidation, BuildOutput, GateStatus, ConfirmLine,
   Gray, Blue, Green, Red, Amber, Bold, Dim */
const { useState, useEffect, useRef } = React;

// ============================================================================
// Scenario: the canonical smctl flow
//
// The demo walks the operator through the hottest path of the tool:
//   1. workspace status              → see state of all repos
//   2. spec ff gpu-accel             → validate an in-flight spec
//   3. build --parallel --test       → build + test in dep order
//   4. feat gpu-accel                → alias: create feature + worktree
//   5. gate status                   → check ModelGate health
//   6. done gpu-accel                → alias: finish feature (confirm)
//
// Each step is a "scene" — the user can click a step button on the right to
// jump around, or use ↵ to advance. No real commands run.
// ============================================================================

const SCENES = [
  {
    id: 'status',
    cmd: 'smctl workspace status',
    render: () => (
      <>
        <WorkspaceStatus repos={[
          { name: 'SmallAIOS',  branch: 'develop', clean: true,  ahead: 3, behind: 0 },
          { name: 'ModelGate',  branch: 'develop', clean: false, dirty: 2 },
          { name: 'Runes',      branch: 'main',    clean: true,  ahead: 0, behind: 0 },
        ]} />
        <Spacer />
        <Line><Dim>last sync  </Dim><Gray>14:02:31Z · 6 repos · 2 worktrees active</Gray></Line>
      </>
    ),
  },
  {
    id: 'spec-ff',
    cmd: 'smctl spec ff gpu-accel',
    render: () => (
      <>
        <Line><Gray>→ reading </Gray><Blue>openspec/changes/gpu-accel/</Blue></Line>
        <SpecValidation rows={[
          { file: 'proposal.md', ok: false, note: 'missing "Impact" section'       },
          { file: 'design.md',   ok: true,  note: '12 decisions · 3 open questions' },
          { file: 'tasks.md',    ok: true,  note: '14/28 tasks complete'            },
          { file: 'specs/*.md',  ok: true,  note: '3 files · 240 requirements'      },
        ]} />
        <Spacer />
        <Line><Amber>1 issue.</Amber>{' '}<Gray>Run </Gray><Bold>smctl spec validate gpu-accel --fix</Bold><Gray> to scaffold.</Gray></Line>
      </>
    ),
  },
  {
    id: 'build',
    cmd: 'smctl build --parallel --test',
    render: () => (
      <>
        <Line><Gray>→ resolving dependency graph · </Gray><Blue>6 crates</Blue><Gray> · parallel=4</Gray></Line>
        <BuildOutput steps={[
          { name: 'smallaios-core',       state: 'ok',  note: 'compiled in 8.4s  · 0 warnings' },
          { name: 'smallaios-net',        state: 'ok',  note: 'compiled in 4.1s  · 0 warnings' },
          { name: 'smallaios-onnx',       state: 'ok',  note: 'compiled in 12.2s · 0 warnings' },
          { name: 'modelgate-runtime',    state: 'run', note: 'compiling · 00:14'              },
          { name: 'modelgate-cedar',      state: 'queue', note: 'waiting on modelgate-runtime' },
          { name: 'smctl',                state: 'queue', note: 'waiting on modelgate-runtime' },
        ]} />
        <Spacer />
        <Line><Gray>tests </Gray><Green>passed 4,143 / 4,143</Green><Gray> · coverage </Gray><Green>MC/DC 98.2%</Green></Line>
      </>
    ),
  },
  {
    id: 'feat',
    cmd: 'smctl feat gpu-accel',
    render: () => (
      <>
        <Line><Gray>→ flow feature start </Gray><Blue>gpu-accel</Blue></Line>
        <Line><Dim>├─</Dim> SmallAIOS   <Green>feature/gpu-accel</Green>{' '}<Gray>created from develop</Gray></Line>
        <Line><Dim>├─</Dim> ModelGate   <Green>feature/gpu-accel</Green>{' '}<Gray>created from develop</Gray></Line>
        <Line><Dim>└─</Dim> Runes       <Gray>skipped · no matching flow policy</Gray></Line>
        <Spacer />
        <Line><Gray>→ worktree add </Gray><Blue>gpu-accel</Blue></Line>
        <Line><Dim>└─</Dim>{' '}<Green>/airgap/wt/gpu-accel</Green>{' '}<Gray>(3 linked worktrees · 230µs)</Gray></Line>
        <Spacer />
        <Line><Green>ready.</Green>{' '}<Gray>cd into </Gray><Bold>/airgap/wt/gpu-accel</Bold><Gray> to begin.</Gray></Line>
      </>
    ),
  },
  {
    id: 'gate',
    cmd: 'smctl gate status',
    render: () => (
      <>
        <GateStatus items={[
          { k: 'endpoint',          v: 'unix:///run/modelgate.sock',  tone: null  },
          { k: 'version',           v: 'v0.1.0-alpha',                 tone: null  },
          { k: 'uptime',            v: '17d 04:21:08',                 tone: null  },
          { k: 'models',            v: '12 loaded · 0 failing',        tone: 'ok'  },
          { k: 'policy',            v: 'verified · ML-DSA-65',         tone: 'ok'  },
          { k: 'boundaries',        v: '29 proofs · all present',      tone: 'ok'  },
          { k: 'tls',               v: 'X25519 + ML-KEM-768',          tone: 'ok'  },
          { k: 'inference latency', v: 'p50 1.2ms · p99 4.8ms',        tone: null  },
          { k: 'cedar analyze',     v: '2 properties unsolved',        tone: 'warn'},
        ]} />
      </>
    ),
  },
  {
    id: 'done',
    cmd: 'smctl done gpu-accel',
    render: ({ confirm, setConfirm }) => (
      <>
        <Line><Gray>→ reviewing </Gray><Blue>feature/gpu-accel</Blue></Line>
        <Line><Dim>├─</Dim> SmallAIOS   <Green>4 commits</Green>{' '}<Gray>since develop</Gray></Line>
        <Line><Dim>├─</Dim> ModelGate   <Green>2 commits</Green>{' '}<Gray>since develop</Gray></Line>
        <Line><Dim>└─</Dim> worktree    <Gray>/airgap/wt/gpu-accel · clean</Gray></Line>
        <Spacer />
        <ConfirmLine
          text="Merge feature/gpu-accel into develop in 2 repos and remove worktree?"
          answered={confirm}
          onAnswer={setConfirm}
        />
        {confirm === 'y' && (
          <>
            <Line><Green>✓</Green>{' '}SmallAIOS   <Gray>merged · fast-forward</Gray></Line>
            <Line><Green>✓</Green>{' '}ModelGate   <Gray>merged · 1 conflict auto-resolved</Gray></Line>
            <Line><Green>✓</Green>{' '}worktree    <Gray>/airgap/wt/gpu-accel removed</Gray></Line>
            <Spacer />
            <Line><Green>done.</Green>{' '}<Gray>archive this spec with </Gray><Bold>smctl spec archive gpu-accel</Bold></Line>
          </>
        )}
        {confirm === 'n' && <Line><Red>aborted.</Red>{' '}<Gray>no changes made.</Gray></Line>}
      </>
    ),
  },
];

function TerminalApp() {
  const [sceneIdx, setSceneIdx] = useState(() => {
    const n = Number(localStorage.getItem('smctl-scene'));
    return Number.isFinite(n) && n >= 0 && n < SCENES.length ? n : 0;
  });
  const [confirm, setConfirm] = useState(null);
  const bodyRef = useRef(null);

  useEffect(() => {
    localStorage.setItem('smctl-scene', String(sceneIdx));
    setConfirm(null);
    // Scroll the terminal body to bottom when scene changes.
    const el = bodyRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [sceneIdx]);

  useEffect(() => {
    function onKey(e) {
      if (e.target && (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA')) return;
      if (e.key === 'Enter' || e.key === 'ArrowRight' || e.key === 'j') {
        setSceneIdx((i) => Math.min(SCENES.length - 1, i + 1));
      } else if (e.key === 'ArrowLeft' || e.key === 'k') {
        setSceneIdx((i) => Math.max(0, i - 1));
      } else if (/^[1-9]$/.test(e.key)) {
        const n = Number(e.key) - 1;
        if (n < SCENES.length) setSceneIdx(n);
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  // Render every scene up to and including the current one — a scrolling history.
  const shown = SCENES.slice(0, sceneIdx + 1);
  const isLastInteractive = SCENES[sceneIdx].id === 'done' && confirm === null;

  return (
    <div style={shellStyle}>
      {/* Titlebar — macOS-style traffic lights on a dark bar */}
      <div style={titlebarStyle}>
        <div style={{ display: 'flex', gap: 6 }}>
          <span style={{ ...tlDot, background: '#ff5f57' }} />
          <span style={{ ...tlDot, background: '#febc2e' }} />
          <span style={{ ...tlDot, background: '#28c840' }} />
        </div>
        <div style={titleTextStyle}>e@airgap · smctl · /airgap/work/smallaios</div>
        <div style={{ width: 58 }} />
      </div>

      <div style={bodyWrapStyle}>
        {/* Terminal body */}
        <div ref={bodyRef} style={bodyStyle}>
          {shown.map((s, i) => (
            <React.Fragment key={s.id}>
              <Prompt>{s.cmd}</Prompt>
              {s.render({ confirm, setConfirm })}
              {i < shown.length - 1 && <Spacer />}
            </React.Fragment>
          ))}
          {/* trailing prompt with blinking caret */}
          {!isLastInteractive && (
            <>
              <Spacer />
              <Line>
                <Gray>e@airgap</Gray>{' '}<Blue>~/work/smallaios</Blue>{' '}<Gray>$</Gray><Caret />
              </Line>
            </>
          )}
        </div>

        {/* Right rail — scene picker */}
        <aside style={railStyle}>
          <div style={railHeaderStyle}>scenes</div>
          {SCENES.map((s, i) => (
            <button
              key={s.id}
              onClick={() => setSceneIdx(i)}
              style={{
                ...railItemStyle,
                ...(i === sceneIdx ? railItemActive : null),
              }}
            >
              <span style={railNumStyle}>{String(i + 1).padStart(2, '0')}</span>
              <span style={{ color: i === sceneIdx ? 'var(--term-fg)' : 'var(--term-dim)' }}>$</span>
              <span style={railCmdStyle}>{s.cmd.replace('smctl ', '')}</span>
            </button>
          ))}
          <div style={railFootStyle}>
            <div><span style={kbdStyle}>↵</span> next</div>
            <div><span style={kbdStyle}>←</span> / <span style={kbdStyle}>→</span> step</div>
            <div><span style={kbdStyle}>1–6</span> jump</div>
          </div>
        </aside>
      </div>

      {/* Status line — vim-style, pinned to bottom of the shell */}
      <div style={statusLineStyle}>
        <span><Bold>workspace</Bold> smallaios</span>
        <span>·</span>
        <span>branch <Blue>feature/gpu-accel</Blue></span>
        <span>·</span>
        <span>scene {sceneIdx + 1}/{SCENES.length}</span>
        <span style={{ marginLeft: 'auto' }}>last sync 14:02:31Z</span>
      </div>
    </div>
  );
}

// --- Styles ----------------------------------------------------------------

const shellStyle = {
  width: '100%',
  maxWidth: 1200,
  height: 720,
  margin: '40px auto',
  display: 'flex',
  flexDirection: 'column',
  background: 'var(--term-bg)',
  border: '1px solid var(--term-border-strong)',
  borderRadius: 8,
  overflow: 'hidden',
  boxShadow: '0 0 0 1px rgba(0,0,0,0.5), 0 24px 48px rgba(0,0,0,0.4)',
  fontFamily: "'JetBrains Mono', ui-monospace, SFMono-Regular, Menlo, monospace",
  fontSize: 13,
  lineHeight: 1.55,
};
const titlebarStyle = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  padding: '0 12px',
  height: 30,
  background: '#1B1D22',
  borderBottom: '1px solid #000',
  flexShrink: 0,
};
const tlDot = {
  width: 11,
  height: 11,
  borderRadius: '50%',
  display: 'block',
};
const titleTextStyle = {
  fontSize: 11,
  color: '#8A8F99',
  letterSpacing: '0.04em',
  fontFamily: "'IBM Plex Sans', sans-serif",
};
const bodyWrapStyle = {
  display: 'grid',
  gridTemplateColumns: '1fr 260px',
  flex: 1,
  minHeight: 0,
};
const bodyStyle = {
  padding: '16px 20px',
  color: 'var(--term-fg)',
  overflowY: 'auto',
  background: 'var(--term-bg)',
};
const railStyle = {
  borderLeft: '1px solid var(--term-border)',
  background: '#101216',
  padding: '12px 8px',
  display: 'flex',
  flexDirection: 'column',
  gap: 2,
  fontFamily: "'JetBrains Mono', monospace",
  fontSize: 12,
  overflowY: 'auto',
};
const railHeaderStyle = {
  color: 'var(--term-dim)',
  fontSize: 10,
  letterSpacing: '0.1em',
  textTransform: 'uppercase',
  padding: '4px 8px 10px',
  borderBottom: '1px solid var(--term-border)',
  marginBottom: 6,
  fontFamily: "'IBM Plex Sans', sans-serif",
};
const railItemStyle = {
  display: 'flex',
  alignItems: 'center',
  gap: 8,
  padding: '6px 8px',
  background: 'transparent',
  border: 'none',
  textAlign: 'left',
  cursor: 'pointer',
  borderRadius: 3,
  color: 'var(--term-muted)',
  font: 'inherit',
};
const railItemActive = {
  background: 'var(--term-bg-row)',
  color: 'var(--term-fg)',
  boxShadow: 'inset 2px 0 0 var(--term-ion)',
};
const railNumStyle = {
  color: 'var(--term-dim)',
  fontSize: 10,
};
const railCmdStyle = {
  whiteSpace: 'nowrap',
  overflow: 'hidden',
  textOverflow: 'ellipsis',
};
const railFootStyle = {
  marginTop: 'auto',
  padding: '12px 8px 4px',
  borderTop: '1px solid var(--term-border)',
  display: 'flex',
  flexDirection: 'column',
  gap: 6,
  color: 'var(--term-dim)',
  fontSize: 11,
  fontFamily: "'IBM Plex Sans', sans-serif",
};
const kbdStyle = {
  display: 'inline-block',
  padding: '1px 5px',
  border: '1px solid var(--term-border)',
  borderRadius: 2,
  fontFamily: "'JetBrains Mono', monospace",
  fontSize: 10,
  color: 'var(--term-muted)',
  background: 'var(--term-bg-row)',
};
const statusLineStyle = {
  display: 'flex',
  alignItems: 'center',
  gap: 8,
  height: 24,
  padding: '0 12px',
  background: '#0D0F13',
  color: 'var(--term-muted)',
  fontSize: 11,
  fontFamily: "'JetBrains Mono', monospace",
  borderTop: '1px solid var(--term-border)',
  flexShrink: 0,
};

Object.assign(window, { TerminalApp });
