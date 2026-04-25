# smctl MCP Server — Design Document

## Context

smctl v0.1.3 provides workspace management, git flow, OpenSpec workflow, and build orchestration as CLI commands. This change adds an MCP server mode so AI coding assistants can invoke the same operations programmatically. The MCP server spec from `smctl-tool-v1` (specs/mcp-server.md) defines the architecture; this document captures implementation decisions.

## Goals / Non-Goals

### Goals

1. Every CLI subcommand has a 1:1 MCP tool equivalent — no capabilities are CLI-only or MCP-only
2. MCP tools return structured JSON; CLI commands return human-readable output (same core logic, different formatters)
3. Support stdio transport (primary) and SSE transport (secondary)
4. Resource subscriptions for real-time workspace state updates
5. Zero-config for local use — `smctl serve --mcp --stdio` works out of the box

### Non-Goals

1. Not adding new smctl capabilities — only exposing existing ones via MCP
2. Not implementing MCP prompts — tools and resources only for v1
3. Not adding authentication — local MCP transport is trusted by design
4. Not supporting MCP sampling — smctl does not call AI models

## Decisions

### Decision 1: Use `rmcp` official Rust MCP SDK

**Choice:** Use the official `rmcp` crate from `modelcontextprotocol/rust-sdk`.

**Rationale:** First-party SDK maintained by the MCP organization. Provides `#[tool]` attribute macros for declarative tool registration, `ServerHandler` trait, and built-in stdio/SSE transports. Generates JSON Schema for tool inputs automatically via `schemars`. No reason to hand-roll MCP protocol handling.

**Key features used:**
- `#[tool]` / `#[tool_handler]` — Declarative tool registration
- `schemars` derive — Compile-time input schema generation
- `stdio()` transport — stdin/stdout for Claude Code, Cursor, etc.
- `SseServer` — HTTP SSE for remote clients
- `ServerHandler` trait — Unified handler for tools and resources

### Decision 2: SmctlServer struct as single handler

**Choice:** A single `SmctlServer` struct implements `ServerHandler`, holding references to workspace config and all smctl-* library crates.

```rust
pub struct SmctlServer {
    workspace_root: PathBuf,
    // delegates to smctl-workspace, smctl-flow, smctl-spec, smctl-build
}

#[tool_handler]
impl ServerHandler for SmctlServer {
    #[tool(description = "Initialize a SmallAIOS multi-repo workspace")]
    async fn workspace_init(&self, name: String) -> CallToolResult { ... }

    #[tool(description = "Show status of all repos in the workspace")]
    async fn workspace_status(&self) -> CallToolResult { ... }
    // ... all other tools
}
```

**Rationale:** Keeps the MCP layer thin — each tool method is a 5-10 line adapter that calls the existing smctl core library function and formats the result as JSON.

### Decision 3: Structured JSON results for all tools

**Choice:** All MCP tool results return structured JSON, not human-readable strings. The same core functions power both CLI (formatted text) and MCP (JSON).

**Rationale:** AI assistants parse JSON reliably. The existing `--json` flag on the CLI already demonstrates this pattern. MCP tools always use the JSON path.

### Decision 4: Resource URIs follow `smctl://` scheme

**Choice:** Resources use `smctl://` URI scheme as defined in the v1 MCP spec.

| URI | Source |
|---|---|
| `smctl://workspace/config` | workspace.toml contents |
| `smctl://workspace/status` | `workspace_status()` output |
| `smctl://flow/branches` | `flow_status()` output |
| `smctl://spec/list` | All specs with completion status |
| `smctl://spec/{name}/tasks` | Task markdown for a specific spec |

**Rationale:** Custom URI scheme clearly identifies smctl resources in MCP client UIs. Follows the pattern used by other MCP servers (e.g., `github://`, `postgres://`).

### Decision 5: Transport selection via CLI flags

**Choice:** Transport mode is selected via `smctl serve` flags:
- `--stdio` (default) — stdin/stdout
- `--sse --port <port>` — SSE over HTTP
- `--http --port <port>` — Streamable HTTP (future)

