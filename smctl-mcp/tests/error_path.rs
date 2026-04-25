//! Integration test: confirm `SMCTL-0204 McpToolFailed` emits with the
//! full STRUCTURED-DATA contract when a tool call fails.
//!
//! Strategy: spawn `smctl serve --mcp --stdio` as a subprocess pointed at
//! a workspace-less tempdir, drive an initialize + `tools/call` handshake
//! over its stdio by hand (raw JSON-RPC lines so no second rmcp client is
//! needed), wait for the error response, shut the subprocess down, and
//! assert the log file captured `SMCTL-0204` with `tool`, `request_id`,
//! `duration_ms`, `error_kind`, and `remediation` STRUCTURED-DATA fields.
//!
//! Subprocess-with-`--log-file` is the chosen capture path per
//! `tasks.md` — the in-process duplex + global-default subscriber
//! pattern hangs at shutdown (see `tests/logging.rs`), so we deliberately
//! route events through the RFC 5424 file transport that production uses.

use std::io::{BufRead, BufReader, Write};
use std::time::{Duration, Instant};

// `CommandCargoExt::cargo_bin` is deprecated in assert_cmd 2.1+ in favour of
// the `cargo::cargo_bin!` macro, but that macro assumes the binary lives in
// the current package. `smctl` lives in a different workspace crate, so we
// keep the function form.
#[allow(deprecated)]
use assert_cmd::cargo::CommandCargoExt;

#[test]
fn tool_failure_emits_smctl_0204_with_structured_fields() -> anyhow::Result<()> {
    // Arrange: a temp directory with no workspace manifest and a
    // dedicated log file the subprocess will append to.
    let tmp = tempfile::tempdir()?;
    let workspace_root = tmp.path().to_path_buf();
    let log_path = tmp.path().join("smctl.log");

    // Spawn `smctl serve --mcp --stdio --workspace <tmp> --log-file <log>`.
    // stderr is inherited so test failures surface any panic messages;
    // the RFC 5424 events we inspect land in the log file.
    #[allow(deprecated)]
    let mut cmd = std::process::Command::cargo_bin("smctl")?;
    cmd.arg("--workspace")
        .arg(&workspace_root)
        .arg("--log-file")
        .arg(&log_path)
        .args(["serve", "--mcp", "--stdio"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit());

    let mut child = cmd.spawn()?;

    let mut stdin = child.stdin.take().expect("child stdin piped");
    let stdout = child.stdout.take().expect("child stdout piped");
    let mut reader = BufReader::new(stdout);

    // --- Raw JSON-RPC handshake over stdio ---
    //
    // rmcp's stdio transport speaks newline-delimited JSON-RPC. We send
    // `initialize` -> wait for its response -> send `initialized` notif
    // -> send `tools/call` for `smctl_workspace_status` -> read the
    // error response the server sends back.
    let initialize = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "error-path-test", "version": "0.0.0" }
        }
    });
    writeln!(stdin, "{initialize}")?;
    let _init_resp = read_jsonrpc(&mut reader)?;

    let initialized = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    writeln!(stdin, "{initialized}")?;

    // Call the tool against the workspace-less tempdir. `load_from_root`
    // will return a manifest-missing error, which becomes a three-part
    // `ErrorData::internal_error` inside the server handler. The tool
    // router surfaces that as a `tools/call` error response.
    let call = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/call",
        "params": {
            "name": "smctl_workspace_status",
            "arguments": {}
        }
    });
    writeln!(stdin, "{call}")?;
    let err_resp = read_jsonrpc(&mut reader)?;

    // Dispatch-level failures come back as a JSON-RPC error object;
    // tool-level errors come back as `result.isError = true`. Either
    // shape is fine — both paths hit the SMCTL-0204 emission site.
    let has_failure = err_resp.get("error").is_some()
        || err_resp
            .get("result")
            .and_then(|r| r.get("isError"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
    assert!(
        has_failure,
        "expected tools/call to fail against workspace-less tempdir, got: {err_resp}"
    );

    // --- Shut the server down cleanly ---
    //
    // Dropping stdin closes the stdio read half the server is blocked
    // on; rmcp's `waiting()` then returns and the process exits.
    drop(stdin);

    let _status = wait_with_timeout(&mut child, Duration::from_secs(5))?;

    // --- Inspect the captured log file ---
    //
    // The subscriber flushes on process exit. Poll the file briefly in
    // case the filesystem hasn't surfaced the final buffered write yet.
    let log_contents = read_log_with_retry(&log_path, Duration::from_secs(2))?;

    // `SMCTL-0204` MUST appear, with every STRUCTURED-DATA field the
    // catalog promises. The formatter renders SD as
    // `[SMCTL@32473 tool="..." request_id="..." ...]`, so substring
    // matching on each `key="` is the stable assertion.
    let smctl_0204_line = log_contents
        .lines()
        .find(|l| l.contains("SMCTL-0204"))
        .unwrap_or_else(|| panic!("expected SMCTL-0204 in log file, got:\n{log_contents}"));

    for needle in [
        "tool=\"smctl_workspace_status\"",
        "request_id=\"",
        "duration_ms=\"",
        "error_kind=\"",
        "remediation=\"",
    ] {
        assert!(
            smctl_0204_line.contains(needle),
            "SMCTL-0204 line missing {needle}, full line: {smctl_0204_line}"
        );
    }

    // The `remediation` field MUST name a real `smctl` subcommand per
    // the design-system-v1 error rubric. The static map in server.rs
    // hard-codes `smctl workspace status` for this tool; assert the
    // executable clause is present.
    assert!(
        smctl_0204_line.contains("smctl workspace status"),
        "SMCTL-0204 remediation must name an smctl subcommand, full line: {smctl_0204_line}"
    );

    Ok(())
}

/// Read one newline-delimited JSON-RPC message from the stream. Returns
/// the parsed value so the caller can assert shape.
fn read_jsonrpc<R: BufRead>(reader: &mut R) -> anyhow::Result<serde_json::Value> {
    let mut line = String::new();
    let n = reader.read_line(&mut line)?;
    if n == 0 {
        anyhow::bail!("unexpected EOF reading JSON-RPC message");
    }
    Ok(serde_json::from_str(line.trim())?)
}

/// Wait for a child process to exit, killing it if the deadline passes.
fn wait_with_timeout(
    child: &mut std::process::Child,
    timeout: Duration,
) -> anyhow::Result<std::process::ExitStatus> {
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait()? {
            Some(status) => return Ok(status),
            None => {
                if Instant::now() >= deadline {
                    // Didn't shut down in time — kill so the test does
                    // not hang the runner.
                    let _ = child.kill();
                    return Ok(child.wait()?);
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}

/// Read the full log file, retrying briefly if it is still empty — the
/// subscriber's buffered writes might not yet be visible to this
/// process on slow filesystems.
fn read_log_with_retry(path: &std::path::Path, timeout: Duration) -> anyhow::Result<String> {
    let deadline = Instant::now() + timeout;
    loop {
        match std::fs::read_to_string(path) {
            Ok(contents) if !contents.is_empty() => return Ok(contents),
            Ok(_) | Err(_) => {
                if Instant::now() >= deadline {
                    return Ok(std::fs::read_to_string(path).unwrap_or_default());
                }
                std::thread::sleep(Duration::from_millis(50));
            }
        }
    }
}
