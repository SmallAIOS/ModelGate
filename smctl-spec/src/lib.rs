use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// OpenSpec feature lifecycle phases.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpecPhase {
    New,
    Draft,
    Active,
    Archived,
}

/// A parsed spec feature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecInfo {
    pub name: String,
    pub phase: SpecPhase,
    pub path: PathBuf,
    pub has_proposal: bool,
    pub has_design: bool,
    pub has_tasks: bool,
    pub tasks_total: usize,
    pub tasks_done: usize,
}

/// Spec validation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub name: String,
    pub valid: bool,
    pub issues: Vec<String>,
}

/// Create a new OpenSpec feature folder with scaffolded documents.
pub fn new_spec(openspec_dir: &Path, name: &str) -> Result<SpecInfo> {
    let spec_dir = openspec_dir.join("changes").join(name);
    if spec_dir.exists() {
        anyhow::bail!(
            "spec '{name}' already exists at {}. The create was rejected to avoid overwriting existing scaffolds. Run `smctl spec archive {name}` to retire the existing spec, or choose a different name.",
            spec_dir.display()
        );
    }

    std::fs::create_dir_all(spec_dir.join("specs")).context("failed to create spec directories")?;

    // Scaffold .openspec.yaml
    std::fs::write(
        spec_dir.join(".openspec.yaml"),
        format!(
            "schema: spec-driven\ncreated: {}\nstatus: draft\n",
            chrono_date()
        ),
    )?;

    // Scaffold proposal.md
    std::fs::write(
        spec_dir.join("proposal.md"),
        format!(
            "# {name} — Proposal\n\n\
             ## Why\n\n\
             <!-- Describe the problem this change addresses -->\n\n\
             ## What Changes\n\n\
             <!-- Describe the proposed solution -->\n\n\
             ## Capabilities\n\n\
             ### New Capabilities\n\n\
             - \n\n\
             ### Modified Capabilities\n\n\
             - (None)\n\n\
             ## Impact\n\n\
             ### Affected Repos\n\n\
             | Repository | Impact |\n\
             |---|---|\n\
             | | |\n\n\
             ## References\n\n\
             - \n"
        ),
    )?;

    // Scaffold design.md
    std::fs::write(
        spec_dir.join("design.md"),
        format!(
            "# {name} — Design Document\n\n\
             ## Context\n\n\
             <!-- Technical context -->\n\n\
             ## Goals / Non-Goals\n\n\
             ### Goals\n\n\
             1. \n\n\
             ### Non-Goals\n\n\
             1. \n\n\
             ## Decisions\n\n\
             ### Decision 1: \n\n\
             **Choice:** \n\n\
             **Rationale:** \n\n\
             ## Risks / Trade-offs\n\n\
             | Risk | Mitigation |\n\
             |---|---|\n\
             | | |\n\n\
             ## Open Questions\n\n\
             1. \n"
        ),
    )?;

    // Scaffold tasks.md
    std::fs::write(
        spec_dir.join("tasks.md"),
        format!(
            "# {name} — Tasks\n\n\
             ## Implementation\n\n\
             - [ ] \n\n\
             ## Testing\n\n\
             - [ ] \n\n\
             ## Documentation\n\n\
             - [ ] \n\n\
             ## Verify\n\n\
             - [ ] All tests pass\n"
        ),
    )?;

    tracing::info!("created spec '{name}' at {}", spec_dir.display());

    Ok(SpecInfo {
        name: name.to_string(),
        phase: SpecPhase::Draft,
        path: spec_dir,
        has_proposal: true,
        has_design: true,
        has_tasks: true,
        tasks_total: 1,
        tasks_done: 0,
    })
}

/// Parse tasks.md checkboxes and return progress info.
pub fn parse_tasks(tasks_path: &Path) -> Result<(usize, usize)> {
    let content = std::fs::read_to_string(tasks_path).context("failed to read tasks.md")?;

    let mut total = 0;
    let mut done = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]") {
            total += 1;
            done += 1;
        } else if trimmed.starts_with("- [ ]") {
            total += 1;
        }
    }

    Ok((total, done))
}

