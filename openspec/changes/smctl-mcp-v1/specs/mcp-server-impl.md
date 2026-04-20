# smctl MCP Server — Implementation Specification

## Overview

This spec covers the implementation details for the `smctl-mcp` crate. The protocol-level specification (tool definitions, resource URIs, transport modes, error codes) is defined in [`smctl-tool-v1/specs/mcp-server.md`](../../smctl-tool-v1/specs/mcp-server.md). This document focuses on the Rust implementation architecture.

## Inherited contracts

This implementation MUST satisfy three contracts declared in earlier changes:

- [`design-system-v1/specs/design-system.md`](../../design-system-v1/specs/design-system.md) — voice, lexicon, error-message rubric, forbidden Unicode pictographs. Applies to tool descriptions, resource summaries, and human-readable error payloads.
- [`smctl-logging-v1/specs/logging.md`](../../smctl-logging-v1/specs/logging.md) — RFC 5424 wire format, MSGID catalog, severity mapping, transport selection. `smctl-mcp` is a log **producer**; it never installs its own subscriber.
- [`smctl-errors-v1/design.md`](../../smctl-errors-v1/design.md) — three-part error rubric (fact → meaning → executable remediation). Every error payload `message` field follows this shape.

## MSGID catalog for smctl-mcp

Allocated from the `SMCTL-0200`–`SMCTL-0299` range reserved in `smctl-logging-v1`. MSGIDs are immutable once published here.

| MSGID | Enum variant | Severity | STRUCTURED-DATA keys | Emitted when |
|---|---|---|---|---|
| `SMCTL-0200` | `McpServerStarted` | Informational | `transport`, `listen_addr` (for sse/http) | `ServerHandler::initialize` completes and the transport is bound |
| `SMCTL-0201` | `McpServerStopped` | Informational | `reason` | graceful shutdown path fires |
| `SMCTL-0202` | `McpToolCalled` | Informational | `tool`, `request_id`, `caller_name` | a `tools/call` request arrives |
| `SMCTL-0203` | `McpToolCompleted` | Informational | `tool`, `request_id`, `duration_ms` | tool call returns a success payload |
| `SMCTL-0204` | `McpToolFailed` | Error | `tool`, `request_id`, `duration_ms`, `error_kind`, `remediation` | tool call returns an error payload |
| `SMCTL-0205` | `McpClientDisconnected` | Warning | `transport`, `reason` | client transport closes unexpectedly |
| `SMCTL-0206` | `McpTransportFatal` | Error | `transport`, `error` | transport itself crashes (e.g. SSE listener panics) |

All STRUCTURED-DATA keys are snake_case ASCII. Numeric values render as decimal strings. `request_id` is the MCP request ID as a string, stable within a session.

Adding a new MSGID in this range:

1. Add the `MsgId::McpFoo` variant in `smctl-log/src/msgid.rs` with its code.
2. Add a row to this table.
3. Run `cargo test -p smctl-log` to pick up the updated catalog-size assertion (if present).
4. Update the emission sites to use the new variant.

MSGIDs **MUST NOT** be renumbered or repurposed. A mistake is corrected by allocating a new MSGID and deprecating the old one in a successor change; not by editing this table in place after merge.

## Crate Structure

```
smctl-mcp/
├── Cargo.toml
└── src/
    ├── lib.rs          # Public API: SmctlServer, start_server()
    ├── server.rs       # ServerHandler impl with #[tool_handler]
    ├── tools.rs        # Tool input/output types (schemars-derived)
    ├── resources.rs    # Resource provider trait and implementations
    └── transport.rs    # Transport enum and startup logic
```

## Cargo.toml

```toml
[package]
name = "smctl-mcp"
version = "0.1.0"
edition = "2024"

[dependencies]
rmcp = { version = "0.8", features = ["server", "transport-stdio", "transport-sse"] }
tokio = { version = "1", features = ["full"] }
schemars = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"

# Internal dependencies
smctl-workspace = { path = "../smctl-workspace" }
smctl-flow = { path = "../smctl-flow" }
smctl-spec = { path = "../smctl-spec" }
smctl-build = { path = "../smctl-build" }
```

## SmctlServer

```rust
use rmcp::handler::server::tool_handler;
use rmcp::model::ServerCapabilities;
use std::path::PathBuf;

pub struct SmctlServer {
    workspace_root: PathBuf,
}

impl SmctlServer {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }
}

#[tool_handler]
impl ServerHandler for SmctlServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            name: "smctl".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            capabilities: ServerCapabilities {
                tools: Some(Default::default()),
                resources: Some(ResourcesCapability {
                    subscribe: Some(true),
                    list_changed: Some(true),
                }),
                logging: Some(Default::default()),
                ..Default::default()
            },
        }
    }

    // Tools defined via #[tool] attribute — see tools section below
}
```

