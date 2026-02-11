use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use smctl_workspace::WorkspaceManifest;

/// A set of linked worktrees across repos.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeSet {
    pub name: String,
    pub worktrees: Vec<WorktreeInfo>,
}

/// Info about a single worktree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub repo_name: String,
    pub branch: String,
    pub path: PathBuf,
    pub exists: bool,
}

/// List all worktree sets in the workspace.
pub fn list_worktrees(root: &Path, manifest: &WorkspaceManifest) -> Result<Vec<WorktreeSet>> {
    let base = root.join(&manifest.worktree.base_dir);
    let mut sets = Vec::new();

    if !base.exists() {
        return Ok(sets);
    }

    let entries = std::fs::read_dir(&base).context("failed to read worktree base dir")?;
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let mut worktrees = Vec::new();

        for repo in &manifest.repos {
            let wt_path = base.join(&name).join(repo.local_path());
            let exists = wt_path.exists();
            let branch = if exists {
                read_worktree_branch(&wt_path).unwrap_or_default()
            } else {
                String::new()
            };
            worktrees.push(WorktreeInfo {
                repo_name: repo.name.clone(),
                branch,
                path: wt_path,
                exists,
            });
        }

        if worktrees.iter().any(|w| w.exists) {
            sets.push(WorktreeSet { name, worktrees });
        }
    }

    Ok(sets)
}

/// Add linked worktrees for a feature across specified repos.
pub fn add_worktree(
    root: &Path,
    manifest: &WorkspaceManifest,
    name: &str,
    repos: Option<&[String]>,
    branch: &str,
) -> Result<Vec<WorktreeInfo>> {
    let base = root.join(&manifest.worktree.base_dir).join(name);
    std::fs::create_dir_all(&base).context("failed to create worktree directory")?;

    let target_repos: Vec<_> = match repos {
        Some(names) => manifest
            .repos
            .iter()
            .filter(|r| names.iter().any(|n| n == &r.name))
            .collect(),
        None => manifest.repos.iter().collect(),
    };

    let mut infos = Vec::new();
    for repo in &target_repos {
        let repo_path = root.join(repo.local_path());
        let wt_path = base.join(repo.local_path());

        let result = std::process::Command::new("git")
            .args(["worktree", "add", wt_path.to_str().unwrap(), "-b", branch])
            .current_dir(&repo_path)
            .output()
            .context("failed to run git worktree add")?;

        if !result.status.success() {
            // Try without -b if branch already exists
            let result = std::process::Command::new("git")
                .args(["worktree", "add", wt_path.to_str().unwrap(), branch])
                .current_dir(&repo_path)
                .output()
                .context("failed to run git worktree add")?;

            if !result.status.success() {
                let stderr = String::from_utf8_lossy(&result.stderr);
                anyhow::bail!(
                    "failed to add worktree for {} at {}: {}",
                    repo.name,
                    wt_path.display(),
                    stderr.trim()
                );
            }
        }

        infos.push(WorktreeInfo {
            repo_name: repo.name.clone(),
            branch: branch.to_string(),
            path: wt_path,
            exists: true,
        });
    }

    tracing::info!("added worktree set '{name}' for {} repos", infos.len());
    Ok(infos)
}

/// Remove a worktree set.
pub fn remove_worktree(
    root: &Path,
    manifest: &WorkspaceManifest,
    name: &str,
    force: bool,
) -> Result<()> {
    let base = root.join(&manifest.worktree.base_dir).join(name);
    if !base.exists() {
        anyhow::bail!("worktree set '{name}' does not exist");
    }

    for repo in &manifest.repos {
        let wt_path = base.join(repo.local_path());
        if !wt_path.exists() {
            continue;
        }

        let repo_path = root.join(repo.local_path());
        let mut args = vec!["worktree", "remove"];
        if force {
            args.push("--force");
        }
        args.push(wt_path.to_str().unwrap());

        let result = std::process::Command::new("git")
            .args(&args)
            .current_dir(&repo_path)
            .output()
            .context("failed to run git worktree remove")?;

        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            tracing::warn!(
                "failed to remove worktree for {}: {}",
                repo.name,
                stderr.trim()
            );
        }
    }

    // Clean up the directory
    if base.exists() {
        std::fs::remove_dir_all(&base).ok();
    }

    tracing::info!("removed worktree set '{name}'");
    Ok(())
}

/// Get the path to a worktree set (for shell integration / `cd`).
pub fn worktree_path(root: &Path, manifest: &WorkspaceManifest, name: &str) -> Result<PathBuf> {
    let base = root.join(&manifest.worktree.base_dir).join(name);
    if !base.exists() {
        anyhow::bail!("worktree set '{name}' does not exist");
    }
    Ok(base)
}

/// Read the current branch of a worktree by examining its .git file.
fn read_worktree_branch(path: &Path) -> Result<String> {
    let repo = git2::Repository::open(path)?;
    let head = repo.head()?;
    Ok(head.shorthand().unwrap_or("detached").to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_empty_worktrees() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = smctl_workspace::WorkspaceManifest::parse(
            r#"
            [workspace]
            name = "test"
            "#,
        )
        .unwrap();

        let result = list_worktrees(dir.path(), &manifest).unwrap();
        assert!(result.is_empty());
    }
}
