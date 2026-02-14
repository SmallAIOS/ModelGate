//! Integration tests for smctl-workspace using real git repositories.

use std::path::Path;

use smctl_workspace::{WorkspaceManifest, add_repo, init_workspace, remove_repo, repo_status};

/// Create a bare git repo and a clone of it within the workspace root.
fn setup_git_repo(root: &Path, name: &str) -> String {
    let bare_dir = root.join(format!("{name}.git"));
    std::fs::create_dir_all(&bare_dir).unwrap();

    // Init bare repo
    let output = std::process::Command::new("git")
        .args(["init", "--bare"])
        .current_dir(&bare_dir)
        .output()
        .unwrap();
    assert!(output.status.success(), "git init --bare failed");

    // Clone into workspace
    let output = std::process::Command::new("git")
        .args([
            "clone",
            bare_dir.to_str().unwrap(),
            root.join(name).to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git clone failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Create initial commit so HEAD exists
    let repo_path = root.join(name);
    std::fs::write(repo_path.join("README.md"), "# Test\n").unwrap();

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
        &["git", "push", "origin", "main"],
    ];
    for cmd in cmds {
        let output = std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .current_dir(&repo_path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{} failed: {}",
            cmd.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    bare_dir.to_str().unwrap().to_string()
}

#[test]
fn test_workspace_init_creates_manifest() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = init_workspace(dir.path(), "test-ws").unwrap();

    assert_eq!(manifest.workspace.name, "test-ws");
    assert!(dir.path().join(".smctl/workspace.toml").exists());

    // Load it back
    let loaded = WorkspaceManifest::load_from_root(dir.path()).unwrap();
    assert_eq!(loaded.workspace.name, "test-ws");
    assert!(loaded.repos.is_empty());
}

#[test]
fn test_workspace_add_remove_repo() {
    let dir = tempfile::tempdir().unwrap();
    let mut manifest = init_workspace(dir.path(), "test-ws").unwrap();

    add_repo(
        &mut manifest,
        "repo-a",
        "https://example.com/a",
        Some("repo-a"),
    )
    .unwrap();
    assert_eq!(manifest.repos.len(), 1);
    assert!(manifest.find_repo("repo-a").is_some());

    add_repo(&mut manifest, "repo-b", "https://example.com/b", None).unwrap();
    assert_eq!(manifest.repos.len(), 2);

    // Duplicate should fail
    assert!(add_repo(&mut manifest, "repo-a", "https://example.com/a2", None).is_err());

    // Save and reload
    manifest.save_to_root(dir.path()).unwrap();
    let loaded = WorkspaceManifest::load_from_root(dir.path()).unwrap();
    assert_eq!(loaded.repos.len(), 2);

    // Remove
    remove_repo(&mut manifest, "repo-b").unwrap();
    assert_eq!(manifest.repos.len(), 1);

    // Remove non-existent should fail
    assert!(remove_repo(&mut manifest, "nope").is_err());
}

#[test]
fn test_workspace_repo_status_with_real_git() {
    let dir = tempfile::tempdir().unwrap();
    let bare_url = setup_git_repo(dir.path(), "my-repo");

    let mut manifest = init_workspace(dir.path(), "status-test").unwrap();
    add_repo(&mut manifest, "my-repo", &bare_url, Some("my-repo")).unwrap();

    let status = repo_status(dir.path(), manifest.find_repo("my-repo").unwrap()).unwrap();
    assert_eq!(status.name, "my-repo");
    assert_eq!(status.branch, "main");
    assert!(status.clean);
    assert_eq!(status.modified_files, 0);

    // Dirty the repo
    std::fs::write(dir.path().join("my-repo/dirty.txt"), "dirty").unwrap();
    let status = repo_status(dir.path(), manifest.find_repo("my-repo").unwrap()).unwrap();
    assert!(!status.clean);
    assert!(status.modified_files > 0);
}

#[test]
fn test_workspace_manifest_roundtrip_with_all_configs() {
    let dir = tempfile::tempdir().unwrap();
    let mut manifest = init_workspace(dir.path(), "full-ws").unwrap();

    add_repo(
        &mut manifest,
        "alpha",
        "https://example.com/alpha",
        Some("alpha"),
    )
    .unwrap();
    add_repo(
        &mut manifest,
        "beta",
        "https://example.com/beta",
        Some("beta"),
    )
    .unwrap();

    // Set custom flow config
    manifest.flow.feature_prefix = "feat/".to_string();
    manifest.flow.develop_branch = "dev".to_string();

    manifest.save_to_root(dir.path()).unwrap();
    let loaded = WorkspaceManifest::load_from_root(dir.path()).unwrap();

    assert_eq!(loaded.flow.feature_prefix, "feat/");
    assert_eq!(loaded.flow.develop_branch, "dev");
    assert_eq!(loaded.repos.len(), 2);
    assert_eq!(loaded.repo_names(), vec!["alpha", "beta"]);
}
