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

// ── Verify commands ──────────────────────────────────────────────────

#[test]
fn test_verify_help_lists_all_subcommands() {
    smctl()
        .args(["verify", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("policy"))
        .stdout(predicate::str::contains("model"))
        .stdout(predicate::str::contains("proof"))
        .stdout(predicate::str::contains("protocol"))
        .stdout(predicate::str::contains("discover"));
}

#[test]
fn test_verify_discover_lists_four_verifiers() {
    let dir = tempfile::tempdir().unwrap();
    smctl()
        .args(["workspace", "init", "--name", "verify-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // Discover should list all four verifiers regardless of host
    // toolchain. policy is always Found (Cedar is a Rust dep); the
    // shell-out trio may be either Found or NotInstalled depending
    // on the runner — assert each row appears, not the per-row
    // status.
    smctl()
        .args(["verify", "discover", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("policy"))
        .stdout(predicate::str::contains("model"))
        .stdout(predicate::str::contains("proof"))
        .stdout(predicate::str::contains("protocol"));
}

#[test]
fn test_verify_discover_json_output() {
    let dir = tempfile::tempdir().unwrap();
    smctl()
        .args(["workspace", "init", "--name", "verify-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    let output = smctl()
        .args(["verify", "discover", "--json", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let body = String::from_utf8(output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body)
        .unwrap_or_else(|e| panic!("discover --json should produce valid JSON: {e}\n{body}"));
    let arr = parsed.as_array().expect("array");
    assert_eq!(arr.len(), 4, "expected 4 entries: {body}");
    let names: Vec<&str> = arr
        .iter()
        .filter_map(|e| e.get("verifier").and_then(|v| v.as_str()))
        .collect();
    assert!(names.contains(&"policy"));
    assert!(names.contains(&"model"));
    assert!(names.contains(&"proof"));
    assert!(names.contains(&"protocol"));
}

#[test]
fn test_verify_policy_no_sources_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    smctl()
        .args(["workspace", "init", "--name", "verify-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // No [verify.policy] section in the fresh manifest -> Cedar
    // verifier reports outcome=no_sources and exits 0. assert_cmd
    // gives the child a non-TTY stdout, so the dispatcher's
    // safety-quality-v1 Decision 9 fallback emits JSON; we assert
    // against the JSON envelope rather than the human table.
    smctl()
        .args(["verify", "policy", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"verifier\": \"policy\""))
        .stdout(predicate::str::contains("\"outcome\": \"no_sources\""));
}

#[test]
fn test_verify_policy_dry_run() {
    let dir = tempfile::tempdir().unwrap();
    smctl()
        .args(["workspace", "init", "--name", "verify-ws", "-w"])
        .arg(dir.path())
        .assert()
        .success();

    // --dry-run should describe what would happen and exit with the
    // DRY_RUN code (10), per safety-quality-v1's exit-code contract.
    smctl()
        .args(["--dry-run", "verify", "policy", "-w"])
        .arg(dir.path())
        .assert()
        .code(10)
        .stdout(predicate::str::contains("would run verifier 'policy'"));
}

#[test]
fn test_verify_policy_with_well_formed_cedar_passes() {
    let dir = tempfile::tempdir().unwrap();
    let repo_dir = dir.path().join("repo");
    std::fs::create_dir_all(repo_dir.join("policies")).unwrap();
    std::fs::write(
        repo_dir.join("policies").join("allow.cedar"),
        "permit(principal, action, resource);",
    )
    .unwrap();

    // Hand-write the manifest so the TOML structure is unambiguous —
    // [[repos]] entries and [verify.policy] both come from one
    // serialise pass rather than init_workspace + an appended tail.
    let smctl_dir = dir.path().join(".smctl");
    std::fs::create_dir_all(&smctl_dir).unwrap();
    let manifest_text = format!(
        r#"[workspace]
name = "verify-ws"
root = "."

[[repos]]
name = "repo"
url = "file://{repo}"
path = "{repo}"
default_branch = "main"

[verify.policy]
sources = ["policies/*.cedar"]
fail_on = "any"
"#,
        repo = repo_dir.display()
    );
    std::fs::write(smctl_dir.join("workspace.toml"), manifest_text).unwrap();

    // Cedar runs against the policy file we wrote. Outcome=passed,
    // 1 policy counted. assert_cmd captures stdout as non-TTY so
    // dispatcher falls through to JSON output per safety-quality-v1
    // Decision 9.
    smctl()
        .args(["verify", "policy", "-w"])
        .arg(dir.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"outcome\": \"passed\""))
        .stdout(predicate::str::contains("allow.cedar"))
        .stdout(predicate::str::contains("\"1 policy"));
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

/// Probe whether cargo-modules is installed. Used to skip the dsm
/// integration test gracefully on machines / CI environments that do not
/// have the tool yet.
fn cargo_modules_present() -> bool {
    std::process::Command::new("cargo")
        .args(["modules", "--version"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn test_quality_dsm_help_is_sentence_case() {
    // Voice check: the help text is sentence case, imperative, no emoji.
    smctl()
        .args(["quality", "dsm", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Build a module dependency structure matrix and detect cycles",
        ));
}

#[test]
fn test_quality_dsm_missing_tool_emits_three_part_error() {
    // When cargo-modules is absent, the CLI must surface a three-part
    // message (what happened / what it means / what to do next). Under
    // assert_cmd, stdout is not a TTY so the CLI renders the error as
    // JSON (per safety-quality-v1/design.md Decision 9); we assert the
    // three-part text is present in that JSON payload.
    if cargo_modules_present() {
        eprintln!("skipping: cargo-modules is installed; this path is only reachable without it");
        return;
    }

    let output = smctl()
        .args(["quality", "dsm"])
        .output()
        .expect("failed to run smctl quality dsm");
    assert!(!output.status.success(), "expected non-zero exit");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        combined.contains("cargo-modules is not installed"),
        "missing 'what happened' line. got: {combined}"
    );
    assert!(
        combined.contains("cargo install cargo-modules"),
        "missing remediation line. got: {combined}"
    );
}

#[test]
fn test_quality_dsm_json_output_is_structurally_valid() {
    // Detection-and-skip: if cargo-modules is not installed, the JSON path
    // still returns valid JSON (an error object), and we verify that shape.
    // If cargo-modules IS installed, we verify the dsm-report JSON shape.
    // Either way, stdout must parse as JSON.
    let output = smctl()
        .args(["quality", "dsm", "--json"])
        .output()
        .expect("failed to run smctl quality dsm --json");

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let value: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {e}\nstdout was:\n{stdout}"));

    assert!(
        value.is_object(),
        "expected top-level JSON object, got: {value}"
    );

    if cargo_modules_present() {
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("root"), "report missing 'root' key");
        assert!(obj.contains_key("cycles"), "report missing 'cycles' key");
        assert!(
            obj.contains_key("cycle_count"),
            "report missing 'cycle_count' key"
        );
        assert!(
            obj.contains_key("crates_scanned"),
            "report missing 'crates_scanned' key"
        );
        assert!(obj.contains_key("fail"), "report missing 'fail' key");
        assert!(
            obj.contains_key("duration_ms"),
            "report missing 'duration_ms' key"
        );
        assert!(obj["cycles"].is_array(), "'cycles' must be an array");
        assert!(
            obj["crates_scanned"].is_array(),
            "'crates_scanned' must be an array"
        );
    } else {
        let obj = value.as_object().unwrap();
        assert_eq!(
            obj.get("error").and_then(|v| v.as_str()),
            Some("tool_missing"),
            "expected error=tool_missing when cargo-modules is absent"
        );
        assert!(obj.contains_key("remediation"), "missing 'remediation' key");
    }
}

/// Probe whether rust-code-analysis-cli is installed. Used to skip the
/// complexity integration test gracefully on machines / CI environments
/// that do not have the tool yet.
fn rust_code_analysis_present() -> bool {
    std::process::Command::new("rust-code-analysis-cli")
        .arg("--help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn test_quality_complexity_help_voice() {
    // Voice check: the help text is sentence case, imperative, no emoji.
    smctl()
        .args(["quality", "complexity", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Measure cyclomatic and cognitive complexity per function",
        ));
}

#[test]
fn test_quality_complexity_missing_tool_three_part_error() {
    // When rust-code-analysis-cli is absent, the CLI must surface a
    // three-part message (what happened / what it means / what to do
    // next). Under assert_cmd, stdout is not a TTY so the CLI renders
    // the error as JSON (per safety-quality-v1/design.md Decision 9);
    // we assert the three-part text is present in that JSON payload.
    if rust_code_analysis_present() {
        eprintln!(
            "skipping: rust-code-analysis-cli is installed; this path is only reachable without it"
        );
        return;
    }

    let output = smctl()
        .args(["quality", "complexity"])
        .output()
        .expect("failed to run smctl quality complexity");
    assert!(!output.status.success(), "expected non-zero exit");

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(
        combined.contains("rust-code-analysis-cli is not installed"),
        "missing 'what happened' line. got: {combined}"
    );
    assert!(
        combined.contains("cargo install rust-code-analysis-cli"),
        "missing remediation line. got: {combined}"
    );
}

#[test]
fn test_quality_complexity_json_shape() {
    // Detection-and-skip: if rust-code-analysis-cli is not installed,
    // the JSON path still returns valid JSON (an error object), and we
    // verify that shape. If the tool IS installed, we verify the
    // complexity-report JSON shape. Either way, stdout must parse as
    // JSON.
    let output = smctl()
        .args(["quality", "complexity", "--json"])
        .output()
        .expect("failed to run smctl quality complexity --json");

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");

    let value: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {e}\nstdout was:\n{stdout}"));

    assert!(
        value.is_object(),
        "expected top-level JSON object, got: {value}"
    );

    if rust_code_analysis_present() {
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("root"), "report missing 'root' key");
        assert!(
            obj.contains_key("violations"),
            "report missing 'violations' key"
        );
        assert!(
            obj.contains_key("violation_count"),
            "report missing 'violation_count' key"
        );
        assert!(
            obj.contains_key("function_count"),
            "report missing 'function_count' key"
        );
        assert!(
            obj.contains_key("threshold_cyclomatic"),
            "report missing 'threshold_cyclomatic' key"
        );
        assert!(
            obj.contains_key("threshold_cognitive"),
            "report missing 'threshold_cognitive' key"
        );
        assert!(obj.contains_key("fail"), "report missing 'fail' key");
        assert!(
            obj.contains_key("duration_ms"),
            "report missing 'duration_ms' key"
        );
        assert!(
            obj["violations"].is_array(),
            "'violations' must be an array"
        );
    } else {
        let obj = value.as_object().unwrap();
        assert_eq!(
            obj.get("error").and_then(|v| v.as_str()),
            Some("tool_missing"),
            "expected error=tool_missing when rust-code-analysis-cli is absent"
        );
        assert!(obj.contains_key("remediation"), "missing 'remediation' key");
    }
}
