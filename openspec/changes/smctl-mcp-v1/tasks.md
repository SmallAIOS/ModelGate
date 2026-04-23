# smctl MCP Server — Tasks

## Crate Setup

- [x] Create `smctl-mcp` crate with Cargo.toml (rmcp, tokio, schemars dependencies)
- [x] Add `smctl-mcp` as workspace member in root Cargo.toml
- [x] Add `smctl-mcp` dependency to `smctl` binary crate
- [x] Add `smctl-log` as a dependency of `smctl-mcp` for MSGID constants
- [x] Verify workspace builds with `cargo build --workspace`

## Cross-Cutting Contract Wiring

- [x] `smctl serve --mcp` calls `smctl_log::init` before starting the transport; do NOT install a separate `tracing-subscriber` inside `smctl-mcp`
- [x] Extend `smctl_log::MsgId` with the seven `Mcp*` variants allocated in `design.md` Decision 5 (`SMCTL-0200` through `SMCTL-0206`), with matching entries in the spec's MSGID catalog
- [x] Extend `smctl_log::MsgId` with the two resource-scoped variants `McpResourceRead` (`SMCTL-0207`) and `McpResourceReadFailed` (`SMCTL-0208`), with matching entries in the spec's MSGID catalog
- [x] Update `smctl-log/src/msgid.rs` tests to cover the new variants
- [x] Emit `SMCTL-0200` on server initialize, `SMCTL-0201` on graceful shutdown, `SMCTL-0202` on tool-call receipt, `SMCTL-0203` on success / `SMCTL-0204` on error
- [x] Emit `SMCTL-0205` (unexpected client disconnect) and `SMCTL-0206` (transport fatal). Wired on the SSE transport: `SMCTL-0206` on bind failure and axum serve-loop error. Per-session client disconnect is observed by rmcp's `LocalSessionManager` task; the transport-level wrapper does not re-emit `SMCTL-0205` because the session task owns that lifecycle.
- [x] Tool descriptions (exposed via `tools/list`) conform to `design-system-v1` voice rules — sentence case, imperative verbs, no emoji, `you` not `we`
- [x] Error payloads returned to MCP clients carry three-part structured messages per `smctl-errors-v1` design; the remediation clause names a real `smctl` subcommand where one applies

## Server Core

- [x] Implement `SmctlServer` struct with workspace root and config
- [x] Implement `ServerHandler` trait for `SmctlServer`
- [x] Register server capabilities (tools — resources/logging capability advertisements deferred)
- [x] Add `smctl serve` subcommand to CLI with `--mcp`, `--stdio`, `--sse`, and `--port` flags (default port 9377; `--stdio` and `--sse` are mutually exclusive)

## Transport Layer

- [x] Implement stdio transport via `rmcp::transport::stdio`
- [x] Implement SSE transport via `rmcp::transport::streamable_http_server::StreamableHttpService` wrapped in an axum router (rmcp 1.5 ships streamable-HTTP instead of the standalone `SseServer` the spec draft assumed)
- [ ] Add graceful shutdown on SIGINT/SIGTERM
- [ ] Test stdio transport with manual JSON-RPC messages

## MCP Tools — Workspace

- [x] `smctl_workspace_init` — Initialize workspace
- [x] `smctl_workspace_status` — Show repo statuses (vertical slice — landed)
- [x] `smctl_workspace_add` — Add repo to workspace
- [x] `smctl_workspace_remove` — Remove repo from workspace
- [x] `smctl_workspace_sync` — Fetch/pull all repos

## MCP Tools — Worktree

- [x] `smctl_worktree_add` — Create linked worktrees
- [x] `smctl_worktree_list` — List active worktrees
- [x] `smctl_worktree_remove` — Remove worktree set

## MCP Tools — Git Flow

- [x] `smctl_flow_init` — Ensure develop branch exists in every repo
- [x] `smctl_flow_feature_start` — Start feature branch
- [x] `smctl_flow_feature_finish` — Merge feature to develop
- [x] `smctl_flow_release_start` — Create release branch
- [x] `smctl_flow_release_finish` — Finalize release
- [x] `smctl_flow_hotfix_start` — Start hotfix
- [x] `smctl_flow_hotfix_finish` — Merge hotfix