/// Get info about a spec.
pub fn spec_info(openspec_dir: &Path, name: &str) -> Result<SpecInfo> {
    let spec_dir = openspec_dir.join("changes").join(name);
    if !spec_dir.exists() {
        // Check archive
        let archive_dir = openspec_dir.join("changes").join("archive");
        if archive_dir.exists() {
            let entries = std::fs::read_dir(&archive_dir)?;
            for entry in entries {
                let entry = entry?;
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.ends_with(name) {
                    return build_spec_info(name, &entry.path(), SpecPhase::Archived);
                }
            }
        }
        anyhow::bail!(
            "spec '{name}' not found. No active or archived spec matches that name. Run `smctl spec list` to see existing specs, or `smctl spec new {name}` to create it."
        );
    }

    let phase = if spec_dir.join("tasks.md").exists() {
        let (total, done) = parse_tasks(&spec_dir.join("tasks.md"))?;
        if total > 0 && total == done {
            SpecPhase::Active
        } else {
            SpecPhase::Draft
        }
    } else {
        SpecPhase::New
    };

    build_spec_info(name, &spec_dir, phase)
}

/// Validate a spec for completeness.
pub fn validate(openspec_dir: &Path, name: &str) -> Result<ValidationResult> {
    let spec_dir = openspec_dir.join("changes").join(name);
    if !spec_dir.exists() {
        anyhow::bail!(
            "spec '{name}' not found. Validation has no spec folder to inspect. Run `smctl spec list` to see existing specs, or `smctl spec new {name}` to create it."
        );
    }

    let mut issues = Vec::new();

    if !spec_dir.join("proposal.md").exists() {
        issues.push("missing proposal.md".to_string());
    }
    if !spec_dir.join("design.md").exists() {
        issues.push("missing design.md".to_string());
    }
    if !spec_dir.join("tasks.md").exists() {
        issues.push("missing tasks.md".to_string());
    } else {
        let content = std::fs::read_to_string(spec_dir.join("tasks.md"))?;
        if !content.contains("- [") {
            issues.push("tasks.md has no task checkboxes".to_string());
        }
    }

    // Check proposal.md content
    if spec_dir.join("proposal.md").exists() {
        let content = std::fs::read_to_string(spec_dir.join("proposal.md"))?;
        if !content.contains("## Why") {
            issues.push("proposal.md missing '## Why' section".to_string());
        }
        if !content.contains("## What Changes") {
            issues.push("proposal.md missing '## What Changes' section".to_string());
        }
    }

    // Check design.md content
    if spec_dir.join("design.md").exists() {
        let content = std::fs::read_to_string(spec_dir.join("design.md"))?;
        if !content.contains("## Decisions") {
            issues.push("design.md missing '## Decisions' section".to_string());
        }
    }

    Ok(ValidationResult {
        name: name.to_string(),
        valid: issues.is_empty(),
        issues,
    })
}

/// List all specs (active + archived).
pub fn list_specs(openspec_dir: &Path) -> Result<Vec<SpecInfo>> {
    let mut specs = Vec::new();
    let changes_dir = openspec_dir.join("changes");

    if !changes_dir.exists() {
        return Ok(specs);
    }

    let entries = std::fs::read_dir(&changes_dir)?;
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "archive" {
            continue;
        }
        if let Ok(info) = spec_info(openspec_dir, &name) {
            specs.push(info);
        }
    }

    // List archived specs
    let archive_dir = changes_dir.join("archive");
    if archive_dir.exists() {
        let entries = std::fs::read_dir(&archive_dir)?;
        for entry in entries {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Ok(info) = build_spec_info(&name, &entry.path(), SpecPhase::Archived) {
                    specs.push(info);
                }
            }
        }
    }

    Ok(specs)
}

/// Archive a spec: move to archive directory.
pub fn archive(openspec_dir: &Path, name: &str) -> Result<PathBuf> {
    let spec_dir = openspec_dir.join("changes").join(name);
    if !spec_dir.exists() {
        anyhow::bail!(
            "spec '{name}' not found. Archive has no active spec to move. Run `smctl spec list` to see existing specs."
        );
    }

    let archive_dir = openspec_dir.join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir)?;

    let date = chrono_date();
    let dest = archive_dir.join(format!("{date}-{name}"));
    std::fs::rename(&spec_dir, &dest).context("failed to move spec to archive")?;

    tracing::info!("archived spec '{name}' to {}", dest.display());
    Ok(dest)
}

