//! Integration tests for smctl-flow using real git repositories.

use std::path::Path;

use smctl_flow::{BranchType, classify_branch, feature_finish, feature_list, feature_start, init};
use smctl_workspace::WorkspaceManifest;

/// Set up a workspace root with one real git repo that has a main branch.
fn setup_workspace(root: &Path, repo_name: &str) -> WorkspaceManifest {
    let repo_path = root.join(repo_name);
    std::fs::create_dir_all(&repo_path).unwrap();

    let cmds: &[&[&str]] = &[&["git", "init"], &["git", "checkout", "-b", "main"]];
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

    // Create initial commit
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

    // Build a manifest referencing this repo
    let toml = format!(
        r#"
[workspace]
name = "test"

[[repos]]
name = "{repo_name}"
url = "file:///tmp/fake"
path = "{repo_name}"
"#,
    );
    WorkspaceManifest::parse(&toml).unwrap()
}

/// Set up a workspace with two repos.
fn setup_multi_repo_workspace(root: &Path) -> WorkspaceManifest {
    for name in &["alpha", "beta"] {
        let repo_path = root.join(name);
        std::fs::create_dir_all(&repo_path).unwrap();

        let cmds: &[&[&str]] = &[&["git", "init"], &["git", "checkout", "-b", "main"]];
        for cmd in cmds {
            std::process::Command::new(cmd[0])
                .args(&cmd[1..])
                .current_dir(&repo_path)
                .output()
                .unwrap();
        }

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
        ];
        for cmd in cmds {
            std::process::Command::new(cmd[0])
                .args(&cmd[1..])
                .current_dir(&repo_path)
                .output()
                .unwrap();
        }
    }

    WorkspaceManifest::parse(
        r#"
[workspace]
name = "multi"

[[repos]]
name = "alpha"
url = "file:///tmp/fake"
path = "alpha"

[[repos]]
name = "beta"
url = "file:///tmp/fake"
path = "beta"
"#,
    )
    .unwrap()
}

#[test]
fn test_flow_init_creates_develop() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = setup_workspace(dir.path(), "repo1");

    let result = init(dir.path(), &manifest).unwrap();
    assert_eq!(result.operation, "flow init");
    assert!(result.repos[0].success);

    // Verify develop branch exists
    let git_repo = git2::Repository::open(dir.path().join("repo1")).unwrap();
    assert!(
        git_repo
            .find_branch("develop", git2::BranchType::Local)
            .is_ok()
    );
}

#[test]
fn test_flow_init_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = setup_workspace(dir.path(), "repo1");

    init(dir.path(), &manifest).unwrap();
    // Second call should succeed (branch already exists)
    let result = init(dir.path(), &manifest).unwrap();
    assert!(result.repos[0].success);
}

#[test]
fn test_feature_start_finish_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = setup_workspace(dir.path(), "repo1");

    // Init flow first (creates develop)
    init(dir.path(), &manifest).unwrap();

    // Start a feature
    let start = feature_start(dir.path(), &manifest, "my-feature", None).unwrap();
    assert_eq!(start.branch_name, "feature/my-feature");
    assert!(start.repos[0].success);

    // Verify we're on the feature branch
    let git_repo = git2::Repository::open(dir.path().join("repo1")).unwrap();
    let head = git_repo.head().unwrap();
    assert_eq!(head.shorthand().unwrap(), "feature/my-feature");

    // Make a commit on the feature branch
    std::fs::write(dir.path().join("repo1/feature.txt"), "feature work").unwrap();
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
            "feature work",
        ],
    ];
    for cmd in cmds {
        std::process::Command::new(cmd[0])
            .args(&cmd[1..])
            .current_dir(dir.path().join("repo1"))
            .output()
            .unwrap();
    }

    // Finish the feature (merges into develop)
    let finish = feature_finish(dir.path(), &manifest, "my-feature").unwrap();
    assert!(finish.repos[0].success);
    assert!(finish.repos[0].message.contains("merged"));

    // Verify we're back on develop
    let git_repo = git2::Repository::open(dir.path().join("repo1")).unwrap();
    let head = git_repo.head().unwrap();
    assert_eq!(head.shorthand().unwrap(), "develop");

    // Feature branch should be deleted
    assert!(
        git_repo
            .find_branch("feature/my-feature", git2::BranchType::Local)
            .is_err()
    );
}

#[test]
fn test_feature_list() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = setup_workspace(dir.path(), "repo1");

    init(dir.path(), &manifest).unwrap();
    feature_start(dir.path(), &manifest, "feat-a", None).unwrap();

    // Checkout develop so we can create another branch
    std::process::Command::new("git")
        .args(["checkout", "develop"])
        .current_dir(dir.path().join("repo1"))
        .output()
        .unwrap();
    feature_start(dir.path(), &manifest, "feat-b", None).unwrap();

    let branches = feature_list(dir.path(), &manifest).unwrap();
    let names: Vec<_> = branches.iter().map(|b| b.branch.as_str()).collect();
    assert!(names.contains(&"feature/feat-a"));
    assert!(names.contains(&"feature/feat-b"));
}

#[test]
fn test_feature_start_across_multiple_repos() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = setup_multi_repo_workspace(dir.path());

    init(dir.path(), &manifest).unwrap();
    let result = feature_start(dir.path(), &manifest, "cross-repo", None).unwrap();

    assert_eq!(result.repos.len(), 2);
    assert!(result.repos.iter().all(|r| r.success));

    // Both repos should have the feature branch
    for name in &["alpha", "beta"] {
        let git_repo = git2::Repository::open(dir.path().join(name)).unwrap();
        assert!(
            git_repo
                .find_branch("feature/cross-repo", git2::BranchType::Local)
                .is_ok(),
            "feature branch missing in {name}"
        );
    }
}

#[test]
fn test_feature_start_with_repo_filter() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = setup_multi_repo_workspace(dir.path());

    init(dir.path(), &manifest).unwrap();

    // Only start in alpha
    let repos = vec!["alpha".to_string()];
    let result = feature_start(dir.path(), &manifest, "filtered", Some(&repos)).unwrap();

    assert_eq!(result.repos.len(), 1);
    assert_eq!(result.repos[0].repo_name, "alpha");

    // Alpha should have the branch, beta should not
    let alpha = git2::Repository::open(dir.path().join("alpha")).unwrap();
    assert!(
        alpha
            .find_branch("feature/filtered", git2::BranchType::Local)
            .is_ok()
    );

    let beta = git2::Repository::open(dir.path().join("beta")).unwrap();
    assert!(
        beta.find_branch("feature/filtered", git2::BranchType::Local)
            .is_err()
    );
}

#[test]
fn test_classify_all_branch_types() {
    let flow = smctl_workspace::FlowConfig::default();
    assert_eq!(classify_branch("main", &flow), BranchType::Main);
    assert_eq!(classify_branch("develop", &flow), BranchType::Develop);
    assert_eq!(classify_branch("feature/x", &flow), BranchType::Feature);
    assert_eq!(classify_branch("release/1.0", &flow), BranchType::Release);
    assert_eq!(classify_branch("hotfix/fix", &flow), BranchType::Hotfix);
    assert_eq!(classify_branch("other", &flow), BranchType::Other);
}
