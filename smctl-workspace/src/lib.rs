use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// A workspace manifest (.smctl/workspace.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    pub workspace: WorkspaceConfig,
    #[serde(default)]
    pub repos: Vec<RepoConfig>,
    #[serde(default)]
    pub flow: FlowConfig,
    #[serde(default)]
    pub worktree: WorktreeConfig,
    #[serde(default)]
    pub spec: SpecConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub name: String,
    #[serde(default = "default_root")]
    pub root: String,
}

fn default_root() -> String {
    ".".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoConfig {
    pub name: String,
    pub url: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default = "default_branch")]
    pub default_branch: String,
    /// If true, this repo is where smctl lives.
    #[serde(default)]
    pub smctl_home: bool,
    /// Build command for this repo.
    #[serde(default)]
    pub build_cmd: Option<String>,
    /// Test command for this repo.
    #[serde(default)]
    pub test_cmd: Option<String>,
    /// Clean command for this repo.
    #[serde(default)]
    pub clean_cmd: Option<String>,
    /// Repos this repo depends on (for build ordering).
    #[serde(default)]
    pub depends_on: Vec<String>,
}

fn default_branch() -> String {
    "main".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowConfig {
    #[serde(default = "default_main_branch")]
    pub main_branch: String,
    #[serde(default = "default_develop_branch")]
    pub develop_branch: String,
    #[serde(default = "default_feature_prefix")]
    pub feature_prefix: String,
    #[serde(default = "default_release_prefix")]
    pub release_prefix: String,
    #[serde(default = "default_hotfix_prefix")]
    pub hotfix_prefix: String,
}

fn default_main_branch() -> String {
    "main".to_string()
}
fn default_develop_branch() -> String {
    "develop".to_string()
}
fn default_feature_prefix() -> String {
    "feature/".to_string()
}
fn default_release_prefix() -> String {
    "release/".to_string()
}
fn default_hotfix_prefix() -> String {
    "hotfix/".to_string()
}

impl Default for FlowConfig {
    fn default() -> Self {
        Self {
            main_branch: default_main_branch(),
            develop_branch: default_develop_branch(),
            feature_prefix: default_feature_prefix(),
            release_prefix: default_release_prefix(),
            hotfix_prefix: default_hotfix_prefix(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeConfig {
    #[serde(default = "default_worktree_base")]
    pub base_dir: String,
}

fn default_worktree_base() -> String {
    ".worktrees".to_string()
}

impl Default for WorktreeConfig {
    fn default() -> Self {
        Self {
            base_dir: default_worktree_base(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecConfig {
    #[serde(default = "default_openspec_dir")]
    pub openspec_dir: String,
}

fn default_openspec_dir() -> String {
    "openspec".to_string()
}

impl Default for SpecConfig {
    fn default() -> Self {
        Self {
            openspec_dir: default_openspec_dir(),
        }
    }
}

impl RepoConfig {
    /// Effective local path for this repo within the workspace.
    pub fn local_path(&self) -> &str {
        self.path.as_deref().unwrap_or(&self.name)
    }
}

impl WorkspaceManifest {
    /// Parse a workspace manifest from a TOML string.
    pub fn parse(s: &str) -> Result<Self> {
        toml::from_str(s).context("failed to parse workspace.toml")
    }

    /// Load a workspace manifest from a file path.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).context("failed to read workspace.toml")?;
        Self::parse(&content)
    }

    /// Load from the workspace root (looks for .smctl/workspace.toml).
    pub fn load_from_root(root: &Path) -> Result<Self> {
        let path = root.join(".smctl").join("workspace.toml");
        Self::load(&path)
    }

    /// Save workspace manifest to disk.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self).context("failed to serialize workspace.toml")?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Save to the workspace root (.smctl/workspace.toml).
    pub fn save_to_root(&self, root: &Path) -> Result<()> {
        let path = root.join(".smctl").join("workspace.toml");
        self.save(&path)
    }

    /// Find a repo by name.
    pub fn find_repo(&self, name: &str) -> Option<&RepoConfig> {
        self.repos.iter().find(|r| r.name == name)
    }

    /// Get all repo names.
    pub fn repo_names(&self) -> Vec<&str> {
        self.repos.iter().map(|r| r.name.as_str()).collect()
    }
}

/// Repo status information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    pub name: String,
    pub branch: String,
    pub clean: bool,
    pub ahead: usize,
    pub behind: usize,
    pub modified_files: usize,
}

/// Initialize a new workspace at the given path.
pub fn init_workspace(root: &Path, name: &str) -> Result<WorkspaceManifest> {
    let smctl_dir = root.join(".smctl");
    std::fs::create_dir_all(&smctl_dir).context("failed to create .smctl directory")?;

    let manifest = WorkspaceManifest {
        workspace: WorkspaceConfig {
            name: name.to_string(),
            root: ".".to_string(),
        },
        repos: Vec::new(),
        flow: FlowConfig::default(),
        worktree: WorktreeConfig::default(),
        spec: SpecConfig::default(),
    };

    manifest.save_to_root(root)?;
    tracing::info!("initialized workspace '{}' at {}", name, root.display());
    Ok(manifest)
}

/// Add a repo to the workspace manifest.
pub fn add_repo(
    manifest: &mut WorkspaceManifest,
    name: &str,
    url: &str,
    path: Option<&str>,
) -> Result<()> {
    if manifest.find_repo(name).is_some() {
        anyhow::bail!("repo '{name}' already exists in workspace");
    }

    manifest.repos.push(RepoConfig {
        name: name.to_string(),
        url: url.to_string(),
        path: path.map(|s| s.to_string()),
        default_branch: "main".to_string(),
        smctl_home: false,
        build_cmd: None,
        test_cmd: None,
        clean_cmd: None,
        depends_on: Vec::new(),
    });

    tracing::info!("added repo '{name}' to workspace");
    Ok(())
}

/// Remove a repo from the workspace manifest.
pub fn remove_repo(manifest: &mut WorkspaceManifest, name: &str) -> Result<()> {
    let len = manifest.repos.len();
    manifest.repos.retain(|r| r.name != name);
    if manifest.repos.len() == len {
        anyhow::bail!("repo '{name}' not found in workspace");
    }
    tracing::info!("removed repo '{name}' from workspace");
    Ok(())
}

/// Get status for a single repo.
pub fn repo_status(root: &Path, repo: &RepoConfig) -> Result<RepoStatus> {
    let repo_path = root.join(repo.local_path());
    let git_repo = git2::Repository::open(&repo_path)
        .with_context(|| format!("failed to open git repo at {}", repo_path.display()))?;

    let head = git_repo.head().context("failed to get HEAD")?;
    let branch = head.shorthand().unwrap_or("detached").to_string();

    let statuses = git_repo
        .statuses(None)
        .context("failed to get git status")?;

    let modified_files = statuses.len();
    let clean = modified_files == 0;

    Ok(RepoStatus {
        name: repo.name.clone(),
        branch,
        clean,
        ahead: 0,
        behind: 0,
        modified_files,
    })
}

// ── Worktree management (merged from smctl-worktree) ────────────────

pub mod worktree {
    use std::path::{Path, PathBuf};

    use anyhow::{Context, Result};
    use serde::{Deserialize, Serialize};

    use crate::WorkspaceManifest;

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
            let manifest = crate::WorkspaceManifest::parse(
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
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TOML: &str = r#"
[workspace]
name = "test-workspace"

[[repos]]
name = "SmallAIOS"
url = "https://github.com/SmallAIOS/SmallAIOS"
path = "smallaios"
default_branch = "main"

[[repos]]
name = "ModelGate"
url = "https://github.com/SmallAIOS/ModelGate"
default_branch = "main"
smctl_home = true
depends_on = ["SmallAIOS"]
"#;

    #[test]
    fn test_parse_workspace_manifest() {
        let manifest = WorkspaceManifest::parse(SAMPLE_TOML).unwrap();
        assert_eq!(manifest.workspace.name, "test-workspace");
        assert_eq!(manifest.repos.len(), 2);
        assert_eq!(manifest.repos[0].name, "SmallAIOS");
        assert_eq!(manifest.repos[0].local_path(), "smallaios");
        assert_eq!(manifest.repos[1].name, "ModelGate");
        assert!(manifest.repos[1].smctl_home);
        assert_eq!(manifest.repos[1].depends_on, vec!["SmallAIOS"]);
    }

    #[test]
    fn test_default_flow_config() {
        let flow = FlowConfig::default();
        assert_eq!(flow.main_branch, "main");
        assert_eq!(flow.develop_branch, "develop");
        assert_eq!(flow.feature_prefix, "feature/");
    }

    #[test]
    fn test_find_repo() {
        let manifest = WorkspaceManifest::parse(SAMPLE_TOML).unwrap();
        assert!(manifest.find_repo("SmallAIOS").is_some());
        assert!(manifest.find_repo("NonExistent").is_none());
    }

    #[test]
    fn test_add_remove_repo() {
        let mut manifest = WorkspaceManifest::parse(SAMPLE_TOML).unwrap();
        add_repo(&mut manifest, "NewRepo", "https://example.com/new", None).unwrap();
        assert_eq!(manifest.repos.len(), 3);

        // Duplicate should fail
        assert!(add_repo(&mut manifest, "NewRepo", "https://example.com/new", None).is_err());

        remove_repo(&mut manifest, "NewRepo").unwrap();
        assert_eq!(manifest.repos.len(), 2);

        // Remove non-existent should fail
        assert!(remove_repo(&mut manifest, "NonExistent").is_err());
    }

    #[test]
    fn test_init_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = init_workspace(dir.path(), "test").unwrap();
        assert_eq!(manifest.workspace.name, "test");
        assert!(dir.path().join(".smctl/workspace.toml").exists());
    }

    #[test]
    fn test_roundtrip_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = init_workspace(dir.path(), "roundtrip").unwrap();
        let loaded = WorkspaceManifest::load_from_root(dir.path()).unwrap();
        assert_eq!(loaded.workspace.name, manifest.workspace.name);
    }
}