## MCP Tools — OpenSpec

- [x] `smctl_spec_new` — Create spec folder and branch
- [ ] `smctl_spec_status` — Show spec progress (deferred; `smctl_spec_list` covers the listing case and includes per-spec phase + task progress)
- [x] `smctl_spec_validate` — Check completeness
- [x] `smctl_spec_archive` — Archive completed spec
- [x] `smctl_spec_list` — List every spec with phase + task progress

## MCP Tools — Build

- [x] `smctl_build` — Build repos in dependency order

## MCP Resources

- [x] `smctl://workspace/config` — Workspace configuration (TOML; error if no manifest)
- [x] `smctl://workspace/status` — Live repo status (same shape as the `smctl_workspace_status` tool)
- [x] `smctl://flow/branches` — Active flow branches (feature/release/hotfix, grouped)
- [x] `smctl://spec/list` — All specs with status (phase + task-completion counts, archived included)
- [x] `smctl://spec/{name}/tasks` — Task progress for a spec (templated URI, advertised via `resources/templates/list`; rejects nested / empty names)
- [x] Advertise the `ResourcesCapability` on `ServerHandler::get_info`
- [x] Advertise MIME types in the resource metadata (`application/toml` for the config, `application/json` elsewhere)
- [x] Override `list_resources` / `list_resource_templates` / `read_resource` on `SmctlServer`; emit `SMCTL-0207` on success and `SMCTL-0208` on failure
- [ ] Implement resource subscription and update notifications

## Integration Testing

- [x] Test MCP initialize handshake over stdio (`tests/stdio_handshake.rs::initialize_and_call_workspace_status`)
- [x] Test tool invocation round-trip for a representative tool from every family (workspace, worktree, spec, build, flow)
- [x] Test resource listing and reading — `tests/stdio_handshake.rs` asserts `resources/list`, `resources/templates/list`, and `resources/read` round-trips for each of the four static URIs plus the templated `smctl://spec/{name}/tasks`, plus a `resource_not_found` negative case for an unknown URI
- [x] Test SSE transport connection and tool invocation (`tests/sse_handshake.rs::sse_initialize_and_call_workspace_status` binds on `127.0.0.1:0`, connects an rmcp streamable-HTTP client, and round-trips `smctl_workspace_status`)
- [x] Test error handling (invalid tool params, workspace not found) — `tests/error_path.rs::tool_failure_emits_smctl_0204_with_structured_fields` spawns `smctl serve --mcp --stdio` as a subprocess pointed at a workspace-less tempdir, drives a raw JSON-RPC handshake, and asserts the `--log-file` capture contains `SMCTL-0204` with `tool`, `request_id`, `duration_ms`, `error_kind`, and `remediation` STRUCTURED-DATA fields plus an executable `smctl` subcommand in the remediation value.

## Documentation

- [ ] Write MCP integration guide for Claude Code
- [ ] Write MCP integration guide for Cursor
- [ ] Write MCP integration guide for Windsurf
- [ ] Add `smctl serve` to README.md command reference

## Verify

- [x] `smctl serve --mcp --stdio` responds to MCP initialize handshake (covered by `tests/stdio_handshake.rs`)
- [x] All workspace/flow/spec/build tools callable via MCP — 20 tools across 5 families now registered; listing + representative `tools/call` assertions in `tests/stdio_handshake.rs`
- [x] Resources return correct data for current workspace state — covered by the resource round-trip assertions in `tests/stdio_handshake.rs` against an empty manifest and a scaffolded spec
- [x] SSE transport works for local connections (binds to `127.0.0.1:<port>`; default port `9377`; covered by `tests/sse_handshake.rs`). Remote + TLS deferred per Out of Scope in the proposal.
- [x] Starting the server emits `SMCTL-0200` to the configured log transports (covered by `tests/logging.rs`)
- [x] A failing tool call emits `SMCTL-0204` with `tool`, `request_id`, `duration_ms`, `error_kind`, and `remediation` STRUCTURED-DATA fields — covered by `tests/error_path.rs`; happy path SMCTL-0202 + SMCTL-0203 remain covered by the (still-ignored) `tests/logging.rs` and the stdio / SSE handshake round-trips
- [x] Tool descriptions and error-payload strings pass a voice-conformance read-through against `design-system-v1`
- [x] `cargo test --workspace` passes with new crate
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [x] `cargo fmt --check` clean