// --- Per-repo aggregation ---
//
// Multi-repo workspaces hold one openspec/ tree per registered repo.
// The functions below extend the single-root API above to walk a
// slice of (repo_name, openspec_dir) pairs and either aggregate or
// resolve. The CLI is responsible for assembling the slice from
// `smctl_workspace::WorkspaceManifest` — keeps this crate independent
// of the manifest schema.

/// One row in an aggregated `list_specs_across` result. Pairs the
/// existing `SpecInfo` with the repo name that owns the spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSpecInfo {
    pub repo: String,
    #[serde(flatten)]
    pub info: SpecInfo,
}

/// Reference to a single spec inside a known repo. Returned by
/// [`find_spec_in_repos`] so callers can dispatch the existing
/// single-root helpers (`validate`, `spec_info`, `archive_in_repo`)
/// against the resolved openspec_dir.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoSpecRef {
    pub repo: String,
    pub name: String,
    pub openspec_dir: PathBuf,
}

impl RepoSpecRef {
    /// Render in the canonical `repo:name` qualified form.
    pub fn qualified(&self) -> String {
        format!("{}:{}", self.repo, self.name)
    }
}

/// Outcome of [`find_spec_in_repos`] when the bare-name path can't
/// resolve a single ref.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ResolveError {
    #[error("spec '{name}' not found in any registered repo")]
    NotFound { name: String },

    #[error(
        "spec '{name}' is ambiguous — declared in {} repos: {}. \
         Re-run with the qualified form to pick one (e.g. `{}`).",
        matches.len(),
        matches.iter().map(|m| m.repo.as_str()).collect::<Vec<_>>().join(", "),
        matches.first().map(|m| m.qualified()).unwrap_or_default(),
    )]
    Ambiguous {
        name: String,
        matches: Vec<RepoSpecRef>,
    },
}

/// List every active + archived spec across each registered repo.
/// Empty `repos` returns an empty list (no error).
pub fn list_specs_across(repos: &[(String, PathBuf)]) -> Result<Vec<RepoSpecInfo>> {
    let mut out = Vec::new();
    for (repo_name, openspec_dir) in repos {
        // A registered repo without an openspec/ directory is fine —
        // contribute nothing rather than erroring. Mirrors the
        // single-root list_specs which returns Ok(vec![]) when the
        // changes/ subdir is absent.
        if !openspec_dir.exists() {
            continue;
        }
        let specs = list_specs(openspec_dir)?;
        for info in specs {
            out.push(RepoSpecInfo {
                repo: repo_name.clone(),
                info,
            });
        }
    }
    Ok(out)
}

/// Resolve a spec name across registered repos.
///
/// See `openspec/changes/openspec-aggregate-v1/design.md` Decision 2
/// for the four-rule resolution table:
///
/// 1. Qualified `repo:name` looks up directly.
/// 2. Bare name in exactly one repo resolves unambiguously.
/// 3. Bare name in multiple repos returns `Ambiguous`.
/// 4. Bare name in no repo returns `NotFound`.
pub fn find_spec_in_repos(
    repos: &[(String, PathBuf)],
    input: &str,
) -> std::result::Result<RepoSpecRef, ResolveError> {
    // Rule 1: qualified form.
    if let Some((repo, name)) = input.split_once(':') {
        let openspec_dir = repos
            .iter()
            .find(|(r, _)| r == repo)
            .map(|(_, p)| p.clone())
            .ok_or_else(|| ResolveError::NotFound {
                name: input.to_string(),
            })?;
        let candidate = openspec_dir.join("changes").join(name);
        if !candidate.exists() {
            return Err(ResolveError::NotFound {
                name: input.to_string(),
            });
        }
        return Ok(RepoSpecRef {
            repo: repo.to_string(),
            name: name.to_string(),
            openspec_dir,
        });
    }

    // Rules 2–4: bare name across every repo.
    let mut matches: Vec<RepoSpecRef> = Vec::new();
    for (repo, openspec_dir) in repos {
        let candidate = openspec_dir.join("changes").join(input);
        if candidate.exists() {
            matches.push(RepoSpecRef {
                repo: repo.clone(),
                name: input.to_string(),
                openspec_dir: openspec_dir.clone(),
            });
        }
    }
    match matches.len() {
        0 => Err(ResolveError::NotFound {
            name: input.to_string(),
        }),
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => Err(ResolveError::Ambiguous {
            name: input.to_string(),
            matches,
        }),
    }
}

