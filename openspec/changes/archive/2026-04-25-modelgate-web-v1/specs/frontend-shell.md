# ModelGate web — Frontend Specification

## Overview

`ui/modelgate-web/` is a Vite + React + TypeScript SPA. It renders the views already prototyped in `ui/ui_kits/modelgate_web/Shell.jsx` and `Screens.jsx`, backed by the typed API client at `src/api.ts` which calls `/api/*` on the same origin.

## Directory Layout

```
ui/modelgate-web/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── index.html
└── src/
    ├── main.tsx            # entry; renders <App />
    ├── App.tsx             # router, command palette (⌘K), left-rail selection
    ├── Shell.tsx           # TopBar, LeftRail, StatusLine, Icon, LogoMark
    ├── Screens.tsx         # OverviewScreen, ModelsScreen, PolicyScreen, TerminalScreen
    ├── api.ts              # typed ModelGate client
    ├── hooks.ts            # useHealth, useModels, useRoutes
    ├── components/
    │   ├── Badge.tsx
    │   ├── Button.tsx      # exports `Btn` for compat with ui_kits
    │   ├── Table.tsx
    │   ├── Section.tsx
    │   └── KVGrid.tsx
    └── styles.css          # copied verbatim from ui/colors_and_type.css
```

All component names match the mockup; the port is line-by-line where possible, with `.jsx` → `.tsx` and inline icons extracted to `components/Icon.tsx` using `lucide-react`.

## API Client

```ts
// src/api.ts

export type HealthStatus = {
  status: string;
  version: string;
  uptime_secs: number;
  model_count: number;
};

export type Model = {
  name: string;
  format: string;
  size_bytes: number;
  registered_at: string;
  status: string;
};

export type Route = {
  model: string;
  endpoint: string;
  active: boolean;
  request_count: number;
};

export type LogEntry = {
  timestamp: string;
  level: string;
  message: string;
  fields: unknown;
};

export class GateApi {
  constructor(private base = '/api') {}
  async health(): Promise<HealthStatus>;
  async listModels(): Promise<Model[]>;
  async removeModel(name: string): Promise<void>;
  async listRoutes(): Promise<Route[]>;
  async setRoute(model: string, endpoint: string): Promise<Route>;
  // inference and streaming land in follow-up commits per tasks.md
}
```

The client throws a `GateApiError` with `{ kind, status, body }` matching the server's JSON error shape. Components render the error via a shared `<ErrorBox />` that uses the design-system empty-state pattern.

## Routing

Hash-based router (`#/overview`, `#/models`, `#/policy`, `#/terminal`) so the SPA works when served from any mount path and survives static hosting without `try_files` rewrites. The selected tab in the left rail persists to `localStorage`.

## Voice Compliance

Every user-visible string is reviewed against `design-system-v1` voice rules:

- Buttons: imperative verbs (`Register model`, not `Register` or `Registering`).
- Status: `healthy` / `degraded` / `unhealthy` / `loaded` / `unloaded` / `active` / `inactive` — verbatim from `smctl-gate::HealthStatus` / `Model` / `Route`.
- No exclamation points. No emoji. Sentence case in labels.
- Empty states match the CLI's `no models registered` / `no routes configured` wording.

A follow-up tool `tools/voice-audit-web.ts` will walk the built TSX and flag strings that violate the above. Out of scope for this change; noted in tasks.md as a deferred item.

## State Management

No Redux, no Zustand. React Query (`@tanstack/react-query`) handles server state (health, models, routes) with 30s stale-time. `useState` + context for UI state (selected tab, palette open).

## Styling

- Single CSS file, copied verbatim from `ui/colors_and_type.css` at build time via a Vite plugin or a tiny `npm run sync-tokens` script.
- No CSS-in-JS.
- No Tailwind.
- All colors, spacing, radii come from the design system — no inline magic numbers.

## Accessibility

- Every actionable element is a `<button>` or `<a>`. Divs with click handlers are a lint failure.
- Focus styles come from the design system (not browser default).
- The command palette (⌘K) is a dialog with focus trap and `aria-modal="true"`.
- Tables use `<table>` / `<thead>` / `<tbody>` semantics, not divs.

## Build & Test

- `npm run build` → `dist/` (embedded into the Rust crate at cargo-build time).
- `npm run dev` → Vite dev server on 5173, proxies `/api` to `http://localhost:9378` during development.
- `npm run typecheck` → `tsc --noEmit` runs in CI before any Rust build.
- Unit tests via Vitest for `api.ts` parsing only. Component tests are out of scope for v1 — the CLI is the system of record and the SPA renders its output.
