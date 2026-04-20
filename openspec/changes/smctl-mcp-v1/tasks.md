# smctl MCP Server — Tasks

## Crate Setup

- [ ] Create `smctl-mcp` crate with Cargo.toml (rmcp, tokio, schemars dependencies)
- [ ] Add `smctl-mcp` as workspace member in root Cargo.toml
- [ ] Add `smctl-mcp` dependency to `smctl` binary crate
- [ ] Add `smctl-log` as a dependency of `smctl-mcp` for MSGID constants
- [ ] Verify workspace builds with `cargo build --workspace`

## Cross-Cutting Contract Wiring

- [ ] `smctl serve --mcp` calls `smctl_log::init` before starting the transport; do NOT install a separate `tracing-subscriber` inside `smctl-mcp`
- [ ] Extend `smctl_log::MsgId` with the seven `Mcp*` variants allocated in `design.md` Decision 5 (`SMCTL-0200` through `SMCTL-0206`), with matching entries in the spec's MSGID catalog
- [ ] Update `smctl-log/src/msgid.rs` tests to cover the new variants
- [ ] Emit `SMCTL-0200` on server initialize, `SMCTL-0201` on graceful shutdown, `SMCTL-0202` on tool-call receipt, `SMCTL-0203` on success / `SMCTL-0204` on error, `SMCTL-0205` on unexpected transport close, `SMCTL-0206` on transport fatal
- [ ] Tool descriptions (exposed via `tools/list`) conform to `design-system-v1` voice rules — sentence case, imperative verbs, no emoji, `you` not `we`
- [ ] Error payloads returned to MCP clients carry three-part structured messages per `smctl-errors-v1` design; the remediation clause names a real `smctl` subcommand where one applies

## Server Core

- [ ] Implement `SmctlServer` struct with workspace root and config
- [ ] Implement `ServerHandler` trait for `SmctlServer`
- [ ] Register server capabilities (tools, resources, logging)
- [ ] Add `smctl serve` subcommand to CLI with `--mcp`, `--stdio`, `--sse`, `--port` flags

## Transport Layer

- [ ] Implement stdio transport via `rmcp::transport::stdio`
- [ ] Implement SSE transport via `rmcp::transport::SseServer`
- [ ] Add graceful shutdown on SIGINT/SIGTERM
- [ ] Test stdio transport with manual JSON-RPC messages

## MCP Tools — Workspace

- [ ] `smctl_workspace_init` — Initialize workspace
- [ ] `smctl_workspace_status` — Show repo statuses
- [ ] `smctl_workspace_add` — Add repo to workspace
- [ ] `smctl_workspace_remove` — Remove repo from workspace
- [ ] `smctl_workspace_sync` — Fetch/pull all repos

## MCP Tools — Worktree

- [ ] `smctl_worktree_add` — Create linked worktrees
- [ ] `smctl_worktree_list` — List active worktrees
- [ ] `smctl_worktree_remove` — Remove worktree set

## MCP Tools — Git Flow

- [ ] `smctl_flow_feature_start` — Start feature branch
- [ ] `smctl_flow_feature_finish` — Merge feature to develop
- [ ] `smctl_flow_release_start` — Create release branch
- [ ] `smctl_flow_release_finish` — Finalize release
- [ ] `smctl_flow_hotfix_start` — Start hotfix
- [ ] `smctl_flow_hotfix_finish` — Merge hotfix

## MCP Tools — OpenSpec

- [ ] `smctl_spec_new` — Create spec folder and branch
- [ ] `smctl_spec_status` — Show spec progress
- [ ] `smctl_spec_validate` — Check completeness
- [ ] `smctl_spec_archive` — Archive completed spec

## MCP Tools — Build

- [ ] `smctl_build` — Build repos in dependency order

## MCP Resources

- [ ] `smctl://workspace/config` — Workspace configuration
- [ ] `smctl://workspace/status` — Live repo status
- [ ] `smctl://flow/branches` — Active flow branches
- [ ] `smctl://spec/list` — All specs with status
- [ ] `smctl://spec/{name}/tasks` — Task progress for a spec
- [ ] Implement resource subscription and update notifications

## Integration Testing

- [ ] Test MCP initialize handshake over stdio
- [ ] Test tool invocation round-trip (call tool → get JSON result)
- [ ] Test resource listing and reading
- [ ] Test SSE transport connection and tool invocation
- [ ] Test error handling (invalid tool params, workspace not found)

## Documentation

- [ ] Write MCP integration guide for Claude Code
- [ ] Write MCP integration guide for Cursor
- [ ] Write MCP integration guide for Windsurf
- [ ] Add `smctl serve` to README.md command reference

## Verify

- [ ] `smctl serve --mcp --stdio` responds to MCP initialize handshake
- [ ] All workspace/flow/spec/build tools callable via MCP
- [ ] Resources return correct data for current workspace state
- [ ] SSE transport works for remote connections
- [ ] Starting the server emits `SMCTL-0200` to the configured log transports
- [ ] A failing tool call emits `SMCTL-0204` with `tool`, `request_id`, and `error_kind` STRUCTURED-DATA fields
- [ ] Tool descriptions and error-payload strings pass a voice-conformance read-through against `design-system-v1`
- [ ] `cargo test --workspace` passes with new crate
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `cargo fmt --check` clean