**Rationale:** Matches how other MCP servers configure transport. Default to stdio since that's what Claude Code and Cursor use.

### Decision 5: MSGID allocations in the SMCTL-0200 range

`smctl-logging-v1` reserved `SMCTL-0200`–`SMCTL-0299` for this change. Initial allocations:

| MSGID | Severity | Event |
|---|---|---|
| `SMCTL-0200` | Informational | `McpServerStarted` — server bound and ready to accept connections |
| `SMCTL-0201` | Informational | `McpServerStopped` — graceful shutdown |
| `SMCTL-0202` | Informational | `McpToolCalled` — a tool invocation arrived; structured fields `tool`, `request_id` |
| `SMCTL-0203` | Informational | `McpToolCompleted` — tool returned successfully; fields `tool`, `request_id`, `duration_ms` |
| `SMCTL-0204` | Error | `McpToolFailed` — tool returned an error payload; fields `tool`, `request_id`, `duration_ms`, `error_kind` |
| `SMCTL-0205` | Warning | `McpClientDisconnected` — transport closed unexpectedly |
| `SMCTL-0206` | Error | `McpTransportFatal` — the transport itself failed (e.g. SSE listener crashed) |

The full catalog with STRUCTURED-DATA keys lives in `specs/mcp-server-impl.md` alongside the tool / resource tables. New MSGIDs inside the reserved range get added there and **MUST NOT** re-use numbers.

**Rationale:** The same reasoning as in `smctl-logging-v1`: MSGIDs are a contract with log consumers; the range allocation makes it immediately obvious which component emitted an event; and pre-declaring the common events avoids per-tool improvisation as the server surface grows.

### Decision 6: Subscriber ownership — smctl-log owns initialization, smctl-mcp only emits

The MCP server does **not** install its own `tracing-subscriber`. `smctl serve --mcp` calls `smctl_log::init` the same way every other CLI path does; the MCP crate only uses `tracing::info!` / `tracing::error!` with canonical MSGIDs.

**Rationale:** A second subscriber registration would either fail (global default already set) or silently swallow events. Consolidating ownership in `smctl-log` keeps one place responsible for RFC 5424 conformance and transport routing. The MCP crate is a log producer, not a log consumer.

**Alternative considered:** Let `smctl-mcp` add its own transport (e.g. ship tool-call logs to the client as MCP notifications). Interesting but out of scope — deferred to a later change if demand arises.

### Decision 7: Tool descriptions and error payloads follow design-system-v1 voice

All human-readable strings the server emits — tool descriptions exposed via `tools/list`, resource summaries, human-readable portions of error payloads — conform to the voice contract declared in `design-system-v1/specs/design-system.md`. Concretely:

- Sentence case, not Title Case.
- Address the operator as `you`, never `we`.
- Imperative verbs for action descriptions.
- No emoji, no forbidden Unicode pictographs.
- Error messages are three-part with an executable `smctl` remediation (same rubric as CLI errors from `smctl-errors-v1`).

**Rationale:** AI assistants consume these strings and often relay them verbatim to the user. Consistency with the rest of `smctl` means the voice doesn't shift when the operator crosses from the terminal to an assistant-mediated flow.

**What this DOES NOT require:** The MCP tool input-schema `description` fields are machine-facing and may be terser; the voice rules apply to human-readable output surfaces.

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| `rmcp` API instability | Pin to specific version; abstract behind trait if needed |
| Async runtime conflicts | smctl currently uses sync code; MCP server needs tokio. Use `tokio::task::spawn_blocking` for sync calls |
| Resource subscription overhead | Only emit update notifications when workspace state actually changes; debounce rapid changes |
| Large tool surface area | Start with core tools (workspace, flow, spec, build); defer gate tools to smctl-gate-v1 |

## Open Questions

1. Should the MCP server support hot-reload of workspace config?
2. Should tool results include timing metadata for performance observability?
