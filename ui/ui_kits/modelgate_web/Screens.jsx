/* global React, Icon */
const { useState } = React;

// ============================================================================
// ModelGate web — page/screen components
//
// Each screen is a complete view inside the shell. Screens are intentionally
// simple: flat rules, monospace data, instrument-style status badges.
// ============================================================================

// --- Shared atoms ----------------------------------------------------------

function PageHeader({ title, kicker, meta, children }) {
  return (
    <div style={pageHeaderStyle}>
      <div>
        {kicker && <div style={kickerStyle}>{kicker}</div>}
        <h1 style={titleStyle}>{title}</h1>
      </div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        {meta && <span style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--fg-2)' }}>{meta}</span>}
        {children}
      </div>
    </div>
  );
}

function Badge({ tone = 'neutral', children, mono, dot }) {
  const tones = {
    ok:   { bg: 'var(--sig-ok-bg)',   fg: 'var(--sig-ok)'   },
    warn: { bg: 'var(--sig-warn-bg)', fg: 'var(--sig-warn)' },
    err:  { bg: 'var(--sig-err-bg)',  fg: 'var(--sig-err)'  },
    info: { bg: 'var(--sig-info-bg)', fg: 'var(--sig-info)' },
    ion:  { bg: 'var(--ion-bg)',      fg: 'var(--ion)'      },
    neutral: { bg: 'var(--bg-2)',     fg: 'var(--fg-1)'     },
  };
  const c = tones[tone] || tones.neutral;
  return (
    <span style={{
      display: 'inline-flex', alignItems: 'center', gap: 5,
      padding: '2px 7px',
      fontFamily: mono ? 'var(--font-mono)' : 'var(--font-sans)',
      fontSize: 11, fontWeight: 500,
      letterSpacing: '0.04em',
      textTransform: mono ? 'none' : 'uppercase',
      background: c.bg, color: c.fg,
      borderRadius: 999,
      lineHeight: 1.3,
    }}>
      {dot && <span style={{ width: 6, height: 6, borderRadius: '50%', background: 'currentColor' }} />}
      {children}
    </span>
  );
}

function Btn({ variant = 'secondary', children, onClick, icon }) {
  const variants = {
    primary:   { bg: 'var(--ion)', fg: '#fff', border: 'var(--ion)' },
    secondary: { bg: 'var(--bg-1)', fg: 'var(--fg-0)', border: 'var(--fg-3)' },
    ghost:     { bg: 'transparent', fg: 'var(--fg-0)', border: 'transparent' },
    danger:    { bg: 'var(--sig-err)', fg: '#fff', border: 'var(--sig-err)' },
  };
  const v = variants[variant];
  return (
    <button onClick={onClick} style={{
      display: 'inline-flex', alignItems: 'center', gap: 6,
      padding: '7px 12px', fontFamily: 'var(--font-sans)', fontSize: 13, fontWeight: 500,
      border: `1px solid ${v.border}`, background: v.bg, color: v.fg,
      borderRadius: 4, cursor: 'pointer', lineHeight: 1,
    }}>
      {icon && <Icon name={icon} size={14} />}
      {children}
    </button>
  );
}

function KVGrid({ items }) {
  return (
    <div style={{ display: 'grid', gridTemplateColumns: '180px 1fr', rowGap: 8, columnGap: 16 }}>
      {items.map((it, i) => (
        <React.Fragment key={i}>
          <div className="ds-label" style={{ padding: '4px 0' }}>{it.k}</div>
          <div style={{ padding: '4px 0', fontFamily: it.mono ? 'var(--font-mono)' : 'var(--font-sans)', fontSize: it.mono ? 13 : 14, color: 'var(--fg-0)' }}>
            {it.children || it.v}
          </div>
        </React.Fragment>
      ))}
    </div>
  );
}

function Rule() { return <div style={{ height: 1, background: 'var(--fg-4)' }} />; }

// --- Overview --------------------------------------------------------------