/// Archive a spec in a specific repo's openspec tree. Same shape as
/// [`archive`] but documents the per-repo intent.
///
/// Returns the absolute destination path under
/// `<openspec_dir>/changes/archive/<YYYY-MM-DD>-<name>/`.
pub fn archive_in_repo(openspec_dir: &Path, name: &str) -> Result<PathBuf> {
    archive(openspec_dir, name)
}

/// Add a synthetic `_workspace` entry to `repos` when the workspace
/// root carries its own `openspec/` directory and no registered repo
/// already covers that path.
///
/// Solves the legacy single-repo case where a workspace was created
/// before any `[[repos]]` entries were added — common in this very
/// repo today. De-duplication is by absolute path of the openspec
/// directory.
pub fn inject_synthetic_workspace_repo(
    workspace_root: &Path,
    openspec_dirname: &str,
    repos: &mut Vec<(String, PathBuf)>,
) {
    let synthetic_path = workspace_root.join(openspec_dirname);
    if !synthetic_path.exists() {
        return;
    }
    // Skip when an explicit repo already covers this path. Compare
    // canonicalised paths so symlink / relative-form differences
    // don't cause double-counts.
    let synth_canon = synthetic_path.canonicalize().ok();
    for (_, existing) in repos.iter() {
        if let (Some(a), Some(b)) = (synth_canon.as_ref(), existing.canonicalize().ok().as_ref())
            && a == b
        {
            return;
        }
    }
    repos.push(("_workspace".to_string(), synthetic_path));
}

// --- Internal helpers ---

fn build_spec_info(name: &str, path: &Path, phase: SpecPhase) -> Result<SpecInfo> {
    let has_proposal = path.join("proposal.md").exists();
    let has_design = path.join("design.md").exists();
    let has_tasks = path.join("tasks.md").exists();

    let (tasks_total, tasks_done) = if has_tasks {
        parse_tasks(&path.join("tasks.md")).unwrap_or((0, 0))
    } else {
        (0, 0)
    };

    Ok(SpecInfo {
        name: name.to_string(),
        phase,
        path: path.to_path_buf(),
        has_proposal,
        has_design,
        has_tasks,
        tasks_total,
        tasks_done,
    })
}

