# ModelGate web — Tasks

Ordered so each block lands as one commit. Plan of record — ticked as work lands.

## Prep

- [x] Decide: block this change on `smctl-gate-v1` merging to develop, or rebase when it lands — resolved: smctl-gate-v1 landed as PR #10 (commit b778d3a) on 2026-04-24; branch rebased
- [ ] Reserve MSGID range `SMCTL-0301..0399` for `modelgate-web` in `openspec/changes/archive/2026-04-24-smctl-logging-v1/specs/logging.md` — touchpoint follow-up, not blocking

## Crate Setup

- [x] Create `modelgate-web/` crate with Cargo.toml (axum, tower-http deps) — `include_dir` deferred to the "embed the built bundle" task in Axum Server
- [x] Add `modelgate-web` as workspace member in root Cargo.toml
- [x] Add `smctl-gate` dependency (the crate reuses `GateClient`)
- [x] `cargo build --workspace` passes — stub `_ping` route + two passing tests ship the vertical

## Frontend Scaffold

- [ ] `npm create vite@latest ui/modelgate-web -- --template react-ts` (or equivalent hand-scaffold)
- [ ] Add `react`, `react-dom`, `lucide-react`, `@tanstack/react-query`, `typescript`
- [ ] Copy `ui/colors_and_type.css` → `ui/modelgate-web/src/styles.css`
- [ ] Port `ui/ui_kits/modelgate_web/Shell.jsx` → `src/Shell.tsx`
- [ ] Port `ui/ui_kits/modelgate_web/Screens.jsx` → `src/Screens.tsx`
- [ ] Port `ui/ui_kits/modelgate_web/App.jsx` → `src/App.tsx` (hash-router)
- [ ] Extract inline icons to `src/components/Icon.tsx` using `lucide-react`
- [ ] `npm run typecheck` passes

## API Client

- [ ] `src/api.ts` declares `HealthStatus`, `Model`, `Route`, `LogEntry`, `GateApi`
- [ ] Implement `health()`, `listModels()`, `listRoutes()` (GET-only for v1 commit)
- [ ] `GateApiError` class with `{ kind, status, body }`
- [ ] `useHealth`, `useModels`, `useRoutes` hooks backed by React Query

## Wire Screens to Data

- [ ] `OverviewScreen` reads `useHealth()` + `useModels()` for counts
- [ ] `ModelsScreen` reads `useModels()`; register / remove actions stubbed (disabled buttons + "coming soon" tooltip)
- [ ] `PolicyScreen` renders "not yet available" state — blocked on ModelGate policy endpoint
- [ ] `TerminalScreen` renders "open in terminal" CTA that links to `smctl gate logs` — embedded xterm.js deferred to follow-up change

## Axum Server

- [x] `modelgate-web/src/lib.rs` — Axum app builder with `/` static + `/api/*` routes — `/api/*` fully wired (GET health/models/routes, POST models/inference, PUT routes, DELETE models/:name, GET logs SSE); static `/` pending
- [x] `modelgate-web/src/proxy.rs` — handlers call `smctl_gate::GateClient` and serialize results — kept inline in lib.rs; file is small enough that a split costs more than it saves
- [x] Error mapping: `GateError` → HTTP status + JSON body per `specs/web-server.md`
- [ ] `include_dir!("../ui/modelgate-web/dist")` embeds the SPA
- [ ] `build.rs` fails with a helpful message when `dist/` is missing (points at `npm run build`)

## CLI Integration

- [x] Add `GateCommands::Web { host, port, open }` to smctl — `--dev` deferred to a follow-up task block once the frontend dev server exists
- [x] Wire dispatch: build `WebServerConfig`, call `modelgate_web::serve`
- [x] `--open` launches the default browser via a hand-rolled platform opener (`open` / `xdg-open` / `cmd /c start`) — no new dep
- [ ] `--dev` reserved flag — deferred until frontend scaffold lands

## Logging

- [ ] Reserve SMCTL-0301..0304 in the logging catalog (see Prep)
- [ ] Emit `SMCTL-0301` on server start, `SMCTL-0302` on graceful shutdown
- [ ] Emit `SMCTL-0303` / `SMCTL-0304` on proxy upstream failures

## Integration Testing

- [x] Rust: axum test harness asserts `/api/health` returns 200 when a wiremock upstream replies 200, and 502 when the upstream is unreachable — also covers 504 on timeout and upstream 5xx passthrough
- [x] Rust: assert `/api/models/:name` DELETE returns 404 when upstream returns 404
- [x] Rust: assert `/api/logs` streams SSE frames through unchanged
- [ ] Frontend: Vitest covers `api.ts` success + error parsing against a mocked `fetch`

## Voice / Design

- [ ] Every string in `src/` passes a manual voice-rule review per `design-system-v1` (imperative buttons, canonical status vocab, no emoji)
- [ ] Accessibility pass: every interactive element is `<button>` or `<a>`; command palette is `aria-modal`
- [ ] `ui/ui_kits/modelgate_web/README.md` updated to note the production app is now at `ui/modelgate-web/`

## Docs

- [ ] Add "Web UI" section to repo README pointing at `smctl gate web`
- [ ] Update `CLAUDE.md` Design System section to note the web app location

## Verify

- [ ] `cargo build --workspace` passes
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace -- -D warnings` stays clean
- [ ] `npm run build` in `ui/modelgate-web/` produces a `dist/` under the 2 MB budget
- [ ] `smctl gate web --open` boots, renders Overview, and shows real counts from a running ModelGate (or a clear "upstream unreachable" state against a dead upstream)