function OverviewScreen() {
  return (
    <>
      <PageHeader
        kicker="gate · status"
        title="Workspace overview"
        meta="refreshed 2s ago"
      >
        <Btn variant="secondary" icon="terminal">Open terminal</Btn>
        <Btn variant="primary" icon="play">Run inference</Btn>
      </PageHeader>

      {/* Top stat row — instrumentation feel */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 16, marginBottom: 24 }}>
        <Stat label="Models loaded"  value="12"   trend="+1 today"   tone="ok"   />
        <Stat label="Active routes"  value="8"    trend="all healthy" tone="ok"  />
        <Stat label="Req / minute"   value="4,143" trend="p99 4.8ms"  tone="neutral" mono />
        <Stat label="Policy"         value="verified" trend="ML-DSA-65 · 17d"    tone="ok" />
      </div>

      {/* Repos table */}
      <Section title="Linked repos" caption="workspace ./smallaios.toml">
        <Table
          columns={['Repo', 'Branch', 'State', 'Ahead', 'Behind', 'Last commit']}
          rightAlign={[3, 4]}
          rows={[
            ['SmallAIOS', <span style={{fontFamily:'var(--font-mono)'}}>develop</span>, <Badge tone="ok" dot>clean</Badge>, '3', '0', '14:02  ev  ff: onnx runtime bump'],
            ['ModelGate', <span style={{fontFamily:'var(--font-mono)'}}>develop</span>, <Badge tone="err" dot>dirty · 2</Badge>, '0', '1', '13:58  ev  wip: cedar analyze'],
            ['Runes',     <span style={{fontFamily:'var(--font-mono)'}}>main</span>,    <Badge tone="ok" dot>clean</Badge>, '0', '0', '2d ago  ci  bump deps'],
          ]}
        />
      </Section>

      <div style={{ height: 24 }} />

      {/* Two columns: recent builds + alerts */}
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16 }}>
        <Section title="Recent builds" caption="dependency-ordered">
          <Table
            columns={['Crate', 'Duration', 'Result']}
            rightAlign={[1]}
            rows={[
              ['smallaios-core',    '8.4s',  <Badge tone="ok" dot>passed</Badge>],
              ['smallaios-net',     '4.1s',  <Badge tone="ok" dot>passed</Badge>],
              ['smallaios-onnx',    '12.2s', <Badge tone="ok" dot>passed</Badge>],
              ['modelgate-runtime', '…',     <Badge tone="info" dot>running</Badge>],
              ['modelgate-cedar',   '—',     <Badge tone="neutral">queued</Badge>],
            ]}
          />
        </Section>
        <Section title="Alerts" caption="2 open">
          <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 14 }}>
            <Alert tone="warn" title="cedar analyze: 2 properties unsolved"
                   body="property no-cross-boundary-write is within SMT timeout but undetermined. Inspect in Policy → Analyze." />
            <Alert tone="err" title="boundary B-17 missing TLA+ refinement proof"
                   body="GPU scheduler → ONNX dispatch. smctl gate boundaries check --regen to generate skeleton." />
          </div>
        </Section>
      </div>
    </>
  );
}

function Stat({ label, value, trend, tone, mono }) {
  return (
    <div style={{ border: '1px solid var(--fg-3)', borderRadius: 6, background: 'var(--bg-1)', padding: '14px 16px' }}>
      <div className="ds-label" style={{ marginBottom: 8 }}>{label}</div>
      <div style={{
        fontSize: 28, fontWeight: 600, letterSpacing: '-0.01em',
        fontFamily: mono ? 'var(--font-mono)' : 'var(--font-sans)',
        fontVariantNumeric: 'tabular-nums slashed-zero',
        color: 'var(--fg-0)', lineHeight: 1.1,
      }}>{value}</div>
      <div style={{ marginTop: 6, display: 'flex', alignItems: 'center', gap: 6, fontSize: 12, color: 'var(--fg-2)', fontFamily: 'var(--font-mono)' }}>
        {tone === 'ok' && <span style={{ color: 'var(--sig-ok)' }}>●</span>}
        {tone === 'warn' && <span style={{ color: 'var(--sig-warn)' }}>●</span>}
        {tone === 'err' && <span style={{ color: 'var(--sig-err)' }}>●</span>}
        {trend}
      </div>
    </div>
  );
}