fn chrono_date() -> String {
    // Simple date without external chrono dependency
    let output = std::process::Command::new("date").arg("+%Y-%m-%d").output();
    match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tasks() {
        let dir = tempfile::tempdir().unwrap();
        let tasks = dir.path().join("tasks.md");
        std::fs::write(
            &tasks,
            "# Tasks\n\
             - [x] Done task\n\
             - [ ] Pending task\n\
             - [X] Also done\n\
             - [ ] Another pending\n",
        )
        .unwrap();

        let (total, done) = parse_tasks(&tasks).unwrap();
        assert_eq!(total, 4);
        assert_eq!(done, 2);
    }

    #[test]
    fn test_new_spec() {
        let dir = tempfile::tempdir().unwrap();
        let info = new_spec(dir.path(), "test-feature").unwrap();
        assert_eq!(info.name, "test-feature");
        assert_eq!(info.phase, SpecPhase::Draft);
        assert!(info.has_proposal);
        assert!(info.has_design);
        assert!(info.has_tasks);
        assert!(dir.path().join("changes/test-feature/proposal.md").exists());
        assert!(dir.path().join("changes/test-feature/design.md").exists());
        assert!(dir.path().join("changes/test-feature/tasks.md").exists());
    }

    #[test]
    fn test_new_spec_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        new_spec(dir.path(), "dup").unwrap();
        assert!(new_spec(dir.path(), "dup").is_err());
    }

    #[test]
    fn test_validate_spec() {
        let dir = tempfile::tempdir().unwrap();
        new_spec(dir.path(), "valid-spec").unwrap();
        let result = validate(dir.path(), "valid-spec").unwrap();
        assert!(result.valid, "issues: {:?}", result.issues);
    }

    #[test]
    fn test_list_specs() {
        let dir = tempfile::tempdir().unwrap();
        new_spec(dir.path(), "spec-a").unwrap();
        new_spec(dir.path(), "spec-b").unwrap();
        let specs = list_specs(dir.path()).unwrap();
        assert_eq!(specs.len(), 2);
    }

    #[test]
    fn test_archive_spec() {
        let dir = tempfile::tempdir().unwrap();
        new_spec(dir.path(), "to-archive").unwrap();
        let dest = archive(dir.path(), "to-archive").unwrap();
        assert!(dest.exists());
        assert!(!dir.path().join("changes/to-archive").exists());
    }

    #[test]
    fn test_spec_info_phase_draft() {
        let dir = tempfile::tempdir().unwrap();
        new_spec(dir.path(), "draft-spec").unwrap();
        let info = spec_info(dir.path(), "draft-spec").unwrap();
        assert_eq!(info.phase, SpecPhase::Draft);
        // Scaffolded tasks.md has 1 pending task, 0 done
        assert!(info.tasks_done < info.tasks_total);
    }

    #[test]
    fn test_spec_info_phase_active_when_all_done() {
        let dir = tempfile::tempdir().unwrap();
        new_spec(dir.path(), "done-spec").unwrap();
        // Overwrite tasks.md with all tasks completed
        std::fs::write(
            dir.path().join("changes/done-spec/tasks.md"),
            "# Tasks\n- [x] Task one\n- [x] Task two\n",
        )
        .unwrap();
        let info = spec_info(dir.path(), "done-spec").unwrap();
        assert_eq!(info.phase, SpecPhase::Active);
        assert_eq!(info.tasks_done, 2);
        assert_eq!(info.tasks_total, 2);
    }

    #[test]
    fn test_validate_missing_sections() {
        let dir = tempfile::tempdir().unwrap();
        new_spec(dir.path(), "bad-spec").unwrap();
        // Overwrite proposal.md with empty content
        std::fs::write(dir.path().join("changes/bad-spec/proposal.md"), "# Empty\n").unwrap();
        let result = validate(dir.path(), "bad-spec").unwrap();
        assert!(!result.valid);
        assert!(result.issues.iter().any(|i| i.contains("Why")));
    }

    // --- Per-repo aggregation ---

    /// Build a fixture: two temp repos, each with one active spec.
    /// Caller owns the `TempDir`s — keep them alive for the test.
    fn two_repo_fixture() -> (tempfile::TempDir, tempfile::TempDir, Vec<(String, PathBuf)>) {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        new_spec(a.path(), "alpha").unwrap();
        new_spec(b.path(), "beta").unwrap();
        let repos = vec![
            ("RepoA".to_string(), a.path().to_path_buf()),
            ("RepoB".to_string(), b.path().to_path_buf()),
        ];
        (a, b, repos)
    }

    #[test]
    fn list_specs_across_aggregates_two_repos() {
        let (_a, _b, repos) = two_repo_fixture();
        let specs = list_specs_across(&repos).unwrap();
        // One row per spec; alpha came from RepoA, beta from RepoB.
        assert_eq!(specs.len(), 2);
        assert!(
            specs
                .iter()
                .any(|s| s.repo == "RepoA" && s.info.name == "alpha")
        );
        assert!(
            specs
                .iter()
                .any(|s| s.repo == "RepoB" && s.info.name == "beta")
        );
    }

    #[test]
    fn list_specs_across_empty_input_returns_empty() {
        let specs = list_specs_across(&[]).unwrap();
        assert!(specs.is_empty());
    }

    #[test]
    fn list_specs_across_skips_repo_without_openspec_dir() {
        // First repo has openspec/, second doesn't.
        let a = tempfile::tempdir().unwrap();
        new_spec(a.path(), "only-spec").unwrap();
        let b = tempfile::tempdir().unwrap(); // no openspec/

        let repos = vec![
            ("HasSpec".to_string(), a.path().to_path_buf()),
            ("NoSpec".to_string(), b.path().join("openspec")),
        ];
        let specs = list_specs_across(&repos).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].repo, "HasSpec");
    }

    #[test]
    fn find_spec_in_repos_qualified_resolves_directly() {
        let (_a, _b, repos) = two_repo_fixture();
        let r = find_spec_in_repos(&repos, "RepoA:alpha").unwrap();
        assert_eq!(r.repo, "RepoA");
        assert_eq!(r.name, "alpha");
        assert_eq!(r.qualified(), "RepoA:alpha");
    }

    #[test]
    fn find_spec_in_repos_bare_unambiguous() {
        let (_a, _b, repos) = two_repo_fixture();
        // alpha lives only in RepoA; bare name should resolve.
        let r = find_spec_in_repos(&repos, "alpha").unwrap();
        assert_eq!(r.repo, "RepoA");
    }

    #[test]
    fn find_spec_in_repos_bare_ambiguous() {
        // Both repos declare "shared" — bare lookup must error with
        // both matches in the payload.
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        new_spec(a.path(), "shared").unwrap();
        new_spec(b.path(), "shared").unwrap();
        let repos = vec![
            ("A".to_string(), a.path().to_path_buf()),
            ("B".to_string(), b.path().to_path_buf()),
        ];
        let err = find_spec_in_repos(&repos, "shared").unwrap_err();
        match err {
            ResolveError::Ambiguous { matches, .. } => {
                assert_eq!(matches.len(), 2);
                let repos_in_match: Vec<&str> = matches.iter().map(|m| m.repo.as_str()).collect();
                assert!(repos_in_match.contains(&"A"));
                assert!(repos_in_match.contains(&"B"));
            }
            other => panic!("expected Ambiguous, got {other:?}"),
        }
    }

    #[test]
    fn find_spec_in_repos_not_found() {
        let (_a, _b, repos) = two_repo_fixture();
        let err = find_spec_in_repos(&repos, "nonexistent").unwrap_err();
        assert!(matches!(err, ResolveError::NotFound { .. }));
    }

    #[test]
    fn find_spec_in_repos_qualified_unknown_repo() {
        let (_a, _b, repos) = two_repo_fixture();
        let err = find_spec_in_repos(&repos, "WrongRepo:alpha").unwrap_err();
        assert!(matches!(err, ResolveError::NotFound { .. }));
    }

    #[test]
    fn find_spec_in_repos_qualified_unknown_name() {
        let (_a, _b, repos) = two_repo_fixture();
        let err = find_spec_in_repos(&repos, "RepoA:nonexistent").unwrap_err();
        assert!(matches!(err, ResolveError::NotFound { .. }));
    }

    #[test]
    fn archive_in_repo_moves_into_repo_archive_tree() {
        let dir = tempfile::tempdir().unwrap();
        new_spec(dir.path(), "doomed").unwrap();
        let dest = archive_in_repo(dir.path(), "doomed").unwrap();
        // Must land under <openspec_dir>/changes/archive/<YYYY-MM-DD>-doomed.
        assert!(dest.starts_with(dir.path().join("changes/archive/")));
        assert!(
            dest.file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with("-doomed")
        );
        assert!(!dir.path().join("changes/doomed").exists());
    }

    #[test]
    fn inject_synthetic_workspace_repo_adds_when_dir_exists() {
        let workspace = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(workspace.path().join("openspec/changes")).unwrap();
        let mut repos: Vec<(String, PathBuf)> = Vec::new();
        inject_synthetic_workspace_repo(workspace.path(), "openspec", &mut repos);
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].0, "_workspace");
        assert_eq!(repos[0].1, workspace.path().join("openspec"));
    }

    #[test]
    fn inject_synthetic_workspace_repo_skips_when_no_openspec_dir() {
        let workspace = tempfile::tempdir().unwrap();
        let mut repos: Vec<(String, PathBuf)> = Vec::new();
        inject_synthetic_workspace_repo(workspace.path(), "openspec", &mut repos);
        assert!(repos.is_empty());
    }

    #[test]
    fn inject_synthetic_workspace_repo_dedupes_against_explicit_repo() {
        let workspace = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(workspace.path().join("openspec/changes")).unwrap();
        // Pretend an explicit repo already covers the workspace path.
        let mut repos = vec![("ModelGate".to_string(), workspace.path().join("openspec"))];
        inject_synthetic_workspace_repo(workspace.path(), "openspec", &mut repos);
        // Should remain a single entry (the explicit one).
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].0, "ModelGate");
    }
}
