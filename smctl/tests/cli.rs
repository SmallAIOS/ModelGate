//! CLI integration tests for smctl using assert_cmd.

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

#[allow(deprecated)]
fn smctl() -> Command {
    Command::cargo_bin("smctl").unwrap()
}

/// Initialize a workspace at the given path with a git repo.
///
/// Forces `init.defaultBranch=main` and writes `user.email` / `user.name`
/// into the repo's local config, so that library-internal git operations
/// (merge, commit) work on CI runners with no global gitconfig.
fn init_workspace_with_git(root: &Path) {
    let cmds: &[&[&str]] = &[
        &["git", "-c", "init.defaultBranch=main", "init"],
        &["git", "config", "user.email", "test@test.com"],
        &["git", "config", "user.name", "Test"],
    ];
    for cmd in cmds {
        std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .current_dir(root)
            .output()
            .unwrap();
    }
    std::fs::write(root.join("README.md"), "# Test\n").unwrap();
    let cmds: &[&[&str]] = &[
        &["git", "add", "."],
        &[
            "git",
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@test.com",
            "commit",
            "-m",
            "init",
        ],
    ];
    for cmd in cmds {
        std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .current_dir(root)
            .output()
            .unwrap();
    }
}

// ── Basic CLI ────────────────────────────────────────────────────────

#[test]
fn test_help() {
    smctl()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("SmallAIOS control"));
}

#[test]
fn test_version() {
    smctl()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("smctl"));
}

#[test]
fn test_no_args_shows_help() {
    smctl()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"));
}

// ── Workspace commands ───────────────────────────────────────────────

#[test]
fn test_workspace_init() {
    let dir = tempfile::tempdir().unwrap();
    smctl()
        .args(["workspace", "init", "--name", "test-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized workspace 'test-ws'"));

    assert!(dir.path().join(".smctl/workspace.toml").exists());
}

#[test]
fn test_workspace_init_json() {
    let dir = tempfile::tempdir().unwrap();
    smctl()
        .args(["workspace", "init", "--name", "json-ws", "-w"])
        .arg(dir.path())
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\""));
}

#[test]
fn test_workspace_init_dry_run() {
    let dir = tempfile::tempdir().unwrap();
    smctl()
        .args(["workspace", "init", "--name", "dry-ws", "-w"])
        .arg(dir.path())
        .arg("--dry-run")
        .assert()
        // dry-run exits with code 10
        .code(10)
        .stdout(predicate::str::contains("would initialize workspace"));

    // Should NOT create the manifest
    assert!(!dir.path().join(".smctl/workspace.toml").exists());
}