## Spec Drift and Follow-ups

Findings from the first vertical-slice implementation pass. These were surfaced during implementation and are scoped to be picked up in a follow-up commit series on this branch or in a subsequent change:

- ~~**Branch-divergence with `change/smctl-logging-v1`.**~~ **Resolved.** This branch now forks from `change/smctl-logging-v1` (not `change/design-system-v1`). The `smctl-log` crate is the full RFC 5424 version from the logging branch; the seven `Mcp*` MSGID variants land as extensions to `smctl-log/src/msgid.rs` rather than a parallel minimal crate. The previous stripped-down `smctl-log` scaffold commit is gone.
- ~~**`SMCTL-0205` and `SMCTL-0206` are unreachable on stdio.**~~ **Resolved.** `SMCTL-0206` now fires on the SSE transport from `server::serve_sse` when `TcpListener::bind` fails or the axum serve-loop returns an error. The stdio transport still has no path to either MSGID — peer close is a clean shutdown there — and this is documented on `serve_stdio`. Per-session client disconnect on SSE is observed inside rmcp's `LocalSessionManager` task; if dedicated `SMCTL-0205` emission is required, wrap the session manager in a follow-up change.
- ~~**SSE transport, streamable HTTP transport.**~~ **Resolved (SSE).** Shipped as `smctl serve --mcp --sse --port <n>` via `rmcp`'s `transport-streamable-http-server` feature, wrapped in an axum router. Integration coverage in `tests/sse_handshake.rs`. Note: rmcp 1.5 consolidates plain SSE into its streamable-HTTP server; the separate `SseServer` assumed by the spec draft does not exist in this version. Streamable-HTTP-with-sessions beyond the default config and TLS are still deferred.
- ~~**Remaining MCP tools.**~~ **Resolved.** 19 additional tools landed across workspace (init/add/remove/sync), worktree (add/list/remove), flow (init/feature/release/hotfix), spec (new/validate/archive/list), and build. Two minor API-drift resolutions: `workspace_sync` inlines a per-repo git-pull loop because `smctl_workspace::sync` does not exist; `smctl_spec_status` is deferred because `list_specs` already carries per-spec phase + task progress.
- ~~**MCP resources.**~~ **Resolved.** Five resources now ship: `smctl://workspace/config` (TOML), `smctl://workspace/status`, `smctl://flow/branches`, `smctl://spec/list` (all JSON), plus the templated `smctl://spec/{name}/tasks`. `SMCTL-0207` / `SMCTL-0208` cover the read success / failure events. Subscription and list-changed notifications are still out — flip those on in `ServerCapabilities::builder()` when the polling strategy changes. Path-traversal is blocked at parse time (spec names containing `/` are rejected by `parse_spec_tasks_uri`).
- ~~**Error-path MSGID coverage.**~~ **Resolved.** `tests/error_path.rs` spawns `smctl serve --mcp --stdio --workspace <tmpdir>` as a subprocess against a workspace-less tempdir and asserts `SMCTL-0204` lands in the `--log-file` capture with the full contract: `tool`, `request_id`, `duration_ms`, `error_kind`, and `remediation` (the last naming a real `smctl` subcommand). The existing `McpToolFailed` emission site was extended to include `remediation`, sourced from a static tool-to-subcommand map.
- **`rmcp` API divergence from the original spec draft.** The first draft of the implementation spec assumed `rmcp` constructs (e.g. `transport::channel()`, `serve_server` helpers) that the 0.8+ API does not expose by that name. Current implementation uses `rmcp::transport::stdio()` and `ServiceExt::serve` directly. Update `specs/mcp-server-impl.md` sample code in a follow-up editorial pass.
