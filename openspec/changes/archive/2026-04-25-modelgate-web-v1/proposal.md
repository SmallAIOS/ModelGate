# ModelGate web — Proposal

## Why

`smctl gate` (PR #10, `smctl-gate-v1`) ships the operator CLI for a running ModelGate instance. A designer-authored React mockup already exists at `ui/ui_kits/modelgate_web/` that draws every view 1:1 against those subcommands. No production web UI exists — operators must use `smctl gate` or `curl`.

Problems this solves:

- **No visual dashboard.** Operators scanning for "is ModelGate healthy, what models are loaded, what's routing" have to string together multiple CLI calls.
- **The mockup is stranded.** `ui/ui_kits/modelgate_web/` is a static React file set with no build pipeline, no routing, no data source — it only renders the demo page.
- **No entry point for non-terminal users.** Some operators (infra leads, SRE reviewers, stakeholders) want a browser URL, not a TUI.

This change is **not** a new API surface. The backend is the same ModelGate REST API that `smctl-gate` already talks to.

## What Changes

Productionize `ui/ui_kits/modelgate_web/` into a real single-page app plus a minimal Rust crate that serves it.

### New Crate

- **`modelgate-web`** — Axum server that serves the built React bundle as static files plus a small JSON proxy layer to the ModelGate REST API (so the SPA doesn't need a second configured URL and so CORS stays simple). Reuses `smctl-gate::GateClient` — no duplicated HTTP code.

### New Frontend Tree

- `ui/modelgate-web/` — Vite + React + TypeScript app built from the kit in `ui/ui_kits/modelgate_web/`. Emits a static bundle under `dist/` that `modelgate-web` embeds via `include_dir!`.

### New CLI Surface

- `smctl gate web` — Start the `modelgate-web` server on a local port. Respects `--gate-url` / `MODELGATE_URL` / `workspace.toml [gate]`. No auth for the first cut (see Non-Goals).

### Screens in Scope

Ported verbatim from the mockup:

- **Overview** — instance health, linked repos, recent activity, open alerts. Backed by `GET /health` and `GET /api/v1/models` (for counts).
- **Models** — table of registered models. Backed by `GET /api/v1/models`. Register / remove actions call the same POST / DELETE routes `smctl gate models` uses.
- **Policy** — Cedar policy viewer. Backed by a new `GET /api/v1/policy` endpoint if ModelGate exposes it; otherwise the panel renders a "not yet available" state. Does **not** block this change.
- **Terminal** — an embedded xterm.js panel that shells out to `smctl` via a WebSocket the server proxies. Deferred to a follow-up change if the socket layer is non-trivial.

## Capabilities

### New Capabilities

- `modelgate-web` crate — Axum static server + JSON proxy to ModelGate
- Frontend build pipeline (Vite, TS, React) under `ui/modelgate-web/`
- `smctl gate web` subcommand to start the server

### Modified Capabilities

- `ui/ui_kits/modelgate_web/` — becomes the *design reference*, no longer the "net-new" mockup. The live app in `ui/modelgate-web/` diverges from the kit once real data lands; the kit stays as the designer's truth and is re-synced on major redesigns.

## Impact

### New Files

```
modelgate-web/
├── Cargo.toml
└── src/
    ├── lib.rs        # Axum app builder
    ├── main.rs       # bin entry: smctl gate web wraps this
    ├── proxy.rs      # /api/* reverse-proxy to ModelGate via smctl-gate
    └── assets.rs     # include_dir! for the built frontend

ui/modelgate-web/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── index.html
└── src/
    ├── main.tsx
    ├── App.tsx
    ├── Shell.tsx         # ported from ui_kits
    ├── Screens.tsx       # ported from ui_kits
    ├── api.ts            # typed ModelGate API client
    └── styles.css        # pulled from ui/colors_and_type.css
```

### Modified Files

- `Cargo.toml` — add `modelgate-web` workspace member
- `smctl/Cargo.toml` — add `modelgate-web` dep
- `smctl/src/main.rs` — add `GateCommands::Web` variant
- `.gitignore` — add `ui/modelgate-web/node_modules/` and `ui/modelgate-web/dist/`
- `ui/ui_kits/modelgate_web/README.md` — note that the production app is now at `ui/modelgate-web/`

### Dependencies

Rust:
- `axum` (0.8+) — already used by `smctl-mcp` for SSE
- `tower-http` (`ServeDir`, `CompressionLayer`) — already pulled by `smctl-mcp`
- `include_dir` (0.7+) — embed the built bundle into the binary

Frontend:
- `react`, `react-dom` (18.x)
- `vite` (5+), `typescript` (5+)
- `lucide-react` — the Lucide icons referenced by the mockup
- No CSS framework — the mockup's hand-rolled design system in `ui/colors_and_type.css` is the single source of truth

## Non-Goals

1. **No auth.** First cut is local-dev only; the server binds to `127.0.0.1`. Adding auth is a separate change once ModelGate itself ships auth.
2. **No SSR, no Next.js.** Pure static SPA + API proxy. Lower ceremony, faster iteration.
3. **No new ModelGate API endpoints.** The server proxies to whatever `smctl-gate` can already reach. If a screen needs data the API doesn't expose, the screen renders a "not yet available" state and the gap is filed as a separate spec.
4. **No Cedar policy editing in-browser.** Read-only viewer in this change; editing lands in a later pass.
5. **No Electron / desktop packaging.** Browser only.

## References

- Design mockup: `ui/ui_kits/modelgate_web/` (README + Screens.jsx + Shell.jsx + App.jsx)
- Design tokens: `ui/colors_and_type.css`, `ui/SKILL.md` (design system skill)
- Contract: `openspec/changes/smctl-gate-v1/specs/gate-api.md`
- Voice rules: `openspec/changes/design-system-v1/specs/design-system.md`
