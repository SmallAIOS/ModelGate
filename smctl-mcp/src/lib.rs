//! `smctl-mcp` — MCP server surface for smctl.
//!
//! This crate is a **log producer**, not a log consumer: every tracing
//! event attaches an MSGID from [`smctl_log::MsgId`], but the crate does
//! not install its own `tracing-subscriber`. The `smctl` binary calls
//! [`smctl_log::init`] once before starting the transport
//! (`smctl-mcp-v1/design.md` Decision 6).
//!
//! The v1 vertical slice ships:
//!
//! - stdio transport (via `rmcp::transport::stdio`)
//! - SSE / streamable-HTTP transport (via
//!   `rmcp::transport::streamable_http_server::StreamableHttpService`
//!   wrapped in an axum router) — see [`serve_sse`]
//! - the full tool surface across workspace, worktree, flow, spec, and
//!   build families (see `server.rs` for the catalog)
//! - five read-only resources under the `smctl://` scheme, including a
//!   templated `smctl://spec/{name}/tasks` URI
//! - MSGID emission at the catalog sites declared in
//!   `smctl-mcp-v1/specs/mcp-server-impl.md`

mod resources;
mod server;

pub use server::{SmctlServer, serve_sse, serve_stdio};

use std::net::SocketAddr;
use std::path::PathBuf;

/// Which transport to bind for `smctl serve --mcp`.
///
/// [`Transport::Stdio`] stays the default — that is what Claude Code,
/// Cursor, and Windsurf drive. [`Transport::Sse`] exposes the same tool
/// and resource surface over HTTP using rmcp's streamable-HTTP server
/// (SSE response framing plus JSON POSTs on a single endpoint).
#[derive(Debug, Clone, Copy)]
pub enum Transport {
    /// stdin / stdout — the default for Claude Code, Cursor, Windsurf.
    Stdio,
    /// SSE over HTTP. Binds a TCP listener at `addr` and serves the MCP
    /// streamable-HTTP endpoint at `/mcp`.
    Sse {
        /// Socket address to bind. Prefer loopback-only for local use;
        /// the server enforces loopback `Host` validation by default
        /// (see rmcp's DNS-rebinding guard).
        addr: SocketAddr,
    },
}

/// Start the MCP server on the given transport and block until the
/// transport closes.
///
/// The caller MUST have already called [`smctl_log::init`]. This
/// function never installs a subscriber; doing so from inside
/// `smctl-mcp` would either panic (global default already set) or
/// silently drop events.
pub async fn start_server(workspace_root: PathBuf, transport: Transport) -> anyhow::Result<()> {
    match transport {
        Transport::Stdio => server::serve_stdio(workspace_root).await,
        Transport::Sse { addr } => server::serve_sse(workspace_root, addr).await,
    }
}