#[test]
fn test_workspace_add_remove() {
    let dir = tempfile::tempdir().unwrap();

    // Init first
    smctl()
        .args(["workspace", "init", "--name", "test-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // Add a repo
    smctl()
        .args([
            "workspace",
            "add",
            "https://example.com/repo.git",
            "--name",
            "my-repo",
            "-w",
        ])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("added repo 'my-repo'"));

    // Remove it
    smctl()
        .args(["workspace", "remove", "my-repo", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("removed repo 'my-repo'"));
}

#[test]
fn test_workspace_status_no_workspace() {
    let dir = tempfile::tempdir().unwrap();
    smctl()
        .args(["workspace", "status", "-w"])
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("failed to read workspace.toml"));
}

// ── Spec commands ────────────────────────────────────────────────────

#[test]
fn test_spec_new_validate_archive() {
    let dir = tempfile::tempdir().unwrap();
    init_workspace_with_git(dir.path());

    // Init workspace
    smctl()
        .args(["workspace", "init", "--name", "spec-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // Create a spec
    smctl()
        .args(["spec", "new", "test-feature", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("created spec 'test-feature'"));

    assert!(
        dir.path()
            .join("openspec/changes/test-feature/proposal.md")
            .exists()
    );
    assert!(
        dir.path()
            .join("openspec/changes/test-feature/design.md")
            .exists()
    );
    assert!(
        dir.path()
            .join("openspec/changes/test-feature/tasks.md")
            .exists()
    );

    // Validate it
    smctl()
        .args(["spec", "validate", "test-feature", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("valid"));

    // Archive it
    smctl()
        .args(["spec", "archive", "test-feature", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("archived spec 'test-feature'"));

    // Original should be gone
    assert!(!dir.path().join("openspec/changes/test-feature").exists());
}

#[test]
fn test_spec_ff() {
    let dir = tempfile::tempdir().unwrap();

    // Init workspace
    smctl()
        .args(["workspace", "init", "--name", "ff-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // Create a spec
    smctl()
        .args(["spec", "new", "ff-test", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // Fast-forward check
    smctl()
        .args(["spec", "ff", "ff-test", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("proposal=present"))
        .stdout(predicate::str::contains("tasks:"));
}

#[test]
fn test_spec_apply() {
    let dir = tempfile::tempdir().unwrap();

    // Init workspace
    smctl()
        .args(["workspace", "init", "--name", "apply-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // Create a spec
    smctl()
        .args(["spec", "new", "apply-test", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // Apply should list tasks
    smctl()
        .args(["spec", "apply", "apply-test", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("pending"));
}

#[test]
fn test_spec_list() {
    let dir = tempfile::tempdir().unwrap();

    smctl()
        .args(["workspace", "init", "--name", "list-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    smctl()
        .args(["spec", "new", "spec-a", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    smctl()
        .args(["spec", "new", "spec-b", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    smctl()
        .args(["spec", "list", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("spec-a"))
        .stdout(predicate::str::contains("spec-b"));
}

// ── Spec duplicate error ─────────────────────────────────────────────

#[test]
fn test_spec_new_duplicate_fails() {
    let dir = tempfile::tempdir().unwrap();

    smctl()
        .args(["workspace", "init", "--name", "dup-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    smctl()
        .args(["spec", "new", "dup-spec", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // Second create should fail.
    // Stable substrings: "already exists" anchors the noun phrase (what happened);
    // "smctl spec archive" anchors the remediation command (what to do next).
    // Both are required by the three-part error rubric in design-system-v1.
    smctl()
        .args(["spec", "new", "dup-spec", "-w"])
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"))
        .stderr(predicate::str::contains("smctl spec archive"));
}

// ── Error-message remediation clauses (smctl-errors-v1) ──────────────
//
// These tests pin the three-part error contract: each error message must
// carry a remediation clause naming a real `smctl` subcommand. Stable
// substring matching keeps wording polish cheap.

#[test]
fn test_spec_not_found_error_names_spec_list() {
    let dir = tempfile::tempdir().unwrap();

    smctl()
        .args(["workspace", "init", "--name", "nf-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // Validating a nonexistent spec must surface the spec-list remediation.
    // Substring `smctl spec list` is the executable remediation clause.
    smctl()
        .args(["spec", "validate", "does-not-exist", "-w"])
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"))
        .stderr(predicate::str::contains("smctl spec list"));
}

#[test]
fn test_workspace_remove_missing_repo_names_status() {
    let dir = tempfile::tempdir().unwrap();

    smctl()
        .args(["workspace", "init", "--name", "rm-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // Removing an unknown repo must point at `smctl workspace status` so the
    // operator can see the configured repo names.
    // Substring `smctl workspace status` is the executable remediation clause.
    smctl()
        .args(["workspace", "remove", "ghost-repo", "-w"])
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found in workspace"))
        .stderr(predicate::str::contains("smctl workspace status"));
}

// ── Config commands ──────────────────────────────────────────────────

#[test]
fn test_config_show() {
    let dir = tempfile::tempdir().unwrap();

    smctl()
        .args(["workspace", "init", "--name", "cfg-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // config show outputs runtime config (not workspace name)
    smctl()
        .args(["config", "show", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("no_color"));
}

// ── Alias commands ───────────────────────────────────────────────────

#[test]
fn test_ss_alias() {
    let dir = tempfile::tempdir().unwrap();

    smctl()
        .args(["workspace", "init", "--name", "alias-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // ss is alias for spec new
    smctl()
        .args(["ss", "alias-spec", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("created spec 'alias-spec'"));
}

// ── Completions ──────────────────────────────────────────────────────

#[test]
fn test_completions_bash() {
    smctl()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("smctl"));
}

// ── Logging (smctl-logging-v1) ───────────────────────────────────────

#[test]
fn test_log_file_emits_rfc5424_workspace_init() {
    let dir = tempfile::tempdir().unwrap();
    let log_path = dir.path().join("smctl.log");

    smctl()
        .args(["workspace", "init", "--name", "log-ws", "-w"])
        .arg(dir.path())
        .arg("--log-file")
        .arg(&log_path)
        .assert()
        .success();

    let log = std::fs::read_to_string(&log_path).expect("log file was not written");

    // PRI for local0(16) × 8 + Informational(6) = 134
    assert!(log.contains("<134>1 "), "missing PRI header: {log}");
    assert!(log.contains(" smctl "), "missing APP-NAME: {log}");
    assert!(log.contains(" SMCTL-0001 "), "missing MSGID: {log}");
    assert!(
        log.contains("[SMCTL@32473"),
        "missing STRUCTURED-DATA: {log}"
    );
    assert!(log.contains("name=\"log-ws\""), "missing name field: {log}");
    assert!(log.ends_with('\n'), "missing trailing newline: {log}");
}

#[test]
fn test_log_level_rejects_bad_value() {
    smctl()
        .args(["--log-level", "banana", "workspace", "status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown log level"));
}

// ── Quality ──────────────────────────────────────────────────────────

/// Probe whether cargo-audit is installed. Used to skip the audit
/// integration test gracefully on machines / CI environments that do not
/// have the tool yet.
fn cargo_audit_present() -> bool {
    std::process::Command::new("cargo")
        .args(["audit", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn test_quality_audit_help_is_sentence_case() {
    // Voice check: the help text is sentence case, imperative, no emoji.
    smctl()
        .args(["quality", "audit", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Audit dependencies for published RUSTSEC advisories",
        ));
}

#[test]
fn test_quality_audit_missing_tool_emits_three_part_error() {
    // When cargo-audit is absent, the CLI must surface a three-part
    // message (what happened / what it means / what to do next). Under
    // assert_cmd, stdout is not a TTY so the CLI renders the error as
    // JSON (per safety-quality-v1/design.md Decision 9); we assert the
    // three-part text is present in that JSON payload.
    if cargo_audit_present() {
        eprintln!("skipping: cargo-audit is installed; this path is only reachable without it");
        return;
    }

    let output = smctl()
        .args(["quality", "audit"])
        .output()
        .expect("failed to run smctl quality audit");
    assert!(!output.status.success(), "expected non-zero exit");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        combined.contains("cargo-audit is not installed"),
        "missing 'what happened' line. got: {combined}"
    );
    assert!(
        combined.contains("cargo install cargo-audit"),
        "missing remediation line. got: {combined}"
    );
}

#[test]
fn test_quality_audit_json_output_is_structurally_valid() {
    // Detection-and-skip: if cargo-audit is not installed, the JSON path
    // still returns valid JSON (an error object), and we verify that shape.
    // If cargo-audit IS installed, we verify the audit-report JSON shape.
    // Either way, stdout must parse as JSON.
    let output = smctl()
        .args(["quality", "audit", "--json"])
        .output()
        .expect("failed to run smctl quality audit --json");

    // Strip any leading non-JSON lines (tracing may emit to stderr only,
    // but guard anyway). stdout should be a single JSON document.
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let value: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {e}\nstdout was:\n{stdout}"));

    // The output is either an error object (tool missing) or a report
    // object. Both are JSON objects at the top level.
    assert!(
        value.is_object(),
        "expected top-level JSON object, got: {value}"
    );

    if cargo_audit_present() {
        // Report shape — structural keys only; do not assert on advisory
        // content because RUSTSEC drifts.
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("root"), "report missing 'root' key");
        assert!(
            obj.contains_key("advisories"),
            "report missing 'advisories' key"
        );
        assert!(
            obj.contains_key("advisory_count"),
            "report missing 'advisory_count' key"
        );
        assert!(obj.contains_key("fail"), "report missing 'fail' key");
        assert!(
            obj.contains_key("duration_ms"),
            "report missing 'duration_ms' key"
        );
        assert!(
            obj["advisories"].is_array(),
            "'advisories' must be an array"
        );
    } else {
        // Tool-missing error object shape.
        let obj = value.as_object().unwrap();
        assert_eq!(
            obj.get("error").and_then(|v| v.as_str()),
            Some("tool_missing"),
            "expected error=tool_missing when cargo-audit is absent"
        );
        assert!(obj.contains_key("remediation"), "missing 'remediation' key");
    }
}

/// Probe whether cargo-machete is installed. Used to skip the deps
/// integration test gracefully on machines / CI environments that do not
/// have the tool yet.
fn cargo_machete_present() -> bool {
    std::process::Command::new("cargo")
        .args(["machete", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn test_quality_deps_help_is_sentence_case() {
    // Voice check: the help text is sentence case, imperative, no emoji.
    smctl()
        .args(["quality", "deps", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Scan for unused dependencies in workspace manifests",
        ));
}

#[test]
fn test_quality_deps_missing_tool_emits_three_part_error() {
    // When cargo-machete is absent, the CLI must surface a three-part
    // message (what happened / what it means / what to do next). Under
    // assert_cmd, stdout is not a TTY so the CLI renders the error as
    // JSON (per safety-quality-v1/design.md Decision 9); we assert the
    // three-part text is present in that JSON payload.
    if cargo_machete_present() {
        eprintln!("skipping: cargo-machete is installed; this path is only reachable without it");
        return;
    }

    let output = smctl()
        .args(["quality", "deps"])
        .output()
        .expect("failed to run smctl quality deps");
    assert!(!output.status.success(), "expected non-zero exit");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        combined.contains("cargo-machete is not installed"),
        "missing 'what happened' line. got: {combined}"
    );
    assert!(
        combined.contains("cargo install cargo-machete"),
        "missing remediation line. got: {combined}"
    );
}

#[test]
fn test_quality_deps_json_output_is_structurally_valid() {
    // Detection-and-skip: if cargo-machete is not installed, the JSON path
    // still returns valid JSON (an error object), and we verify that shape.
    // If cargo-machete IS installed, we verify the deps-report JSON shape.
    // Either way, stdout must parse as JSON.
    let output = smctl()
        .args(["quality", "deps", "--json"])
        .output()
        .expect("failed to run smctl quality deps --json");

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let value: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {e}\nstdout was:\n{stdout}"));

    assert!(
        value.is_object(),
        "expected top-level JSON object, got: {value}"
    );

    if cargo_machete_present() {
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("root"), "report missing 'root' key");
        assert!(obj.contains_key("unused"), "report missing 'unused' key");
        assert!(
            obj.contains_key("unused_count"),
            "report missing 'unused_count' key"
        );
        assert!(obj.contains_key("fail"), "report missing 'fail' key");
        assert!(
            obj.contains_key("duration_ms"),
            "report missing 'duration_ms' key"
        );
        assert!(obj["unused"].is_array(), "'unused' must be an array");
    } else {
        let obj = value.as_object().unwrap();
        assert_eq!(
            obj.get("error").and_then(|v| v.as_str()),
            Some("tool_missing"),
            "expected error=tool_missing when cargo-machete is absent"
        );
        assert!(obj.contains_key("remediation"), "missing 'remediation' key");
    }
}

/// Probe whether cargo-geiger is installed. Used to skip the unsafe
/// integration test gracefully on machines / CI environments that do not
/// have the tool yet.
fn cargo_geiger_present() -> bool {
    std::process::Command::new("cargo")
        .args(["geiger", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn test_quality_unsafe_help_is_sentence_case() {
    // Voice check: the help text is sentence case, imperative, no emoji.
    smctl()
        .args(["quality", "unsafe", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Report unsafe code usage across the workspace",
        ));
}

#[test]
fn test_quality_unsafe_missing_tool_emits_three_part_error() {
    // When cargo-geiger is absent, the CLI must surface a three-part
    // message (what happened / what it means / what to do next). Under
    // assert_cmd, stdout is not a TTY so the CLI renders the error as
    // JSON (per safety-quality-v1/design.md Decision 9); we assert the
    // three-part text is present in that JSON payload.
    if cargo_geiger_present() {
        eprintln!("skipping: cargo-geiger is installed; this path is only reachable without it");
        return;
    }

    let output = smctl()
        .args(["quality", "unsafe"])
        .output()
        .expect("failed to run smctl quality unsafe");
    assert!(!output.status.success(), "expected non-zero exit");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        combined.contains("cargo-geiger is not installed"),
        "missing 'what happened' line. got: {combined}"
    );
    assert!(
        combined.contains("cargo install cargo-geiger"),
        "missing remediation line. got: {combined}"
    );
}

#[test]
fn test_quality_unsafe_json_output_is_structurally_valid() {
    // Detection-and-skip: if cargo-geiger is not installed, the JSON path
    // still returns valid JSON (an error object), and we verify that shape.
    // If cargo-geiger IS installed, we verify the unsafe-report JSON shape.
    // Either way, stdout must parse as JSON.
    let output = smctl()
        .args(["quality", "unsafe", "--json"])
        .output()
        .expect("failed to run smctl quality unsafe --json");

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let value: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {e}\nstdout was:\n{stdout}"));

    assert!(
        value.is_object(),
        "expected top-level JSON object, got: {value}"
    );

    if cargo_geiger_present() {
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("root"), "report missing 'root' key");
        assert!(obj.contains_key("crates"), "report missing 'crates' key");
        assert!(
            obj.contains_key("crate_count"),
            "report missing 'crate_count' key"
        );
        assert!(
            obj.contains_key("total_unsafe"),
            "report missing 'total_unsafe' key"
        );
        assert!(obj.contains_key("fail"), "report missing 'fail' key");
        assert!(
            obj.contains_key("duration_ms"),
            "report missing 'duration_ms' key"
        );
        assert!(obj["crates"].is_array(), "'crates' must be an array");
    } else {
        let obj = value.as_object().unwrap();
        assert_eq!(
            obj.get("error").and_then(|v| v.as_str()),
            Some("tool_missing"),
            "expected error=tool_missing when cargo-geiger is absent"
        );
        assert!(obj.contains_key("remediation"), "missing 'remediation' key");
    }
}