function Alert({ tone, title, body }) {
  const tones = {
    warn: { fg: 'var(--sig-warn)', bg: 'var(--sig-warn-bg)' },
    err:  { fg: 'var(--sig-err)',  bg: 'var(--sig-err-bg)' },
    info: { fg: 'var(--sig-info)', bg: 'var(--sig-info-bg)' },
  }[tone];
  return (
    <div style={{ display: 'flex', gap: 12 }}>
      <div style={{ width: 3, alignSelf: 'stretch', background: tones.fg, flexShrink: 0 }} />
      <div>
        <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--fg-0)', marginBottom: 2 }}>{title}</div>
        <div style={{ fontSize: 12, color: 'var(--fg-1)', lineHeight: 1.5 }}>{body}</div>
      </div>
    </div>
  );
}

// --- Models ---------------------------------------------------------------

function ModelsScreen() {
  return (
    <>
      <PageHeader kicker="gate · models" title="Models" meta="12 loaded · 0 failing">
        <Btn variant="secondary" icon="terminal">smctl gate models</Btn>
        <Btn variant="primary" icon="plus">Register model</Btn>
      </PageHeader>

      <div style={{ display: 'flex', gap: 8, alignItems: 'center', marginBottom: 16 }}>
        {['All', 'Loaded', 'Failing', 'Unsigned'].map((t, i) => (
          <button key={t} style={{
            padding: '5px 12px', fontSize: 12, fontFamily: 'var(--font-sans)', fontWeight: 500,
            background: i === 0 ? 'var(--ion-bg)' : 'var(--bg-1)',
            color: i === 0 ? 'var(--ion)' : 'var(--fg-1)',
            border: '1px solid ' + (i === 0 ? 'transparent' : 'var(--fg-3)'),
            borderRadius: 4, cursor: 'pointer',
          }}>{t}</button>
        ))}
        <div style={{ flex: 1 }} />
        <div style={searchBoxStyle}>
          <Icon name="search" size={14} color="var(--fg-2)" />
          <input placeholder="filter by name, sha, route…" style={searchInputStyle} />
        </div>
      </div>

      <Section title="" caption="" flush>
        <Table
          columns={['Name', 'Version', 'Format', 'Size', 'Signed', 'Route', 'State']}
          rightAlign={[3]}
          rows={[
            [<Mono>llama-3-8b-instruct</Mono>,    <Mono>3.0.1</Mono>,  'ONNX fp16', '4.2 GB', <Badge tone="ok" dot>ML-DSA-65</Badge>,  <Mono>/v1/chat</Mono>,     <Badge tone="ok" dot>loaded</Badge>],
            [<Mono>phi-3-mini</Mono>,              <Mono>1.4.0</Mono>,  'ONNX int4', '1.8 GB', <Badge tone="ok" dot>ML-DSA-65</Badge>,  <Mono>/v1/small</Mono>,    <Badge tone="ok" dot>loaded</Badge>],
            [<Mono>whisper-tiny.en</Mono>,         <Mono>0.9.2</Mono>,  'ONNX fp32', '72 MB',  <Badge tone="ok" dot>ML-DSA-65</Badge>,  <Mono>/v1/transcribe</Mono>, <Badge tone="ok" dot>loaded</Badge>],
            [<Mono>clip-vit-b32</Mono>,            <Mono>2.1.0</Mono>,  'ONNX fp16', '350 MB', <Badge tone="ok" dot>ML-DSA-65</Badge>,  <Mono>/v1/embed</Mono>,    <Badge tone="ok" dot>loaded</Badge>],
            [<Mono>yolov8-nano</Mono>,             <Mono>0.3.7</Mono>,  'ONNX fp16', '12 MB',  <Badge tone="warn" dot>unsigned</Badge>,  <Mono>—</Mono>,            <Badge tone="warn" dot>pending</Badge>],
            [<Mono>detr-resnet-50</Mono>,          <Mono>1.0.0</Mono>,  'ONNX fp32', '167 MB', <Badge tone="err" dot>expired</Badge>,    <Mono>—</Mono>,            <Badge tone="err" dot>failed</Badge>],
          ]}
        />
      </Section>
    </>
  );
}

function Mono({ children }) {
  return <span style={{ fontFamily: 'var(--font-mono)', fontSize: 13 }}>{children}</span>;
}

// --- Policy ---------------------------------------------------------------

