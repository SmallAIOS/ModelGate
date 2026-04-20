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
    "smctl_worktree_add",
    "smctl_worktree_list",
    "smctl_worktree_remove",
    "smctl_flow_init",
    "smctl_flow_feature_start",
    "smctl_flow_feature_finish",
    "smctl_flow_release_start",
    "smctl_flow_release_finish",
    "smctl_flow_hotfix_start",
    "smctl_flow_hotfix_finish",
    "smctl_spec_new",
    "smctl_spec_validate",
    "smctl_spec_archive",
    "smctl_spec_list",
    "smctl_build",
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

    // Worktree family: list with no worktrees returns an empty set.
    let wt = client
        .call_tool(CallToolRequestParams::new("smctl_worktree_list"))
        .await?;
    assert!(
        !wt.is_error.unwrap_or(false),
        "smctl_worktree_list reported an error payload: {wt:?}"
    );
    let wt_text = wt
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("smctl_worktree_list should carry text content");
    let wt_json: serde_json::Value = serde_json::from_str(wt_text)?;
    assert!(
        wt_json.get("sets").is_some(),
        "expected sets field in {wt_text}"
    );

    // Spec family: list on a manifest with no openspec dir returns an
    // empty specs array (the library returns Ok(vec![]) when the
    // directory is absent).
    let specs = client
        .call_tool(CallToolRequestParams::new("smctl_spec_list"))
        .await?;
    assert!(
        !specs.is_error.unwrap_or(false),
        "smctl_spec_list reported an error payload: {specs:?}"
    );
    let specs_text = specs
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("smctl_spec_list should carry text content");
    let specs_json: serde_json::Value = serde_json::from_str(specs_text)?;
    assert!(
        specs_json.get("specs").is_some(),
        "expected specs field in {specs_text}"
    );

    // Build family: with an empty workspace, build reports an empty
    // results array and all_passed=true.
    let build = client
        .call_tool(CallToolRequestParams::new("smctl_build"))
        .await?;
    assert!(
        !build.is_error.unwrap_or(false),
        "smctl_build reported an error payload: {build:?}"
    );
    let build_text = build
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("smctl_build should carry text content");
    let build_json: serde_json::Value = serde_json::from_str(build_text)?;
    assert!(
        build_json.get("results").is_some(),
        "expected results field in {build_text}"
    );

    // Flow family: init with no repos completes and returns a
    // FlowResult with an empty repos array.
    let flow = client
        .call_tool(CallToolRequestParams::new("smctl_flow_init"))
        .await?;
    assert!(
        !flow.is_error.unwrap_or(false),
        "smctl_flow_init reported an error payload: {flow:?}"
    );
    let flow_text = flow
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("smctl_flow_init should carry text content");
    let flow_json: serde_json::Value = serde_json::from_str(flow_text)?;
    assert_eq!(
        flow_json.get("operation").and_then(|v| v.as_str()),
        Some("flow init"),
        "expected operation=flow init in {flow_text}"
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
