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
        anyhow::bail!("spec '{name}' already exists at {}", spec_dir.display());
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
        anyhow::bail!("spec '{name}' not found");
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
        anyhow::bail!("spec '{name}' not found");
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
        anyhow::bail!("spec '{name}' not found");
    }

    let archive_dir = openspec_dir.join("changes").join("archive");
    std::fs::create_dir_all(&archive_dir)?;

    let date = chrono_date();
    let dest = archive_dir.join(format!("{date}-{name}"));
    std::fs::rename(&spec_dir, &dest).context("failed to move spec to archive")?;

    tracing::info!("archived spec '{name}' to {}", dest.display());
    Ok(dest)
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
        std::fs::write(
            dir.path().join("changes/bad-spec/proposal.md"),
            "# Empty\n",
        )
        .unwrap();
        let result = validate(dir.path(), "bad-spec").unwrap();
        assert!(!result.valid);
        assert!(result.issues.iter().any(|i| i.contains("Why")));
    }
}
