use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use smctl_workspace::{FlowConfig, WorkspaceManifest};

/// Result of a flow operation across repos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowResult {
    pub operation: String,
    pub branch_name: String,
    pub repos: Vec<FlowRepoResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowRepoResult {
    pub repo_name: String,
    pub success: bool,
    pub message: String,
}

/// Active branch info for a repo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub repo_name: String,
    pub branch: String,
    pub branch_type: BranchType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BranchType {
    Main,
    Develop,
    Feature,
    Release,
    Hotfix,
    Other,
}

/// Classify a branch name based on flow config.
pub fn classify_branch(name: &str, flow: &FlowConfig) -> BranchType {
    if name == flow.main_branch {
        BranchType::Main
    } else if name == flow.develop_branch {
        BranchType::Develop
    } else if name.starts_with(&flow.feature_prefix) {
        BranchType::Feature
    } else if name.starts_with(&flow.release_prefix) {
        BranchType::Release
    } else if name.starts_with(&flow.hotfix_prefix) {
        BranchType::Hotfix
    } else {
        BranchType::Other
    }
}

/// Initialize git flow: ensure develop branch exists in all repos.
pub fn init(root: &Path, manifest: &WorkspaceManifest) -> Result<FlowResult> {
    let mut results = Vec::new();

    for repo in &manifest.repos {
        let repo_path = root.join(repo.local_path());
        let git_repo = git2::Repository::open(&repo_path)
            .with_context(|| format!("failed to open repo {}", repo.name))?;

        let result = ensure_branch_exists(&git_repo, &manifest.flow.develop_branch);
        results.push(FlowRepoResult {
            repo_name: repo.name.clone(),
            success: result.is_ok(),
            message: match &result {
                Ok(_) => format!("'{}' branch ready", manifest.flow.develop_branch),
                Err(e) => format!("{e}"),
            },
        });
    }

    Ok(FlowResult {
        operation: "flow init".to_string(),
        branch_name: manifest.flow.develop_branch.clone(),
        repos: results,
    })
}

/// Start a feature branch across specified repos.
pub fn feature_start(
    root: &Path,
    manifest: &WorkspaceManifest,
    name: &str,
    repos: Option<&[String]>,
) -> Result<FlowResult> {
    let branch = format!("{}{}", manifest.flow.feature_prefix, name);
    let base = &manifest.flow.develop_branch;
    start_branch(root, manifest, &branch, base, repos, "feature start")
}

/// Finish a feature branch: merge into develop.
pub fn feature_finish(root: &Path, manifest: &WorkspaceManifest, name: &str) -> Result<FlowResult> {
    let branch = format!("{}{}", manifest.flow.feature_prefix, name);
    let target = &manifest.flow.develop_branch;
    finish_branch(root, manifest, &branch, target, "feature finish")
}

/// List active feature branches across repos.
pub fn feature_list(root: &Path, manifest: &WorkspaceManifest) -> Result<Vec<BranchInfo>> {
    list_branches_by_type(root, manifest, BranchType::Feature)
}

/// Start a release branch.
pub fn release_start(
    root: &Path,
    manifest: &WorkspaceManifest,
    version: &str,
    repos: Option<&[String]>,
) -> Result<FlowResult> {
    let branch = format!("{}{}", manifest.flow.release_prefix, version);
    let base = &manifest.flow.develop_branch;
    start_branch(root, manifest, &branch, base, repos, "release start")
}

/// Finish a release: merge to main + develop, tag.
pub fn release_finish(
    root: &Path,
    manifest: &WorkspaceManifest,
    version: &str,
) -> Result<FlowResult> {
    let branch = format!("{}{}", manifest.flow.release_prefix, version);
    let main = &manifest.flow.main_branch;
    // Phase 1: merge to main
    let main_result = finish_branch(root, manifest, &branch, main, "release finish → main")?;
    // Phase 2: merge to develop
    let dev_result = finish_branch(
        root,
        manifest,
        &branch,
        &manifest.flow.develop_branch,
        "release finish → develop",
    )?;

    // Combine results
    let mut repos = main_result.repos;
    repos.extend(dev_result.repos);
    Ok(FlowResult {
        operation: "release finish".to_string(),
        branch_name: branch,
        repos,
    })
}

/// Start a hotfix branch from main.
pub fn hotfix_start(
    root: &Path,
    manifest: &WorkspaceManifest,
    name: &str,
    repos: Option<&[String]>,
) -> Result<FlowResult> {
    let branch = format!("{}{}", manifest.flow.hotfix_prefix, name);
    let base = &manifest.flow.main_branch;
    start_branch(root, manifest, &branch, base, repos, "hotfix start")
}

/// Finish a hotfix: merge to main + develop.
pub fn hotfix_finish(root: &Path, manifest: &WorkspaceManifest, name: &str) -> Result<FlowResult> {
    let branch = format!("{}{}", manifest.flow.hotfix_prefix, name);
    let main = &manifest.flow.main_branch;
    let main_result = finish_branch(root, manifest, &branch, main, "hotfix finish → main")?;
    let dev_result = finish_branch(
        root,
        manifest,
        &branch,
        &manifest.flow.develop_branch,
        "hotfix finish → develop",
    )?;

    let mut repos = main_result.repos;
    repos.extend(dev_result.repos);
    Ok(FlowResult {
        operation: "hotfix finish".to_string(),
        branch_name: branch,
        repos,
    })
}

