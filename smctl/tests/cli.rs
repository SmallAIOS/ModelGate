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
