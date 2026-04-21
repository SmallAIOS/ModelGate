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
//! - the full tool surface across workspace, worktree, flow, spec, and
//!   build families (see `server.rs` for the catalog)
//! - five read-only resources under the `smctl://` scheme, including a
//!   templated `smctl://spec/{name}/tasks` URI
//! - MSGID emission at the catalog sites declared in
//!   `smctl-mcp-v1/specs/mcp-server-impl.md`
//!
//! SSE and streamable-HTTP transports are deferred. See `tasks.md` for
//! the follow-up list.

mod resources;
mod server;

pub use server::SmctlServer;

use std::path::PathBuf;

/// Which transport to bind for `smctl serve --mcp`.
///
/// v1 only wires [`Transport::Stdio`]; other variants exist to keep the
/// CLI flag surface stable as new transports come online (see
/// `smctl-mcp-v1/tasks.md` — "Spec drift and follow-ups").
#[derive(Debug, Clone, Copy)]
pub enum Transport {
    /// stdin / stdout — the default for Claude Code, Cursor, Windsurf.
    Stdio,
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
    }
}
