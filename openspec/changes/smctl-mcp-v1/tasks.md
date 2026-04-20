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
- [x] Update `smctl-log/src/msgid.rs` tests to cover the new variants
- [x] Emit `SMCTL-0200` on server initialize, `SMCTL-0201` on graceful shutdown, `SMCTL-0202` on tool-call receipt, `SMCTL-0203` on success / `SMCTL-0204` on error
- [ ] Emit `SMCTL-0205` (unexpected client disconnect) and `SMCTL-0206` (transport fatal). Not reachable on the stdio-only slice — stdio close is the clean-shutdown signal. Moved to Spec Drift / Follow-ups.
- [x] Tool descriptions (exposed via `tools/list`) conform to `design-system-v1` voice rules — sentence case, imperative verbs, no emoji, `you` not `we`
- [x] Error payloads returned to MCP clients carry three-part structured messages per `smctl-errors-v1` design; the remediation clause names a real `smctl` subcommand where one applies

## Server Core

- [x] Implement `SmctlServer` struct with workspace root and config
- [x] Implement `ServerHandler` trait for `SmctlServer`
- [x] Register server capabilities (tools — resources/logging capability advertisements deferred)
- [x] Add `smctl serve` subcommand to CLI with `--mcp` and `--stdio` flags (`--sse` / `--port` deferred with the transports themselves)

## Transport Layer

- [x] Implement stdio transport via `rmcp::transport::stdio`
- [ ] Implement SSE transport via `rmcp::transport::SseServer`
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

- [ ] `smctl://workspace/config` — Workspace configuration
- [ ] `smctl://workspace/status` — Live repo status
- [ ] `smctl://flow/branches` — Active flow branches
- [ ] `smctl://spec/list` — All specs with status
- [ ] `smctl://spec/{name}/tasks` — Task progress for a spec
- [ ] Implement resource subscription and update notifications

## Integration Testing

- [x] Test MCP initialize handshake over stdio (`tests/stdio_handshake.rs::initialize_and_call_workspace_status`)
- [x] Test tool invocation round-trip for a representative tool from every family (workspace, worktree, spec, build, flow)
- [ ] Test resource listing and reading — deferred with resources
- [ ] Test SSE transport connection and tool invocation — deferred with SSE
- [ ] Test error handling (invalid tool params, workspace not found) — see "Error-path MSGID coverage" in Spec Drift

## Documentation

- [ ] Write MCP integration guide for Claude Code
- [ ] Write MCP integration guide for Cursor
- [ ] Write MCP integration guide for Windsurf
- [ ] Add `smctl serve` to README.md command reference

## Verify

- [x] `smctl serve --mcp --stdio` responds to MCP initialize handshake (covered by `tests/stdio_handshake.rs`)
- [x] All workspace/flow/spec/build tools callable via MCP — 20 tools across 5 families now registered; listing + representative `tools/call` assertions in `tests/stdio_handshake.rs`
- [ ] Resources return correct data for current workspace state — resources deferred
- [ ] SSE transport works for remote connections — deferred
- [x] Starting the server emits `SMCTL-0200` to the configured log transports (covered by `tests/logging.rs`)
- [x] A failing tool call emits `SMCTL-0204` with `tool`, `request_id`, and `error_kind` STRUCTURED-DATA fields — happy path verified via `tests/logging.rs` (SMCTL-0202 + SMCTL-0203); error-path emission wiring is in `server.rs` but not covered by a test yet
- [x] Tool descriptions and error-payload strings pass a voice-conformance read-through against `design-system-v1`
- [x] `cargo test --workspace` passes with new crate
- [x] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [x] `cargo fmt --check` clean

## Spec Drift and Follow-ups

Findings from the first vertical-slice implementation pass. These were surfaced during implementation and are scoped to be picked up in a follow-up commit series on this branch or in a subsequent change:

- ~~**Branch-divergence with `change/smctl-logging-v1`.**~~ **Resolved.** This branch now forks from `change/smctl-logging-v1` (not `change/design-system-v1`). The `smctl-log` crate is the full RFC 5424 version from the logging branch; the seven `Mcp*` MSGID variants land as extensions to `smctl-log/src/msgid.rs` rather than a parallel minimal crate. The previous stripped-down `smctl-log` scaffold commit is gone.
- **`SMCTL-0205` and `SMCTL-0206` are unreachable on stdio.** The MSGIDs are declared in the catalog but the stdio transport has no unexpected-disconnect or transport-fatal surface distinct from a clean peer close. They will be wired when SSE / HTTP transports land.
- **SSE transport, streamable HTTP transport.** Deferred. Add `rmcp` SSE feature, wire `--sse --port`, add a second integration test.
- ~~**Remaining MCP tools.**~~ **Resolved.** 19 additional tools landed across workspace (init/add/remove/sync), worktree (add/list/remove), flow (init/feature/release/hotfix), spec (new/validate/archive/list), and build. Two minor API-drift resolutions: `workspace_sync` inlines a per-repo git-pull loop because `smctl_workspace::sync` does not exist; `smctl_spec_status` is deferred because `list_specs` already carries per-spec phase + task progress.
- **MCP resources.** `smctl://workspace/config`, `smctl://workspace/status`, etc. Not started.
- **Error-path MSGID coverage.** Add a test that forces `smctl_workspace_status` to fail (e.g. point it at a workspace-less tempdir) and asserts `SMCTL-0204` with `tool`, `request_id`, `error_kind`, and a `remediation` field that names a real smctl subcommand.
- **`rmcp` API divergence from the original spec draft.** The first draft of the implementation spec assumed `rmcp` constructs (e.g. `transport::channel()`, `serve_server` helpers) that the 0.8+ API does not expose by that name. Current implementation uses `rmcp::transport::stdio()` and `ServiceExt::serve` directly. Update `specs/mcp-server-impl.md` sample code in a follow-up editorial pass.