/// List active hotfix branches.
pub fn hotfix_list(root: &Path, manifest: &WorkspaceManifest) -> Result<Vec<BranchInfo>> {
    list_branches_by_type(root, manifest, BranchType::Hotfix)
}

/// List active release branches.
pub fn release_list(root: &Path, manifest: &WorkspaceManifest) -> Result<Vec<BranchInfo>> {
    list_branches_by_type(root, manifest, BranchType::Release)
}

// --- Internal helpers ---

fn start_branch(
    root: &Path,
    manifest: &WorkspaceManifest,
    branch: &str,
    base: &str,
    repos: Option<&[String]>,
    operation: &str,
) -> Result<FlowResult> {
    let target_repos: Vec<_> = match repos {
        Some(names) => manifest
            .repos
            .iter()
            .filter(|r| names.iter().any(|n| n == &r.name))
            .collect(),
        None => manifest.repos.iter().collect(),
    };

    // Phase 1: validate all repos
    for repo in &target_repos {
        let repo_path = root.join(repo.local_path());
        let git_repo = git2::Repository::open(&repo_path)
            .with_context(|| format!("failed to open repo {}", repo.name))?;

        // Check base branch exists
        git_repo
            .find_branch(base, git2::BranchType::Local)
            .with_context(|| format!("base branch '{base}' not found in {}", repo.name))?;
    }

    // Phase 2: execute
    let mut results = Vec::new();
    for repo in &target_repos {
        let repo_path = root.join(repo.local_path());
        let result = std::process::Command::new("git")
            .args(["checkout", "-b", branch, base])
            .current_dir(&repo_path)
            .output()
            .context("failed to run git checkout")?;

        results.push(FlowRepoResult {
            repo_name: repo.name.clone(),
            success: result.status.success(),
            message: if result.status.success() {
                format!("created '{branch}' from '{base}'")
            } else {
                String::from_utf8_lossy(&result.stderr).trim().to_string()
            },
        });
    }

    Ok(FlowResult {
        operation: operation.to_string(),
        branch_name: branch.to_string(),
        repos: results,
    })
}

fn finish_branch(
    root: &Path,
    manifest: &WorkspaceManifest,
    branch: &str,
    target: &str,
    operation: &str,
) -> Result<FlowResult> {
    let mut results = Vec::new();

    for repo in &manifest.repos {
        let repo_path = root.join(repo.local_path());
        let git_repo = git2::Repository::open(&repo_path);
        let git_repo = match git_repo {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Check if branch exists in this repo
        if git_repo
            .find_branch(branch, git2::BranchType::Local)
            .is_err()
        {
            continue;
        }

        // Checkout target
        let checkout = std::process::Command::new("git")
            .args(["checkout", target])
            .current_dir(&repo_path)
            .output()?;

        if !checkout.status.success() {
            results.push(FlowRepoResult {
                repo_name: repo.name.clone(),
                success: false,
                message: format!(
                    "failed to checkout '{target}': {}",
                    String::from_utf8_lossy(&checkout.stderr).trim()
                ),
            });
            continue;
        }

        // Merge
        let merge = std::process::Command::new("git")
            .args(["merge", "--no-ff", branch])
            .current_dir(&repo_path)
            .output()?;

        let success = merge.status.success();
        let message = if success {
            // Delete the feature branch
            let _ = std::process::Command::new("git")
                .args(["branch", "-d", branch])
                .current_dir(&repo_path)
                .output();
            format!("merged '{branch}' into '{target}'")
        } else {
            String::from_utf8_lossy(&merge.stderr).trim().to_string()
        };

        results.push(FlowRepoResult {
            repo_name: repo.name.clone(),
            success,
            message,
        });
    }

    Ok(FlowResult {
        operation: operation.to_string(),
        branch_name: branch.to_string(),
        repos: results,
    })
}

fn ensure_branch_exists(repo: &git2::Repository, branch_name: &str) -> Result<()> {
    if repo
        .find_branch(branch_name, git2::BranchType::Local)
        .is_ok()
    {
        return Ok(());
    }

    let head = repo.head().context("repo has no HEAD")?;
    let commit = head.peel_to_commit().context("HEAD is not a commit")?;

    repo.branch(branch_name, &commit, false)
        .with_context(|| format!("failed to create branch '{branch_name}'"))?;

    Ok(())
}

fn list_branches_by_type(
    root: &Path,
    manifest: &WorkspaceManifest,
    branch_type: BranchType,
) -> Result<Vec<BranchInfo>> {
    let mut branches = Vec::new();

    for repo in &manifest.repos {
        let repo_path = root.join(repo.local_path());
        let git_repo = match git2::Repository::open(&repo_path) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let git_branches = git_repo.branches(Some(git2::BranchType::Local))?;
        for branch_result in git_branches {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                let bt = classify_branch(name, &manifest.flow);
                if bt == branch_type {
                    branches.push(BranchInfo {
                        repo_name: repo.name.clone(),
                        branch: name.to_string(),
                        branch_type: bt,
                    });
                }
            }
        }
    }

    Ok(branches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_branch() {
        let flow = FlowConfig::default();
        assert_eq!(classify_branch("main", &flow), BranchType::Main);
        assert_eq!(classify_branch("develop", &flow), BranchType::Develop);
        assert_eq!(classify_branch("feature/foo", &flow), BranchType::Feature);
        assert_eq!(classify_branch("release/1.0", &flow), BranchType::Release);
        assert_eq!(classify_branch("hotfix/fix", &flow), BranchType::Hotfix);
        assert_eq!(classify_branch("random", &flow), BranchType::Other);
    }
}
