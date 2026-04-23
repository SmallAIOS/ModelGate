//! Integration test: drive `smctl-mcp` over SSE / streamable HTTP with a real
//! rmcp client.
//!
//! Binds [`smctl_mcp::serve_sse`] to `127.0.0.1:0` so the kernel picks a free
//! port, then connects an rmcp streamable-HTTP client, drives an `initialize`
//! handshake, and exercises one tool-call round-trip against a freshly-inited
//! workspace. Handshake success plus a `repos` payload proves the SSE
//! transport carries the same tool surface as stdio.
//!
//! The transport runs on a background tokio task and is torn down by
//! dropping its cancellation token — no external process is spawned.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, ClientInfo};
use rmcp::transport::StreamableHttpClientTransport;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;

/// Spin up the SSE server on a kernel-assigned port; return the full `/mcp`
/// URL and a cancellation token the test can use to tear down the listener.
///
/// This mirrors the flow in [`smctl_mcp::serve_sse`], but inlines `bind` and
/// the axum serve-loop so we can capture the assigned port without reaching
/// into the private `listener.local_addr()` after the function has moved it.
async fn start_sse_server(root: PathBuf) -> anyhow::Result<(String, CancellationToken)> {
    let cancel = CancellationToken::new();
    let server = smctl_mcp::SmctlServer::new(root);

    let service = StreamableHttpService::new(
        move || Ok(server.clone()),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default().with_cancellation_token(cancel.child_token()),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let addr: SocketAddr = ([127, 0, 0, 1], 0).into();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    let url = format!("http://127.0.0.1:{}/mcp", local_addr.port());

    let serve_cancel = cancel.clone();
    tokio::spawn(async move {
        let _ = axum::serve(listener, router)
            .with_graceful_shutdown(async move { serve_cancel.cancelled().await })
            .await;
    });

    // Give the listener a beat to finish entering the accept loop. 50ms is
    // generous; the first connect will otherwise race the accept.
    tokio::time::sleep(Duration::from_millis(50)).await;

    Ok((url, cancel))
}

#[tokio::test]
async fn sse_initialize_and_call_workspace_status() -> anyhow::Result<()> {
    // Arrange: a temp workspace with a minimal manifest and no repos.
    let tmp = tempfile::tempdir()?;
    let root = tmp.path().to_path_buf();
    smctl_workspace::init_workspace(&root, "mcp-sse-test-workspace")?;

    let (url, cancel) = start_sse_server(root).await?;

    // Act: connect an rmcp streamable-HTTP client.
    let transport = StreamableHttpClientTransport::from_config(
        StreamableHttpClientTransportConfig::with_uri(url),
    );
    let client = ClientInfo::default().serve(transport).await?;

    // `initialize` is driven implicitly by `serve`. The negotiated peer info
    // must advertise the `tools` capability and identify the server as
    // `smctl` — parity with the stdio-handshake assertions.
    let peer_info = client
        .peer_info()
        .expect("peer_info available after initialize");
    assert!(
        peer_info.capabilities.tools.is_some(),
        "initialize response must advertise tools capability"
    );
    assert_eq!(peer_info.server_info.name, "smctl");

    // Call `smctl_workspace_status` with no arguments; default workspace is
    // the one the server was started with.
    let result = client
        .call_tool(CallToolRequestParams::new("smctl_workspace_status"))
        .await?;
    assert!(
        !result.is_error.unwrap_or(false),
        "call_tool reported an error payload: {result:?}"
    );
    let text = result
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("tool result should carry text content");
    let json: serde_json::Value = serde_json::from_str(text)?;
    assert!(
        json.get("repos").is_some(),
        "expected repos field in {text}"
    );

    // Tear down: cancel the client, then the server.
    let _ = client.cancel().await;
    cancel.cancel();

    Ok(())
}