function PolicyScreen() {
  return (
    <>
      <PageHeader kicker="gate · policy" title="SecurityPolicy" meta="verified · 17d 04:21:08">
        <Btn variant="secondary" icon="logs">View history</Btn>
        <Btn variant="secondary">Analyze</Btn>
        <Btn variant="primary">Load policy blob</Btn>
      </PageHeader>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 360px', gap: 16 }}>
        <Section title="Cedar policy" caption="policies/default.cedar · 142 lines">
          <pre style={codeBlockStyle}>
{`permit (
  principal in Group::"operators",
  action in [Action::"infer", Action::"read_logs"],
  resource in Model::"phi-3-mini"
) when {
  resource.signature.verified &&
  resource.format == "ONNX" &&
  context.tls.kem == "ML-KEM-768" &&
  context.boundary.label == SecurityLabel::"TRUSTED"
};

forbid (
  principal,
  action,
  resource
) when {
  !resource.signature.verified
};`}
          </pre>
        </Section>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>
          <Section title="Analysis" caption="Cedar SMT · TLA+ refinement">
            <div style={{ padding: 16, display: 'flex', flexDirection: 'column', gap: 10 }}>
              <AnalysisRow label="no-unsigned-inference" state="ok" />
              <AnalysisRow label="no-cross-boundary-write" state="warn" />
              <AnalysisRow label="only-operators-can-route" state="ok" />
              <AnalysisRow label="all-requests-logged" state="ok" />
              <AnalysisRow label="pqc-required" state="ok" />
            </div>
          </Section>
          <Section title="Signing" caption="">
            <div style={{ padding: 16 }}>
              <KVGrid items={[
                { k: 'Algorithm',   v: 'ML-DSA-65', mono: true },
                { k: 'Key ID',      v: 'mg-root-2026', mono: true },
                { k: 'Fingerprint', v: '0f:18:70:22:99:ab', mono: true },
                { k: 'Valid until', v: '2027-04-17', mono: true },
              ]} />
            </div>
          </Section>
        </div>
      </div>
    </>
  );
}

function AnalysisRow({ label, state }) {
  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
      {state === 'ok'   && <Icon name="check" size={14} color="var(--sig-ok)" />}
      {state === 'warn' && <span style={{ color: 'var(--sig-warn)', fontFamily: 'var(--font-mono)' }}>!</span>}
      {state === 'err'  && <Icon name="x" size={14} color="var(--sig-err)" />}
      <span style={{ fontFamily: 'var(--font-mono)', fontSize: 13 }}>{label}</span>
      <span style={{ marginLeft: 'auto' }}>
        {state === 'ok' && <Badge tone="ok">verified</Badge>}
        {state === 'warn' && <Badge tone="warn">unsolved</Badge>}
        {state === 'err' && <Badge tone="err">violated</Badge>}
      </span>
    </div>
  );
}

// --- Terminal (embeds the CLI look) ---------------------------------------

function TerminalScreen() {
  return (
    <>
      <PageHeader kicker="workspace · terminal" title="Terminal" meta="smctl · interactive">
        <Btn variant="secondary" icon="plus">New session</Btn>
      </PageHeader>
      <div style={{
        background: '#0B0C0E', color: '#F3F4F6',
        border: '1px solid var(--fg-3)', borderRadius: 6,
        fontFamily: 'var(--font-mono)', fontSize: 13, lineHeight: 1.55,
        padding: '16px 20px', minHeight: 360,
      }}>
        <div><span style={{color:'#6A6E78'}}>e@airgap</span> <span style={{color:'#6E83FF'}}>~/work/smallaios</span> <span style={{color:'#6A6E78'}}>$</span> smctl workspace status</div>
        <div><span style={{color:'#6A6E78'}}>├─</span> SmallAIOS   <span style={{color:'#6E83FF'}}>develop</span>   <span style={{color:'#3BCB72'}}>✓ clean</span>    <span style={{color:'#6A6E78'}}>+3/-0</span></div>
        <div><span style={{color:'#6A6E78'}}>├─</span> ModelGate   <span style={{color:'#6E83FF'}}>develop</span>   <span style={{color:'#E56372'}}>✗ dirty</span>    <span style={{color:'#6A6E78'}}>2 files</span></div>
        <div><span style={{color:'#6A6E78'}}>└─</span> Runes       <span style={{color:'#6E83FF'}}>main</span>      <span style={{color:'#3BCB72'}}>✓ clean</span>    <span style={{color:'#6A6E78'}}>+0/-0</span></div>
        <div style={{marginTop:8}}><span style={{color:'#6A6E78'}}>e@airgap</span> <span style={{color:'#6E83FF'}}>~/work/smallaios</span> <span style={{color:'#6A6E78'}}>$</span><span style={{ display:'inline-block', width:8, height:15, background:'#3BCB72', verticalAlign:'-3px', marginLeft:2, animation:'smctl-blink 1s steps(2) infinite'}} /></div>
      </div>
    </>
  );
}

