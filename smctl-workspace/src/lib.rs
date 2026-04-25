use std::path::{Path, PathBuf};

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
    /// Optional `[logging]` table. When absent, callers use the same
    /// defaults as the CLI: stderr only, INFO level, `local0` facility.
    /// Declared in `openspec/changes/smctl-logging-v1/specs/logging.md`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingManifestSection>,

    /// Optional `[gate]` table. When absent, callers use the
    /// smctl-gate defaults (`http://localhost:8080`, 30s timeout).
    /// Declared in `openspec/changes/smctl-gate-v1/specs/gate-api.md`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<GateManifestSection>,
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

/// The `[logging]` section of `workspace.toml`.
///
/// All fields are optional. When absent, each maps to its default:
///
/// - `transports` — `["stderr"]`
/// - `level` — `"info"`
/// - `facility` — `"local0"` (numeric `16`)
/// - `file` — unset
///
/// Facility names are validated at parse time. Only names from the
/// spec's facility table are accepted (`daemon`, `local0` through
/// `local7`). Unknown names produce a parse error rather than a
/// silent fallback.
///
/// Precedence is resolved by the CLI: CLI flags > env vars > this
/// section > built-in defaults.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoggingManifestSection {
    /// Active transports. Each entry is one of `"stderr"`, `"file"`,
    /// `"syslog"`. Empty vector is equivalent to the default.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub transports: Vec<String>,

    /// Path for the file transport. Only read when `transports`
    /// contains `"file"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<PathBuf>,

    /// Syslog facility name (`"local0"`..`"local7"` or `"daemon"`).
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_facility"
    )]
    pub facility: Option<String>,

    /// Minimum severity: `"error"`, `"warn"`, `"info"`, `"debug"`,
    /// `"trace"`. Case-insensitive at parse time in the CLI resolver.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

/// The `[gate]` section of `workspace.toml`.
///
/// Both fields are optional. When unset, the defaults in
/// `smctl_gate::GateConfig::default()` apply. Precedence is resolved by
/// the CLI: CLI flags / env vars (`MODELGATE_URL`) > this section >
/// built-in defaults.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateManifestSection {
    /// ModelGate endpoint URL (e.g. `"http://localhost:8080"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Request timeout in seconds for all gate operations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

/// Map a facility name from the spec's table to its RFC 5424 numeric
/// code. Returns `None` for unknown names. Exposed so the CLI
/// precedence resolver can translate the manifest value without
/// re-implementing the table.
pub fn facility_code(name: &str) -> Option<u8> {
    match name {
        "daemon" => Some(3),
        "local0" => Some(16),
        "local1" => Some(17),
        "local2" => Some(18),
        "local3" => Some(19),
        "local4" => Some(20),
        "local5" => Some(21),
        "local6" => Some(22),
        "local7" => Some(23),
        _ => None,
    }
}

