/* global React */
const { useState } = React;

// ============================================================================
// ModelGate web — shell chrome components (TopBar, LeftRail, StatusLine)
// Drawn straight from VISUAL FOUNDATIONS:
//  - 64px top chrome, 240px left rail, 24px vim-style status line at bottom
//  - 1px borders, no shadows in chrome
//  - Monospace workspace / branch identifier in the top bar
// ============================================================================

// --- Small icon primitives (Lucide-style strokes) --------------------------
// We draw these inline rather than loading a CDN so the kit is self-contained.
// All use 1.5px stroke, round caps, 24px grid — match the Lucide spec.

const Ic = {
  activity: <polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>,
  package:  <><path d="M3 7l9-5 9 5-9 5z"/><path d="M3 12l9 5 9-5"/><path d="M3 17l9 5 9-5"/></>,
  route:    <><circle cx="6" cy="6" r="2.5"/><circle cx="18" cy="18" r="2.5"/><path d="M6 8.5v7a3 3 0 0 0 3 3h6.5"/></>,
  shield:   <><path d="M12 2 4 6v6c0 5 3.5 9 8 10 4.5-1 8-5 8-10V6z"/><path d="m9 12 2 2 4-4"/></>,
  border:   <><rect x="4" y="4" width="16" height="16" rx="2"/><path d="M4 10h16M10 4v16"/></>,
  terminal: <><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></>,
  settings: <><circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M4.9 4.9l2.1 2.1M17 17l2.1 2.1M4.9 19.1l2.1-2.1M17 7l2.1-2.1"/></>,
  search:   <><circle cx="11" cy="11" r="7"/><line x1="16.5" y1="16.5" x2="21" y2="21"/></>,
  chevron:  <polyline points="9 6 15 12 9 18"/>,
  check:    <polyline points="4 12 10 18 20 6"/>,
  x:        <><line x1="6" y1="6" x2="18" y2="18"/><line x1="18" y1="6" x2="6" y2="18"/></>,
  dot:      <circle cx="12" cy="12" r="4" fill="currentColor" stroke="none"/>,
  play:     <polygon points="6 4 20 12 6 20"/>,
  pause:    <><rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/></>,
  plus:     <><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></>,
  logs:     <><path d="M5 4h14v16H5z"/><line x1="8" y1="8" x2="16" y2="8"/><line x1="8" y1="12" x2="16" y2="12"/><line x1="8" y1="16" x2="12" y2="16"/></>,
  bell:     <><path d="M6 9a6 6 0 1 1 12 0c0 5 2 6 2 6H4s2-1 2-6"/><path d="M10 19a2 2 0 1 0 4 0"/></>,
};

function Icon({ name, size = 16, color = 'currentColor', strokeWidth = 1.5 }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width={size}
      height={size}
      fill="none"
      stroke={color}
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      {Ic[name] || null}
    </svg>
  );
}

// --- Logo mark (from assets/logo-mark.svg, inlined so chrome is self-contained)

function LogoMark({ size = 24 }) {
  return (
    <svg viewBox="0 0 48 48" width={size} height={size} fill="none"
         stroke="currentColor" strokeWidth="2" strokeLinecap="square" aria-hidden="true">
      <rect x="4" y="4" width="40" height="40" rx="3"/>
      <path d="M14 14 H34 M14 24 H24 M24 24 V34 H34"/>
      <circle cx="14" cy="14" r="1.5" fill="currentColor" stroke="none"/>
      <circle cx="34" cy="14" r="1.5" fill="currentColor" stroke="none"/>
      <circle cx="14" cy="24" r="1.5" fill="currentColor" stroke="none"/>
      <circle cx="24" cy="24" r="1.5" fill="currentColor" stroke="none"/>
      <circle cx="24" cy="34" r="1.5" fill="currentColor" stroke="none"/>
      <circle cx="34" cy="34" r="1.5" fill="currentColor" stroke="none"/>
    </svg>
  );
}

// --- TopBar (64px) ---------------------------------------------------------