## Tool Adapter Pattern

Each MCP tool is a thin adapter that:
1. Deserializes MCP tool input → Rust struct
2. Calls the existing smctl-* library function
3. Serializes the result → JSON `CallToolResult`

```rust
#[tool(description = "Show status of all repos in the workspace")]
async fn workspace_status(&self) -> CallToolResult {
    let result = smctl_workspace::status(&self.workspace_root)?;
    let json = serde_json::to_string_pretty(&result)?;
    Ok(CallToolResult::text(json))
}
```

For operations that may be long-running (build, workspace sync), use `tokio::task::spawn_blocking` to avoid blocking the async runtime:

```rust
#[tool(description = "Build repos in dependency order")]
async fn build(&self, parallel: Option<bool>) -> CallToolResult {
    let root = self.workspace_root.clone();
    let result = tokio::task::spawn_blocking(move || {
        smctl_build::build(&root, parallel.unwrap_or(false))
    }).await??;
    Ok(CallToolResult::text(serde_json::to_string_pretty(&result)?))
}
```

## Transport Startup

```rust
pub enum Transport {
    Stdio,
    Sse { port: u16 },
    Http { port: u16 },
}

pub async fn start_server(
    workspace_root: PathBuf,
    transport: Transport,
) -> Result<(), Box<dyn std::error::Error>> {
    let server = SmctlServer::new(workspace_root);

    match transport {
        Transport::Stdio => {
            let service = server.serve(rmcp::transport::stdio()).await?;
            service.waiting().await?;
        }
        Transport::Sse { port } => {
            let addr = format!("0.0.0.0:{port}");
            let service = server.serve(rmcp::transport::SseServer::new(addr)).await?;
            service.waiting().await?;
        }
        Transport::Http { port } => {
            // Streamable HTTP — implement when rmcp adds support
            todo!("Streamable HTTP transport not yet available in rmcp")
        }
    }
    Ok(())
}
```

## Resource Implementation

Resources are read-only views into workspace state. They use a polling model — when a tool mutates state, the server emits a `notifications/resources/updated` notification.

```rust
async fn list_resources(&self) -> Vec<Resource> {
    vec![
        Resource::new("smctl://workspace/config", "Workspace configuration", "application/toml"),
        Resource::new("smctl://workspace/status", "Repo statuses", "application/json"),
        Resource::new("smctl://flow/branches", "Flow branches", "application/json"),
        Resource::new("smctl://spec/list", "Spec list", "application/json"),
    ]
}

async fn read_resource(&self, uri: &str) -> ResourceResult {
    match uri {
        "smctl://workspace/config" => {
            let content = std::fs::read_to_string(
                self.workspace_root.join(".smctl/workspace.toml")
            )?;
            Ok(ResourceResult::text(content, "application/toml"))
        }
        "smctl://workspace/status" => {
            let status = smctl_workspace::status(&self.workspace_root)?;
            Ok(ResourceResult::text(serde_json::to_string(&status)?, "application/json"))
        }
        _ => Err(McpError::resource_not_found(uri)),
    }
}
```

## Error Mapping

smctl errors map to MCP JSON-RPC error codes as defined in the protocol spec:

| smctl error | MCP error code | Message format |
|---|---|---|
| Workspace not initialized | -32001 | "Workspace not initialized at {path}" |
| Git operation failed | -32002 | "Git error: {details}" |
| Spec validation failed | -32003 | "Spec validation failed: {reasons}" |
| Build failed | -32004 | "Build failed for {repo}: {error}" |
| I/O error | -32000 | "I/O error: {details}" |

## Testing Strategy

1. **Unit tests:** Test tool input deserialization and error mapping
2. **Integration tests:** Spawn smctl MCP server on stdio, send JSON-RPC messages, verify responses
3. **Handshake test:** Verify `initialize` → `initialized` exchange produces correct capabilities

```rust
#[tokio::test]
async fn test_mcp_initialize() {
    let (client_transport, server_transport) = rmcp::transport::channel();
    let server = SmctlServer::new(temp_workspace());

    tokio::spawn(async move {
        server.serve(server_transport).await.unwrap().waiting().await.unwrap();
    });

    let client = McpClient::new(client_transport).await.unwrap();
    let info = client.server_info();
    assert_eq!(info.name, "smctl");
    assert!(info.capabilities.tools.is_some());
}
```
