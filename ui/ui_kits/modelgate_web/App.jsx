/* global React, TopBar, LeftRail, StatusLine, Icon,
   OverviewScreen, ModelsScreen, PolicyScreen, TerminalScreen */
const { useState, useEffect } = React;

// ============================================================================
// ModelGate web — top-level app
// Shell (TopBar + LeftRail + StatusLine) + routed screen
// ============================================================================

function GateApp() {
  const [screen, setScreen] = useState(() => localStorage.getItem('mg-screen') || 'overview');
  const [paletteOpen, setPaletteOpen] = useState(false);

  useEffect(() => { localStorage.setItem('mg-screen', screen); }, [screen]);
  useEffect(() => {
    function onKey(e) {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault(); setPaletteOpen((v) => !v);
      } else if (e.key === 'Escape') {
        setPaletteOpen(false);
      }
    }
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: 'var(--bg-0)' }}>
      <TopBar
        workspace="smallaios"
        branch="feature/gpu-accel"
        onOpenPalette={() => setPaletteOpen(true)}
      />
      <div style={{ display: 'flex', flex: 1, minHeight: 0 }}>
        <LeftRail current={screen} onNavigate={setScreen} />
        <main style={{
          flex: 1, minWidth: 0, overflowY: 'auto',
          padding: '32px 40px 48px',
          maxWidth: 1440, margin: '0 auto', width: '100%', boxSizing: 'border-box',
        }} data-screen-label={`ModelGate · ${screen}`}>
          {screen === 'overview'   && <OverviewScreen />}
          {screen === 'models'     && <ModelsScreen />}
          {screen === 'routes'     && <PlaceholderScreen title="Routes"     kicker="gate · routes"     />}
          {screen === 'policy'     && <PolicyScreen />}
          {screen === 'boundaries' && <PlaceholderScreen title="Boundaries" kicker="gate · boundaries" />}
          {screen === 'terminal'   && <TerminalScreen />}
          {screen === 'settings'   && <PlaceholderScreen title="Settings"   kicker="workspace · config" />}
        </main>
      </div>
      <StatusLine workspace="smallaios" branch="feature/gpu-accel" lastSync="14:02:31Z" />

      {paletteOpen && <Palette onClose={() => setPaletteOpen(false)} onNavigate={(s) => { setScreen(s); setPaletteOpen(false); }} />}
    </div>
  );
}

function PlaceholderScreen({ title, kicker }) {
  return (
    <>
      <div style={{ paddingBottom: 16, marginBottom: 24, borderBottom: '1px solid var(--fg-3)' }}>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--fg-2)', letterSpacing: '0.08em', textTransform: 'uppercase' }}>{kicker}</div>
        <h1 style={{ margin: '4px 0 0', fontSize: 22, fontWeight: 600, letterSpacing: '-0.01em' }}>{title}</h1>
      </div>
      <div style={{
        border: '1px dashed var(--fg-3)',
        borderRadius: 6,
        padding: '48px 32px',
        textAlign: 'center',
        color: 'var(--fg-2)',
        fontFamily: 'var(--font-mono)',
        fontSize: 13,
        background: 'var(--bg-1)',
      }}>
        <div style={{ fontSize: 14, color: 'var(--fg-1)', marginBottom: 6 }}>Not drawn in this kit.</div>
        <div>Out of scope for the first design pass — see ui_kits/modelgate_web/README.md.</div>
      </div>
    </>
  );
}

function Palette({ onClose, onNavigate }) {
  const items = [
    { id: 'overview',   label: 'Overview',   hint: 'workspace summary',       section: 'Navigate' },
    { id: 'models',     label: 'Models',     hint: '12 loaded · 0 failing',   section: 'Navigate' },
    { id: 'routes',     label: 'Routes',     hint: '8 active',                section: 'Navigate' },
    { id: 'policy',     label: 'Policy',     hint: 'Cedar · verified',        section: 'Navigate' },
    { id: 'boundaries', label: 'Boundaries', hint: '29 proofs',               section: 'Navigate' },
    { id: 'terminal',   label: 'Terminal',   hint: 'smctl interactive',       section: 'Navigate' },
    { id: 'settings',   label: 'Settings',   hint: 'workspace config',        section: 'Navigate' },
  ];
  const actions = [
    { label: 'smctl workspace sync',       hint: 'fetch + pull all repos' },
    { label: 'smctl build --parallel',     hint: 'dependency-ordered build' },
    { label: 'smctl gate policy verify',   hint: 'Cedar SMT + TLA+' },
    { label: 'smctl spec new <name>',      hint: 'scaffold an OpenSpec folder' },
  ];
  return (
    <div
      onClick={onClose}
      style={{
        position: 'fixed', inset: 0, zIndex: 1000,
        background: 'rgba(11,12,14,0.32)',
        display: 'flex', alignItems: 'flex-start', justifyContent: 'center',
        paddingTop: 96,
        backdropFilter: 'blur(4px)',
      }}>
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          width: 640, background: 'var(--bg-1)',
          border: '1px solid var(--fg-3)', borderRadius: 8,
          boxShadow: '0 0 0 1px rgba(11,12,14,0.08), 0 2px 4px rgba(11,12,14,0.08), 0 16px 24px rgba(11,12,14,0.08)',
          overflow: 'hidden',
        }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 10, padding: '14px 18px', borderBottom: '1px solid var(--fg-4)' }}>
          <Icon name="search" size={16} color="var(--fg-2)" />
          <input
            autoFocus placeholder="Jump to page or run command…"
            style={{ flex: 1, border: 'none', outline: 'none', font: 'inherit', fontSize: 14, background: 'transparent' }}
          />
          <span className="ds-kbd">esc</span>
        </div>
        <div style={{ padding: '8px 0', maxHeight: 420, overflowY: 'auto' }}>
          <PaletteGroup title="Navigate">
            {items.map((it) => (
              <button key={it.id} onClick={() => onNavigate(it.id)} style={paletteRow}>
                <Icon name="chevron" size={12} color="var(--fg-2)" />
                <span>{it.label}</span>
                <span style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--fg-2)' }}>{it.hint}</span>
              </button>
            ))}
          </PaletteGroup>
          <PaletteGroup title="Run">
            {actions.map((a) => (
              <button key={a.label} style={paletteRow}>
                <Icon name="terminal" size={12} color="var(--fg-2)" />
                <span style={{ fontFamily: 'var(--font-mono)', fontSize: 13 }}>{a.label}</span>
                <span style={{ fontFamily: 'var(--font-sans)', fontSize: 12, color: 'var(--fg-2)' }}>{a.hint}</span>
              </button>
            ))}
          </PaletteGroup>
        </div>
      </div>
    </div>
  );
}

function PaletteGroup({ title, children }) {
  return (
    <>
      <div style={{
        padding: '10px 18px 4px',
        fontFamily: 'var(--font-sans)', fontSize: 10,
        letterSpacing: '0.12em', textTransform: 'uppercase',
        color: 'var(--fg-2)', fontWeight: 500,
      }}>{title}</div>
      {children}
    </>
  );
}

const paletteRow = {
  display: 'grid',
  gridTemplateColumns: '20px 1fr auto',
  alignItems: 'center',
  gap: 12,
  width: '100%',
  padding: '8px 18px',
  border: 'none', background: 'transparent',
  cursor: 'pointer',
  fontSize: 13, color: 'var(--fg-0)', fontFamily: 'var(--font-sans)',
  textAlign: 'left',
};

Object.assign(window, { GateApp });