function TopBar({ workspace, branch, onOpenPalette }) {
  return (
    <header style={topBarStyle}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, width: 240, paddingLeft: 20 }}>
        <LogoMark size={22} />
        <div style={{ display: 'flex', flexDirection: 'column', lineHeight: 1.1 }}>
          <span style={{ fontSize: 14, fontWeight: 600, letterSpacing: '-0.01em' }}>ModelGate</span>
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--fg-2)' }}>
            v0.1.0-alpha
          </span>
        </div>
      </div>

      {/* Workspace / branch identifier (monospace, ion-on-hover) */}
      <div style={wsChipStyle}>
        <Icon name="package" size={14} />
        <span style={{ color: 'var(--fg-0)' }}>{workspace}</span>
        <span style={{ color: 'var(--fg-3)' }}>/</span>
        <span style={{ color: 'var(--ion)' }}>{branch}</span>
      </div>

      {/* Command palette trigger */}
      <button style={paletteBtnStyle} onClick={onOpenPalette}>
        <Icon name="search" size={14} color="var(--fg-2)" />
        <span style={{ color: 'var(--fg-2)' }}>Jump to…</span>
        <span style={{ marginLeft: 'auto', display: 'flex', gap: 4 }}>
          <span className="ds-kbd">⌘</span>
          <span className="ds-kbd">K</span>
        </span>
      </button>

      <div style={{ marginLeft: 'auto', display: 'flex', alignItems: 'center', gap: 4, paddingRight: 20 }}>
        <button style={iconBtnStyle} title="Logs"><Icon name="logs" size={16} /></button>
        <button style={iconBtnStyle} title="Alerts">
          <Icon name="bell" size={16} />
          <span style={alertDotStyle} />
        </button>
        <div style={{ width: 1, height: 24, background: 'var(--fg-3)', margin: '0 8px' }} />
        <div style={avatarStyle}>ev</div>
      </div>
    </header>
  );
}

// --- LeftRail (240px) ------------------------------------------------------

function LeftRail({ current, onNavigate }) {
  const nav = [
    { id: 'overview',   label: 'Overview',        icon: 'activity' },
    { id: 'models',     label: 'Models',          icon: 'package',  badge: '12' },
    { id: 'routes',     label: 'Routes',          icon: 'route',    badge: '8'  },
    { id: 'policy',     label: 'Policy',          icon: 'shield',   ok: true    },
    { id: 'boundaries', label: 'Boundaries',      icon: 'border',   badge: '29' },
    { id: 'terminal',   label: 'Terminal',        icon: 'terminal'              },
    { id: 'settings',   label: 'Settings',        icon: 'settings'              },
  ];
  return (
    <nav style={railStyle} aria-label="Primary">
      <div style={railSectionStyle}>
        <div style={railLabelStyle}>Gate</div>
        {nav.slice(0, 6).map((item) => (
          <RailItem key={item.id} item={item} active={current === item.id} onClick={() => onNavigate(item.id)} />
        ))}
      </div>

      <div style={railSectionStyle}>
        <div style={railLabelStyle}>Workspace</div>
        <RailItem
          item={{ id: 'settings', label: 'Settings', icon: 'settings' }}
          active={current === 'settings'}
          onClick={() => onNavigate('settings')}
        />
        <div style={{ padding: '12px 12px 4px', display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div style={railLabelStyle}>Worktrees</div>
          <button style={railAddStyle} aria-label="Add worktree"><Icon name="plus" size={12} /></button>
        </div>
        <div style={wtItemStyle}>
          <Icon name="dot" size={10} color="var(--sig-ok)" />
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 12 }}>gpu-accel</span>
          <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--fg-2)' }}>+3</span>
        </div>
        <div style={wtItemStyle}>
          <Icon name="dot" size={10} color="var(--sig-warn)" />
          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 12 }}>onnx-bump</span>
          <span style={{ marginLeft: 'auto', fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--fg-2)' }}>!2</span>
        </div>
      </div>

      <div style={{ marginTop: 'auto', padding: 12, borderTop: '1px solid var(--fg-4)' }}>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--fg-2)', display: 'flex', flexDirection: 'column', gap: 2 }}>
          <span>host · airgap-01</span>
          <span>unix:///run/modelgate.sock</span>
          <span><span style={{ color: 'var(--sig-ok)' }}>● </span>connected · 17d</span>
        </div>
      </div>
    </nav>
  );
}

function RailItem({ item, active, onClick }) {
  return (
    <button
      onClick={onClick}
      style={{ ...railItemStyle, ...(active ? railItemActiveStyle : null) }}
    >
      <Icon name={item.icon} size={16} />
      <span>{item.label}</span>
      {item.badge && <span style={railBadgeStyle}>{item.badge}</span>}
      {item.ok && <span style={{ ...railBadgeStyle, background: 'var(--sig-ok-bg)', color: 'var(--sig-ok)' }}>ok</span>}
    </button>
  );
}

// --- StatusLine (24px, vim-style) ------------------------------------------

function StatusLine({ workspace, branch, lastSync }) {
  return (
    <footer style={statusLineStyle}>
      <span style={{ fontWeight: 600, color: 'var(--fg-0)' }}>workspace</span>
      <span>{workspace}</span>
      <span style={{ color: 'var(--fg-3)' }}>·</span>
      <span>branch</span>
      <span style={{ color: 'var(--ion)' }}>{branch}</span>
      <span style={{ color: 'var(--fg-3)' }}>·</span>
      <span><span style={{ color: 'var(--sig-ok)' }}>●</span> gate online</span>
      <span style={{ marginLeft: 'auto' }}>last sync {lastSync}</span>
      <span style={{ color: 'var(--fg-3)' }}>·</span>
      <span>p50 1.2ms · p99 4.8ms</span>
    </footer>
  );
}

