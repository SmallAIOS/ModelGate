# ModelGate web — Design Document

## Context

`smctl-gate-v1` provides the operator CLI surface for ModelGate. The designer-authored React mockup at `ui/ui_kits/modelgate_web/` is a static HTML/JSX preview drawn 1:1 against those subcommands. It has no router, no data, no build — it renders a single demo page. This change turns it into a production SPA served by a minimal Axum crate.

## Goals / Non-Goals

### Goals

1. Single binary (`modelgate-web`) serves a static SPA plus a proxy to the ModelGate REST API.
2. Operator runs `smctl gate web` and the dashboard opens in a browser.
3. Every screen maps 1:1 to a `smctl gate` subcommand — the CLI stays the source of truth for *what* the product does; the web is a rendering of *that*.
4. Frontend is TypeScript + React + Vite. No CSS framework — the in-house design system in `ui/colors_and_type.css` is the only style source.
5. Design-system voice rules (imperative buttons, sentence case, no emoji) apply to every label. The existing `voice-audit` script in `openspec/changes/archive/2026-04-24-smctl-copy-v1` is the precedent.

### Non-Goals

1. No auth layer. Local-dev only, 127.0.0.1 bind.
2. No SSR, no Next.js, no meta-framework.
3. No new ModelGate-side API endpoints.
4. No in-browser Cedar policy editing — read-only in this change.

## Decisions

### Decision 1: Rust-served SPA, not separate static hosting

**Choice:** Ship a Rust crate (`modelgate-web`) that embeds the built frontend via `include_dir!` and serves it alongside a `/api/*` reverse proxy.

**Rationale:** One binary, one port, one run command. CORS disappears because the proxy and the SPA share an origin. Matches the user's Rust-primary preference (memory: "Rust-primary, TS only for frontend"). The existing `smctl-mcp` crate already uses Axum, so the dependency footprint is already paid for.

**Alternative rejected:** Separate static host + CORS to the ModelGate API directly. This adds CORS config to ModelGate itself for a dev-only UI — not worth it.

### Decision 2: Proxy through `smctl-gate::GateClient`, not raw reqwest

**Choice:** The `/api/*` proxy in `modelgate-web` reuses `smctl_gate::GateClient`. Each inbound HTTP handler maps to one `GateClient` method (`list_models`, `set_route`, etc.) and re-serializes the result.

**Rationale:** Zero HTTP duplication. Error variants (`GateError::ModelNotFound`, `ConnectionRefused`, `Timeout`) already carry the right semantics; the proxy maps them to sensible HTTP statuses (404, 502, 504). SSE endpoints (`stream_logs`) stream through as `text/event-stream` unchanged.

**Trade-off:** Couples the web crate to `smctl-gate`. Accepted — both are thin layers over the same upstream; divergence would be bugs anyway.

### Decision 3: Vite + React + TypeScript, no CSS framework

**Choice:** `ui/modelgate-web/` is a Vite project. Config is the Vite defaults plus a `base: './'` so the bundle runs at any mount path. All styling comes from `ui/colors_and_type.css` (copied into `src/styles.css` at build time).

**Rationale:** Vite + React is the path of least surprise for 2026 frontend work. TypeScript catches the kind of prop-name drift the mockup would otherwise accumulate. No Tailwind / no styled-components — the design system is hand-rolled and already prescriptive; adding a utility layer fights it.

### Decision 4: Typed API client generated from `smctl-gate` types

**Choice:** `ui/modelgate-web/src/api.ts` declares TypeScript mirrors of `smctl_gate::{HealthStatus, Model, Route, InferenceResult, LogEntry}`. A small script in `tools/` (follow-up) will generate these from the Rust structs with `ts-rs` once the surface stabilizes.

**Rationale:** Hand-roll now, generate later. The API surface is small enough (4 types, ~15 fields) that hand-written TS will stay accurate for one release cycle. When it stops being accurate, generation pays for itself.

### Decision 5: Embed the built bundle, don't read from disk

**Choice:** `include_dir!("../ui/modelgate-web/dist")` at compile time. The built frontend ships inside the `smctl` binary.

**Rationale:** One artifact. `smctl gate web` works from an `scp`'d binary without a separate asset directory. Build-time verification that the frontend compiled. Roughly matches how `smctl-mcp` does things today.

**Trade-off:** Rebuilding the frontend requires a Rust rebuild for the bundle to refresh. Dev mode (`smctl gate web --dev`) can bypass this by proxying to `vite dev` instead — deferred, note in tasks.md.

### Decision 6: `smctl gate web` is the only entry point

**Choice:** No standalone `modelgate-web` binary on PATH. The only way to start the server is via `smctl gate web [--port N] [--host 127.0.0.1] [--open]`.

**Rationale:** One UX surface. The same precedence chain (`--url` / `MODELGATE_URL` / `[gate]`) already resolves where the server proxies to. `--open` uses `open` / `xdg-open` to launch a browser.

### Decision 7: Design-system voice applies to every label

**Choice:** Every user-visible string in `ui/modelgate-web/` is reviewed against `design-system-v1`'s voice rules. Buttons use imperative verbs (`Register model`, not `Registering…`), status uses the canonical vocabulary (`healthy`, `degraded`, `unhealthy` from `smctl-gate`), and no emoji.

**Rationale:** The CLI already passes this bar (per `smctl-copy-v1`). A web UI that contradicts it would be confusing. A follow-up PR will add a `tools/voice-audit-web.ts` that walks the TSX and flags new non-compliant strings.

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| Frontend build breaks on CI | Add `cargo build -p modelgate-web` to CI matrix; `include_dir!` will refuse to compile if `dist/` is absent, so the Rust build proves the frontend built |
| Bundle size grows unbounded | Set a 2 MB budget; fail CI when `dist/` exceeds it. React + a small icon set should come in well under |
| ModelGate API changes break the SPA | The proxy forwards `GateError` → HTTP with a stable shape; the SPA degrades per-panel instead of whiting out |
| Cedar policy panel has no data source yet | Screen renders "not yet available" state; design matches the `design-system-v1` empty-state pattern |
| Dev ergonomics: rebuild-for-reload hurts | `--dev` flag in a follow-up change proxies to `vite dev` on 5173 instead of serving embedded assets |

## Open Questions

1. Should `smctl gate web` live inside the `smctl-gate` crate or in its own `modelgate-web` crate? Leaning toward its own crate — keeps frontend build dependencies out of `smctl-gate`.
2. Where does the session-state data (selected left-rail item, command palette history) persist — `localStorage` only, or a `/api/session` endpoint? localStorage-only for v1; server-side later if multi-user ever matters.
3. Do we generate the TS types from Rust now (`ts-rs`) or hand-roll and regenerate later? Hand-roll now.