// --- Shared layout primitives ---------------------------------------------

function Section({ title, caption, children, flush }) {
  return (
    <section style={{
      background: 'var(--bg-1)',
      border: '1px solid var(--fg-3)',
      borderRadius: 6,
      overflow: 'hidden',
    }}>
      {(title || caption) && (
        <header style={{
          display: 'flex', alignItems: 'center', justifyContent: 'space-between',
          height: 40, padding: '0 16px',
          borderBottom: '1px solid var(--fg-4)',
          background: 'var(--bg-1)',
        }}>
          <div style={{ fontSize: 13, fontWeight: 600, color: 'var(--fg-0)' }}>{title}</div>
          {caption && <div style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--fg-2)' }}>{caption}</div>}
        </header>
      )}
      <div style={flush ? {} : { }}>{children}</div>
    </section>
  );
}

function Table({ columns, rows, rightAlign = [] }) {
  return (
    <table style={{ width: '100%', borderCollapse: 'collapse' }}>
      <thead>
        <tr>
          {columns.map((c, i) => (
            <th key={i} style={{
              textAlign: rightAlign.includes(i) ? 'right' : 'left',
              padding: '10px 16px',
              fontSize: 11, fontWeight: 500, letterSpacing: '0.08em', textTransform: 'uppercase',
              color: 'var(--fg-2)', background: 'var(--bg-1)',
              borderBottom: '1px solid var(--fg-3)',
              fontFamily: 'var(--font-sans)',
            }}>{c}</th>
          ))}
        </tr>
      </thead>
      <tbody>
        {rows.map((row, ri) => (
          <tr key={ri} style={{ borderBottom: '1px solid var(--fg-4)' }}>
            {row.map((cell, ci) => (
              <td key={ci} style={{
                padding: '10px 16px', fontSize: 13, color: 'var(--fg-0)',
                textAlign: rightAlign.includes(ci) ? 'right' : 'left',
                fontFamily: ci === 0 ? 'var(--font-mono)' : 'var(--font-sans)',
                fontVariantNumeric: 'tabular-nums slashed-zero',
              }}>{cell}</td>
            ))}
          </tr>
        ))}
      </tbody>
    </table>
  );
}

// --- Styles ----------------------------------------------------------------

const pageHeaderStyle = {
  display: 'flex', alignItems: 'flex-end', justifyContent: 'space-between',
  paddingBottom: 16, marginBottom: 24, borderBottom: '1px solid var(--fg-3)',
};
const kickerStyle = {
  fontFamily: 'var(--font-mono)', fontSize: 11,
  color: 'var(--fg-2)', letterSpacing: '0.08em', textTransform: 'uppercase',
  marginBottom: 4,
};
const titleStyle = {
  margin: 0, fontSize: 22, fontWeight: 600,
  letterSpacing: '-0.01em', color: 'var(--fg-0)',
};
const searchBoxStyle = {
  display: 'flex', alignItems: 'center', gap: 8,
  padding: '0 10px', height: 28, width: 320,
  border: '1px solid var(--fg-3)', borderRadius: 4, background: 'var(--bg-1)',
};
const searchInputStyle = {
  flex: 1, border: 'none', outline: 'none',
  font: 'inherit', fontFamily: 'var(--font-mono)', fontSize: 12,
  background: 'transparent', color: 'var(--fg-0)',
};
const codeBlockStyle = {
  margin: 0, padding: 16,
  fontFamily: 'var(--font-mono)', fontSize: 12, lineHeight: 1.55,
  color: 'var(--fg-0)', background: 'var(--bg-0)',
  overflowX: 'auto',
};

Object.assign(window, { OverviewScreen, ModelsScreen, PolicyScreen, TerminalScreen });
