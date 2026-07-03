//! Integration test: drive `smctl-mcp` over stdio with a real rmcp client.
//!
//! Exercises the initialize handshake plus the tool-surface contract:
//! every tool in `EXPECTED_TOOLS` must be listed, and representative
//! tools from each family must respond to `tools/call`. Also covers
//! the resource surface: `resources/list`, `resources/templates/list`,
//! and `resources/read` for every static URI plus the templated
//! spec-tasks URI.

use std::path::PathBuf;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, ClientInfo, ReadResourceRequestParams, ResourceContents};

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
    "smctl_verify_policy",
    "smctl_verify_model",
    "smctl_verify_proof",
    "smctl_verify_protocol",
    "smctl_verify_discover",
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
    // empty specs array (the aggregating library returns
    // Ok(vec![]) when no registered repo declares a spec).
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
    assert!(
        specs_json["specs"]
            .as_array()
            .map(|a| a.is_empty())
            .unwrap_or(false),
        "fresh fixture should have zero specs, got {specs_text}"
    );

    // Aggregating sanity: scaffold a real spec via smctl_spec_new
    // (no `repo` set — defaults to the synthetic `_workspace` repo),
    // then re-list and assert the new entry carries the `repo` field.
    let new_resp = client
        .call_tool(
            CallToolRequestParams::new("smctl_spec_new").with_arguments(
                serde_json::json!({ "name": "mcp-aggregate-fixture" })
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
        )
        .await?;
    assert!(
        !new_resp.is_error.unwrap_or(false),
        "smctl_spec_new reported an error payload: {new_resp:?}"
    );
    let listed = client
        .call_tool(CallToolRequestParams::new("smctl_spec_list"))
        .await?;
    let listed_text = listed
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("smctl_spec_list (post-new) should carry text content");
    let listed_json: serde_json::Value = serde_json::from_str(listed_text)?;
    let entries = listed_json["specs"].as_array().expect("specs is an array");
    let aggregated = entries
        .iter()
        .find(|e| e.get("name").and_then(|v| v.as_str()) == Some("mcp-aggregate-fixture"))
        .expect("the new spec should appear in the aggregated list");
    assert_eq!(
        aggregated.get("repo").and_then(|v| v.as_str()),
        Some("_workspace"),
        "aggregated entry should carry the synthetic _workspace repo: {aggregated}"
    );

    // Validate via the qualified `repo:name` form. The response
    // wraps the ValidationResult with a top-level `qualified`
    // string for traceability.
    let validate_resp = client
        .call_tool(
            CallToolRequestParams::new("smctl_spec_validate").with_arguments(
                serde_json::json!({ "name": "_workspace:mcp-aggregate-fixture" })
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
        )
        .await?;
    assert!(
        !validate_resp.is_error.unwrap_or(false),
        "smctl_spec_validate reported an error payload: {validate_resp:?}"
    );
    let validate_text = validate_resp
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("smctl_spec_validate should carry text content");
    let validate_json: serde_json::Value = serde_json::from_str(validate_text)?;
    assert_eq!(
        validate_json.get("qualified").and_then(|v| v.as_str()),
        Some("_workspace:mcp-aggregate-fixture"),
        "validate envelope should carry the qualified name: {validate_text}"
    );

    // Bare-name not-found returns an error envelope whose message
    // includes the three-part remediation pointing at smctl spec list.
    let missing_resp = client
        .call_tool(
            CallToolRequestParams::new("smctl_spec_validate").with_arguments(
                serde_json::json!({ "name": "definitely-not-a-spec" })
                    .as_object()
                    .cloned()
                    .unwrap(),
            ),
        )
        .await;
    match missing_resp {
        Ok(result) => {
            assert!(
                result.is_error.unwrap_or(false),
                "smctl_spec_validate on a missing name should error, got {result:?}"
            );
        }
        Err(e) => {
            let msg = format!("{e}");
            assert!(
                msg.contains("smctl spec list"),
                "missing-spec error should suggest `smctl spec list`: {msg}"
            );
        }
    }

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

    // Verify family: discover lists every shipped verifier and
    // reports kind=found for Cedar (Rust dep) regardless of host
    // tooling. policy with no [verify.policy] sources returns an
    // empty source list with outcome=no_sources.
    let verify_discover = client
        .call_tool(CallToolRequestParams::new("smctl_verify_discover"))
        .await?;
    assert!(
        !verify_discover.is_error.unwrap_or(false),
        "smctl_verify_discover reported an error payload: {verify_discover:?}"
    );
    let discover_text = verify_discover
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("smctl_verify_discover should carry text content");
    let discover_json: serde_json::Value = serde_json::from_str(discover_text)?;
    let entries = discover_json
        .as_array()
        .expect("discover returns a JSON array");
    assert_eq!(
        entries.len(),
        4,
        "expected 4 verifier entries (policy/model/proof/protocol) in {discover_text}"
    );
    let policy_entry = entries
        .iter()
        .find(|e| e.get("verifier").and_then(|v| v.as_str()) == Some("policy"))
        .expect("policy entry");
    assert_eq!(
        policy_entry
            .pointer("/discovery/kind")
            .and_then(|v| v.as_str()),
        Some("found"),
        "Cedar should always be Found (Rust dep): {policy_entry}"
    );

    let verify_policy = client
        .call_tool(CallToolRequestParams::new("smctl_verify_policy"))
        .await?;
    assert!(
        !verify_policy.is_error.unwrap_or(false),
        "smctl_verify_policy reported an error payload: {verify_policy:?}"
    );
    let policy_text = verify_policy
        .content
        .first()
        .and_then(|c| c.raw.as_text())
        .map(|t| t.text.as_str())
        .expect("smctl_verify_policy should carry text content");
    let policy_json: serde_json::Value = serde_json::from_str(policy_text)?;
    assert_eq!(
        policy_json.get("verifier").and_then(|v| v.as_str()),
        Some("policy")
    );
    assert_eq!(
        policy_json.get("outcome").and_then(|v| v.as_str()),
        Some("no_sources"),
        "no [verify.policy] in fixture manifest -> outcome=no_sources, got {policy_text}"
    );

    // Each remaining verb runs through its own #[tool] handler and
    // through SmctlServer::run_verifier with a different verb name —
    // exercise all three even though they short-circuit identically
    // (no [verify.<verb>] section -> no_sources, OR tool_missing if
    // the runner happens to have tlc/lake/spin installed).
    for verb_tool in [
        "smctl_verify_model",
        "smctl_verify_proof",
        "smctl_verify_protocol",
    ] {
        let resp = client
            .call_tool(CallToolRequestParams::new(verb_tool))
            .await?;
        assert!(
            !resp.is_error.unwrap_or(false),
            "{verb_tool} reported an error payload: {resp:?}"
        );
        let body_text = resp
            .content
            .first()
            .and_then(|c| c.raw.as_text())
            .map(|t| t.text.as_str())
            .unwrap_or_else(|| panic!("{verb_tool} should carry text content"));
        let body: serde_json::Value = serde_json::from_str(body_text)?;
        let outcome = body
            .get("outcome")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        assert!(
            ["no_sources", "tool_missing"].contains(&outcome),
            "{verb_tool} outcome should be no_sources or tool_missing on a fresh fixture; got '{outcome}' in {body_text}"
        );
    }

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

    // Resources capability should be advertised on initialize.
    assert!(
        peer_info.capabilities.resources.is_some(),
        "initialize response must advertise resources capability"
    );

    // `resources/list` must include every static URI.
    let resources = client.list_resources(Default::default()).await?;
    let resource_uris: Vec<String> = resources
        .resources
        .iter()
        .map(|r| r.raw.uri.clone())
        .collect();
    for expected in [
        "smctl://workspace/config",
        "smctl://workspace/status",
        "smctl://flow/branches",
        "smctl://spec/list",
    ] {
        assert!(
            resource_uris.iter().any(|u| u == expected),
            "resource list must include {expected}, got {resource_uris:?}"
        );
    }

    // Every static resource should declare a MIME type.
    for resource in &resources.resources {
        assert!(
            resource.raw.mime_type.is_some(),
            "resource {} missing mime_type advertisement",
            resource.raw.uri
        );
    }

    // `resources/templates/list` must include the spec-tasks template.
    let templates = client.list_resource_templates(Default::default()).await?;
    let template_uris: Vec<String> = templates
        .resource_templates
        .iter()
        .map(|t| t.raw.uri_template.clone())
        .collect();
    assert!(
        template_uris
            .iter()
            .any(|u| u == "smctl://spec/{name}/tasks"),
        "template list must include smctl://spec/{{name}}/tasks, got {template_uris:?}"
    );

    // `resources/read` round-trip: workspace config returns TOML.
    let config = client
        .read_resource(ReadResourceRequestParams::new("smctl://workspace/config"))
        .await?;
    let config_contents = config.contents.first().expect("config read has contents");
    let (config_text, config_mime) = match config_contents {
        ResourceContents::TextResourceContents {
            text, mime_type, ..
        } => (text.as_str(), mime_type.as_deref()),
        _ => panic!("expected text contents for workspace config, got {config_contents:?}"),
    };
    assert_eq!(config_mime, Some("application/toml"));
    assert!(
        config_text.contains("mcp-test-workspace"),
        "workspace config should include the workspace name, got: {config_text}"
    );

    // `resources/read` round-trip: workspace status returns JSON with `repos`.
    let status = client
        .read_resource(ReadResourceRequestParams::new("smctl://workspace/status"))
        .await?;
    let status_text = text_of(&status.contents);
    let status_json: serde_json::Value = serde_json::from_str(status_text)?;
    assert!(
        status_json.get("repos").is_some(),
        "expected repos field in workspace status, got: {status_text}"
    );
    assert_mime(&status.contents, "application/json");

    // `resources/read` round-trip: flow branches returns grouped JSON.
    let flow_branches = client
        .read_resource(ReadResourceRequestParams::new("smctl://flow/branches"))
        .await?;
    let flow_text = text_of(&flow_branches.contents);
    let flow_json: serde_json::Value = serde_json::from_str(flow_text)?;
    for key in ["features", "releases", "hotfixes"] {
        assert!(
            flow_json.get(key).is_some(),
            "expected {key} field in flow branches, got: {flow_text}"
        );
    }
    assert_mime(&flow_branches.contents, "application/json");

    // `resources/read` round-trip: spec list returns empty array for
    // a workspace with no openspec directory.
    let spec_list = client
        .read_resource(ReadResourceRequestParams::new("smctl://spec/list"))
        .await?;
    let specs_text = text_of(&spec_list.contents);
    let specs_json: serde_json::Value = serde_json::from_str(specs_text)?;
    assert!(
        specs_json.get("specs").is_some(),
        "expected specs field in spec list, got: {specs_text}"
    );

    // Scaffold one spec so the templated URI has real data to serve.
    let openspec_dir = root.join("openspec");
    smctl_spec::new_spec(&openspec_dir, "tasks-roundtrip-spec")?;

    let spec_tasks = client
        .read_resource(ReadResourceRequestParams::new(
            "smctl://spec/tasks-roundtrip-spec/tasks",
        ))
        .await?;
    let tasks_text = text_of(&spec_tasks.contents);
    let tasks_json: serde_json::Value = serde_json::from_str(tasks_text)?;
    assert_eq!(
        tasks_json.get("name").and_then(|v| v.as_str()),
        Some("tasks-roundtrip-spec")
    );
    assert!(
        tasks_json.get("tasks_total").is_some(),
        "expected tasks_total in spec tasks, got: {tasks_text}"
    );
    assert_mime(&spec_tasks.contents, "application/json");

    // Unknown URIs should return a resource-not-found error.
    let err = client
        .read_resource(ReadResourceRequestParams::new("smctl://no/such/uri"))
        .await
        .expect_err("unknown URI must error");
    let err_msg = format!("{err:?}");
    assert!(
        err_msg.to_lowercase().contains("no resource registered")
            || err_msg.to_lowercase().contains("not found"),
        "expected resource-not-found error, got: {err_msg}"
    );

    // Tear down cleanly.
    client.cancel().await?;
    server_handle.await??;
    Ok(())
}

/// Pull the single text payload out of a `resources/read` response.
fn text_of(contents: &[ResourceContents]) -> &str {
    match contents
        .first()
        .expect("read response has at least one content")
    {
        ResourceContents::TextResourceContents { text, .. } => text.as_str(),
        other => panic!("expected text contents, got {other:?}"),
    }
}

/// Assert the single content block carries the expected MIME type.
fn assert_mime(contents: &[ResourceContents], expected: &str) {
    let mime = match contents
        .first()
        .expect("read response has at least one content")
    {
        ResourceContents::TextResourceContents { mime_type, .. } => mime_type.as_deref(),
        ResourceContents::BlobResourceContents { mime_type, .. } => mime_type.as_deref(),
    };
    assert_eq!(
        mime,
        Some(expected),
        "expected mime {expected}, got {mime:?}"
    );
}

/// Minimal client handler — no server-initiated requests expected.
#[derive(Debug, Clone, Default)]
struct ClientHandlerStub;

impl rmcp::ClientHandler for ClientHandlerStub {
    fn get_info(&self) -> ClientInfo {
        ClientInfo::default()
    }
}
