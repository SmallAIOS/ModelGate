//! CLI integration tests for smctl using assert_cmd.

use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

#[allow(deprecated)]
fn smctl() -> Command {
    Command::cargo_bin("smctl").unwrap()
}

/// Initialize a workspace at the given path with a git repo.
fn init_workspace_with_git(root: &Path) {
    let cmds: &[&[&str]] = &[&["git", "init"], &["git", "checkout", "-b", "main"]];
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
        .stdout(predicate::str::contains("proposal=ok"))
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

    // Second create should fail
    smctl()
        .args(["spec", "new", "dup-spec", "-w"])
        .arg(dir.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
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
