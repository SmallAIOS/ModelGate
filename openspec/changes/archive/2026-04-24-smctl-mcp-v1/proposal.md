# smctl MCP Server — Proposal

## Why

`smctl` v0.1.3 provides a complete CLI for managing SmallAIOS multi-repo workspaces: workspace management, git flow, OpenSpec workflow, and build orchestration. However, all of these capabilities are only accessible via terminal commands. AI coding assistants — Claude Code, Cursor, Windsurf, Cline — cannot programmatically interact with the SmallAIOS workspace, branching model, or spec lifecycle.

The Model Context Protocol (MCP) is the emerging standard for tool integration with AI coding assistants. By exposing `smctl` as an MCP server, AI agents can autonomously manage workspaces, create feature branches, scaffold specs, and trigger builds without the user manually typing CLI commands.

Key problems this solves:

- **AI assistants are disconnected from the workflow.** They can write code but cannot create branches, start specs, or trigger builds through structured tool calls.
- **Context switching between terminal and editor.** Developers must relay workspace state between their AI assistant and `smctl` manually.
- **No structured workspace context for AI.** Assistants lack visibility into repo status, active branches, spec progress, and build state.

## What Changes

Add `smctl serve --mcp` command that starts an MCP server exposing all existing `smctl` capabilities as MCP tools and resources. The server supports three transport modes:

- **stdio** — Standard transport for local AI tools (Claude Code, Cursor)
- **SSE** — Server-Sent Events for web-based or remote assistants
- **Streamable HTTP** — Newer MCP transport for bidirectional HTTP

### New Crate

- **`smctl-mcp`** — MCP server crate using the official Rust MCP SDK (`rmcp`). Implements `ServerHandler` trait, registers all tools and resources, manages transport lifecycle.

### New CLI Surface

- `smctl serve --mcp --stdio` — Start MCP server on stdin/stdout
- `smctl serve --mcp --sse --port <port>` — Start MCP server with SSE transport
- `smctl serve --mcp --http --port <port>` — Start MCP server with streamable HTTP

### MCP Tools (1:1 with CLI subcommands)

Every existing CLI subcommand becomes an MCP tool: `smctl_workspace_init`, `smctl_workspace_status`, `smctl_worktree_add`, `smctl_flow_feature_start`, `smctl_spec_new`, `smctl_build`, etc.

### MCP Resources (read-only context)

Resources provide AI assistants with workspace state: `smctl://workspace/config`, `smctl://workspace/status`, `smctl://spec/{name}/tasks`, `smctl://flow/branches`, etc. Resources support subscription for real-time updates.

## Capabilities

### New Capabilities

- `smctl-mcp` crate — MCP server implementation with `rmcp` SDK
- `smctl serve` CLI subcommand — Server lifecycle management
- MCP tool registry — All workspace/flow/spec/build tools
- MCP resource registry — Read-only workspace context
- stdio, SSE, and streamable HTTP transports

### Modified Capabilities

- `smctl` CLI binary — Add `serve` subcommand to existing command tree
- Core library functions — Ensure all return structured results suitable for JSON serialization

## Impact

### New Files

```
smctl-mcp/
├── Cargo.toml
└── src/
    ├── lib.rs          # ServerHandler impl, tool/resource registration
    ├── tools.rs        # MCP tool definitions and handlers
    ├── resources.rs    # MCP resource definitions and providers
    └── transport.rs    # Transport configuration (stdio/SSE/HTTP)
```

### Modified Files

- `Cargo.toml` — Add `smctl-mcp` workspace member
- `smctl/Cargo.toml` — Add `smctl-mcp` dependency
- `smctl/src/main.rs` — Add `serve` subcommand

### Dependencies

- `rmcp` (0.8+) — Official Rust MCP SDK with server, stdio, and SSE features
- `tokio` (1.x) — Async runtime for MCP server
- `schemars` (1.x) — JSON Schema generation for tool input schemas
- `axum` — HTTP server for SSE transport
- `smctl-log` (in-tree) — RFC 5424 subscriber and MSGID catalog; the MCP server reserves the `SMCTL-0200`–`SMCTL-0299` range per `smctl-logging-v1/specs/logging.md`

## Cross-Cutting Contracts

This change inherits three declared contracts from prior changes. Adherence is **MUST**, not **SHOULD**.

- **Voice and lexicon** (`design-system-v1/specs/design-system.md`). Tool descriptions, resource summaries, error payloads, and any human-readable text emitted by the MCP server conform to sentence case, imperative verb forms, the canonical status vocabulary, and the three-part error rubric (what happened → what it means → executable remediation). No emoji. No forbidden Unicode pictographs.
- **Logging** (`smctl-logging-v1/specs/logging.md`). Any tracing event emitted from `smctl-mcp` uses a MSGID from the `SMCTL-0200..0299` reserved range. This change defines the initial allocations in `design.md` and the catalog in `specs/mcp-server-impl.md`. The `smctl-log::init` subscriber is installed as part of `smctl serve --mcp` startup; the server does **not** install its own formatter.
- **Error handling.** Rust errors follow the same three-part remediation rubric as CLI errors (per `smctl-errors-v1`). When a tool call returns an MCP error payload, the payload's `message` field carries the structured error; if a remediation is offered, it names a real `smctl` subcommand.

## References

- [Model Context Protocol specification](https://spec.modelcontextprotocol.io)
- [rmcp — Official Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [MCP server spec (from smctl-tool-v1)](../smctl-tool-v1/specs/mcp-server.md)
- [smctl design document](../smctl-tool-v1/design.md) — Decisions 10 and 11
- [design-system-v1 — voice and lexicon contract](../design-system-v1/specs/design-system.md)
- [smctl-logging-v1 — MSGID catalog and severity mapping](../smctl-logging-v1/specs/logging.md)
- [smctl-errors-v1 — three-part error rubric](../smctl-errors-v1/design.md)