fn deserialize_facility<'de, D>(de: D) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error as _;
    let opt: Option<String> = Option::deserialize(de)?;
    if let Some(ref name) = opt
        && facility_code(name).is_none()
    {
        return Err(D::Error::custom(format!(
            "unknown facility '{name}'; expected one of daemon, local0..local7"
        )));
    }
    Ok(opt)
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
        logging: None,
        gate: None,
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
        anyhow::bail!(
            "repo '{name}' already exists in workspace. The add was rejected to avoid overwriting config. Run `smctl workspace remove {name}` first, or choose a different name."
        );
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
        anyhow::bail!(
            "repo '{name}' not found in workspace. The remove had no target to act on. Run `smctl workspace status` to list configured repos."
        );
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
                        "failed to add worktree for {} at {}: {}. Git refused the worktree creation, so no worktree set was produced. Inspect existing worktrees with `smctl worktree list`, then retry or remove the stale entry with `smctl worktree remove <name>`.",
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
            anyhow::bail!(
                "worktree set '{name}' does not exist. The remove had no target to act on. Run `smctl worktree list` to see active worktree sets, or create one with `smctl worktree add {name}`."
            );
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
            anyhow::bail!(
                "worktree set '{name}' does not exist. The path lookup has no worktree to return. Run `smctl worktree list` to see active worktree sets, or create one with `smctl worktree add {name}`."
            );
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

    #[test]
    fn test_logging_section_parses() {
        let toml_text = r#"
[workspace]
name = "log-ws"

[logging]
transports = ["stderr", "file"]
file = "/var/log/smctl.log"
facility = "local2"
level = "debug"
"#;
        let manifest = WorkspaceManifest::parse(toml_text).unwrap();
        let logging = manifest.logging.expect("logging section present");
        assert_eq!(logging.transports, vec!["stderr", "file"]);
        assert_eq!(
            logging.file.as_deref(),
            Some(Path::new("/var/log/smctl.log"))
        );
        assert_eq!(logging.facility.as_deref(), Some("local2"));
        assert_eq!(logging.level.as_deref(), Some("debug"));
    }

    #[test]
    fn test_logging_section_absent_is_none() {
        let toml_text = r#"
[workspace]
name = "no-log"
"#;
        let manifest = WorkspaceManifest::parse(toml_text).unwrap();
        assert!(manifest.logging.is_none());
    }

    #[test]
    fn test_logging_section_rejects_unknown_facility() {
        let toml_text = r#"
[workspace]
name = "bad-facility"

[logging]
facility = "kern"
"#;
        let err = WorkspaceManifest::parse(toml_text).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("kern"),
            "error should cite the bad name: {msg}"
        );
    }

    #[test]
    fn test_gate_section_parses() {
        let toml_text = r#"
[workspace]
name = "gate-ws"

[gate]
url = "http://gate.internal:9000"
timeout_secs = 120
"#;
        let manifest = WorkspaceManifest::parse(toml_text).unwrap();
        let gate = manifest.gate.expect("gate section present");
        assert_eq!(gate.url.as_deref(), Some("http://gate.internal:9000"));
        assert_eq!(gate.timeout_secs, Some(120));
    }

    #[test]
    fn test_gate_section_absent_is_none() {
        let toml_text = r#"
[workspace]
name = "no-gate"
"#;
        let manifest = WorkspaceManifest::parse(toml_text).unwrap();
        assert!(manifest.gate.is_none());
    }

    #[test]
    fn test_gate_section_rejects_unknown_fields() {
        let toml_text = r#"
[workspace]
name = "strict-gate"

[gate]
url = "http://x:8080"
bogus = "nope"
"#;
        let err = WorkspaceManifest::parse(toml_text).unwrap_err();
        let msg = format!("{err:#}");
        assert!(
            msg.contains("bogus"),
            "error should cite the unknown key: {msg}"
        );
    }

    #[test]
    fn test_facility_code_mapping() {
        assert_eq!(facility_code("daemon"), Some(3));
        assert_eq!(facility_code("local0"), Some(16));
        assert_eq!(facility_code("local7"), Some(23));
        assert_eq!(facility_code("kern"), None);
        assert_eq!(facility_code("local8"), None);
    }

    #[test]
    fn test_logging_section_roundtrip_save_load() {
        let dir = tempfile::tempdir().unwrap();
        let mut manifest = init_workspace(dir.path(), "roundtrip-logging").unwrap();
        manifest.logging = Some(LoggingManifestSection {
            transports: vec!["stderr".to_string(), "syslog".to_string()],
            file: Some(PathBuf::from("/tmp/smctl.log")),
            facility: Some("local3".to_string()),
            level: Some("warn".to_string()),
        });
        manifest.save_to_root(dir.path()).unwrap();

        let loaded = WorkspaceManifest::load_from_root(dir.path()).unwrap();
        let logging = loaded.logging.expect("logging section persisted");
        assert_eq!(logging.transports, vec!["stderr", "syslog"]);
        assert_eq!(logging.file.as_deref(), Some(Path::new("/tmp/smctl.log")));
        assert_eq!(logging.facility.as_deref(), Some("local3"));
        assert_eq!(logging.level.as_deref(), Some("warn"));
    }
}