// --- Styles ----------------------------------------------------------------

const topBarStyle = {
  height: 64,
  display: 'flex',
  alignItems: 'center',
  background: 'var(--bg-1)',
  borderBottom: '1px solid var(--fg-3)',
  flexShrink: 0,
  gap: 20,
};
const wsChipStyle = {
  display: 'flex',
  alignItems: 'center',
  gap: 8,
  padding: '6px 12px',
  fontFamily: 'var(--font-mono)',
  fontSize: 12,
  border: '1px solid var(--fg-3)',
  borderRadius: 4,
  background: 'var(--bg-0)',
  color: 'var(--fg-1)',
  height: 32,
  boxSizing: 'border-box',
};
const paletteBtnStyle = {
  display: 'flex',
  alignItems: 'center',
  gap: 8,
  padding: '0 12px',
  height: 32,
  minWidth: 340,
  fontFamily: 'var(--font-sans)',
  fontSize: 13,
  border: '1px solid var(--fg-3)',
  borderRadius: 4,
  background: 'var(--bg-0)',
  cursor: 'pointer',
  color: 'var(--fg-2)',
};
const iconBtnStyle = {
  position: 'relative',
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  width: 32, height: 32,
  border: '1px solid transparent',
  borderRadius: 4,
  background: 'transparent',
  color: 'var(--fg-1)',
  cursor: 'pointer',
};
const alertDotStyle = {
  position: 'absolute',
  top: 8, right: 8,
  width: 6, height: 6,
  borderRadius: '50%',
  background: 'var(--sig-warn)',
  boxShadow: '0 0 0 2px var(--bg-1)',
};
const avatarStyle = {
  width: 28, height: 28,
  borderRadius: 4,
  background: 'var(--ion)',
  color: '#fff',
  fontSize: 11,
  fontWeight: 600,
  fontFamily: 'var(--font-mono)',
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'center',
  letterSpacing: '0.04em',
};
const railStyle = {
  width: 240,
  flexShrink: 0,
  background: 'var(--bg-1)',
  borderRight: '1px solid var(--fg-3)',
  display: 'flex',
  flexDirection: 'column',
  overflowY: 'auto',
};
const railSectionStyle = {
  padding: '12px 8px 16px',
  borderBottom: '1px solid var(--fg-4)',
};
const railLabelStyle = {
  fontFamily: 'var(--font-sans)',
  fontSize: 10,
  letterSpacing: '0.12em',
  textTransform: 'uppercase',
  color: 'var(--fg-2)',
  padding: '0 8px 8px',
  fontWeight: 500,
};
const railItemStyle = {
  display: 'flex',
  alignItems: 'center',
  gap: 10,
  width: '100%',
  padding: '7px 10px',
  background: 'transparent',
  border: 'none',
  borderRadius: 4,
  cursor: 'pointer',
  fontFamily: 'var(--font-sans)',
  fontSize: 13,
  color: 'var(--fg-1)',
  textAlign: 'left',
};
const railItemActiveStyle = {
  background: 'var(--ion-bg)',
  color: 'var(--ion)',
  fontWeight: 500,
};
const railBadgeStyle = {
  marginLeft: 'auto',
  fontFamily: 'var(--font-mono)',
  fontSize: 10,
  padding: '2px 6px',
  background: 'var(--bg-2)',
  color: 'var(--fg-2)',
  borderRadius: 3,
};
const railAddStyle = {
  width: 18, height: 18,
  border: '1px solid var(--fg-3)',
  background: 'var(--bg-0)',
  color: 'var(--fg-1)',
  borderRadius: 3,
  display: 'inline-flex',
  alignItems: 'center',
  justifyContent: 'center',
  cursor: 'pointer',
};
const wtItemStyle = {
  display: 'flex',
  alignItems: 'center',
  gap: 8,
  padding: '6px 10px',
};
const statusLineStyle = {
  height: 24,
  display: 'flex',
  alignItems: 'center',
  gap: 8,
  padding: '0 16px',
  background: 'var(--bg-1)',
  borderTop: '1px solid var(--fg-3)',
  fontFamily: 'var(--font-mono)',
  fontSize: 11,
  color: 'var(--fg-2)',
  flexShrink: 0,
};

Object.assign(window, { Icon, LogoMark, TopBar, LeftRail, StatusLine });
