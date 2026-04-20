//! Integration test: drive `smctl-mcp` over stdio with a real rmcp client.
//!
//! Exercises the initialize handshake plus the tool-surface contract:
//! every tool in `EXPECTED_TOOLS` must be listed, and representative
//! tools from each family must respond to `tools/call`.

use std::path::PathBuf;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, ClientInfo};

/// Every tool smctl-mcp advertises. Extend this list as new tools land.
const EXPECTED_TOOLS: &[&str] = &[
    "smctl_workspace_status",
    "smctl_workspace_init",
    "smctl_workspace_add",
    "smctl_workspace_remove",
    "smctl_workspace_sync",
];

#[tokio::test]
async fn initialize_and_call_workspace_status() -> anyhow::Result<()> {
    // Arrange: a temp workspace with a minimal manifest and no repos.
    let tmp = tempfile::tempdir()?;
    let root: PathBuf = tmp.path().to_path_buf();
    smctl_workspace::init_workspace(&root, "mcp-test-workspace")?;

    // Wire the server and client to an in-memory duplex stream. This
    // stands in for the stdio transport without spawning a subprocess.
    let (server_io, client_io) = tokio::io::duplex(8192);

    let server = smctl_mcp::SmctlServer::new(root.clone());
    let server_handle = tokio::spawn(async move {
        let running = server.serve(server_io).await?;
        running.waiting().await?;
        anyhow::Ok(())
    });

    // Act: drive the client side.
    let client_handler = ClientHandlerStub;
    let client = client_handler.serve(client_io).await?;

    // `initialize` is driven implicitly by `serve` — the peer info it
    // negotiated is now readable and MUST carry `tools` capability.
    let peer_info = client
        .peer_info()
        .expect("peer_info available after initialize");
    assert!(
        peer_info.capabilities.tools.is_some(),
        "initialize response must advertise tools capability"
    );
    assert_eq!(peer_info.server_info.name, "smctl");

    // List tools: every tool in EXPECTED_TOOLS must be advertised.
    let tools = client.list_tools(Default::default()).await?;
    let names: Vec<String> = tools
        .tools
        .iter()
        .map(|t| t.name.clone().into_owned())
        .collect();
    for expected in EXPECTED_TOOLS {
        assert!(
            names.iter().any(|n| n == expected),
            "tool list must include {expected}, got {names:?}"
        );
    }

    // Call the tool with no arguments — default workspace is the one
    // the server was started with.
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

    // Tear down cleanly.
    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

/// Minimal client handler — no server-initiated requests expected.
#[derive(Debug, Clone, Default)]
struct ClientHandlerStub;

impl rmcp::ClientHandler for ClientHandlerStub {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }
}
