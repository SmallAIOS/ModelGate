use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use smctl_workspace::{RepoConfig, WorkspaceManifest};

/// Build result for a single repo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub repo_name: String,
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
}

/// Overall build report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildReport {
    pub results: Vec<BuildResult>,
    pub total_duration_ms: u64,
    pub all_passed: bool,
}

/// Resolve build order from dependency graph (topological sort).
pub fn resolve_build_order(manifest: &WorkspaceManifest) -> Result<Vec<&RepoConfig>> {
    let repos = &manifest.repos;
    let mut visited = vec![false; repos.len()];
    let mut in_stack = vec![false; repos.len()];
    let mut order = Vec::new();

    fn dfs(
        idx: usize,
        repos: &[RepoConfig],
        visited: &mut [bool],
        in_stack: &mut [bool],
        order: &mut Vec<usize>,
    ) -> Result<()> {
        if in_stack[idx] {
            anyhow::bail!(
                "circular dependency detected involving '{}'",
                repos[idx].name
            );
        }
        if visited[idx] {
            return Ok(());
        }

        in_stack[idx] = true;
        for dep_name in &repos[idx].depends_on {
            if let Some(dep_idx) = repos.iter().position(|r| &r.name == dep_name) {
                dfs(dep_idx, repos, visited, in_stack, order)?;
            }
        }
        in_stack[idx] = false;
        visited[idx] = true;
        order.push(idx);
        Ok(())
    }

    for i in 0..repos.len() {
        dfs(i, repos, &mut visited, &mut in_stack, &mut order)?;
    }

    Ok(order.into_iter().map(|i| &repos[i]).collect())
}

/// Build repos in dependency order.
pub fn build(
    root: &Path,
    manifest: &WorkspaceManifest,
    repo_name: Option<&str>,
    run_tests: bool,
    clean_first: bool,
) -> Result<BuildReport> {
    let build_order = resolve_build_order(manifest)?;
    let start = std::time::Instant::now();

    let repos_to_build: Vec<_> = match repo_name {
        Some(name) => {
            // Build specific repo + its dependencies
            let _target = manifest
                .find_repo(name)
                .with_context(|| format!("repo '{name}' not found"))?;
            let deps = collect_deps(manifest, name);
            build_order
                .into_iter()
                .filter(|r| r.name == name || deps.contains(&r.name))
                .collect()
        }
        None => build_order,
    };

    let mut results = Vec::new();
    for repo in &repos_to_build {
        if clean_first && let Some(cmd) = &repo.clean_cmd {
            run_cmd(root, repo, cmd)?;
        }

        let build_cmd = repo.build_cmd.as_deref().unwrap_or("cargo build");
        let result = run_cmd(root, repo, build_cmd);
        let build_result = match result {
            Ok(output) => BuildResult {
                repo_name: repo.name.clone(),
                success: true,
                output,
                duration_ms: 0,
            },
            Err(e) => BuildResult {
                repo_name: repo.name.clone(),
                success: false,
                output: e.to_string(),
                duration_ms: 0,
            },
        };

        let build_ok = build_result.success;
        results.push(build_result);

        if !build_ok {
            break;
        }

        if run_tests {
            let test_cmd = repo.test_cmd.as_deref().unwrap_or("cargo test");
            let test_result = match run_cmd(root, repo, test_cmd) {
                Ok(output) => BuildResult {
                    repo_name: format!("{} (test)", repo.name),
                    success: true,
                    output,
                    duration_ms: 0,
                },
                Err(e) => BuildResult {
                    repo_name: format!("{} (test)", repo.name),
                    success: false,
                    output: e.to_string(),
                    duration_ms: 0,
                },
            };
            let test_ok = test_result.success;
            results.push(test_result);
            if !test_ok {
                break;
            }
        }
    }

    let all_passed = results.iter().all(|r| r.success);
    Ok(BuildReport {
        results,
        total_duration_ms: start.elapsed().as_millis() as u64,
        all_passed,
    })
}

fn run_cmd(root: &Path, repo: &RepoConfig, cmd: &str) -> Result<String> {
    let repo_path = root.join(repo.local_path());
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        anyhow::bail!("empty command");
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .current_dir(&repo_path)
        .output()
        .with_context(|| format!("failed to run '{cmd}' in {}", repo.name))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        anyhow::bail!(
            "{}: command '{}' failed:\n{}",
            repo.name,
            cmd,
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn collect_deps(manifest: &WorkspaceManifest, name: &str) -> Vec<String> {
    let mut deps = Vec::new();
    let mut stack = vec![name.to_string()];
    while let Some(current) = stack.pop() {
        if let Some(repo) = manifest.find_repo(&current) {
            for dep in &repo.depends_on {
                if !deps.contains(dep) {
                    deps.push(dep.clone());
                    stack.push(dep.clone());
                }
            }
        }
    }
    deps
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_manifest() -> WorkspaceManifest {
        WorkspaceManifest::parse(
            r#"
            [workspace]
            name = "test"

            [[repos]]
            name = "A"
            url = "https://example.com/a"
            depends_on = []

            [[repos]]
            name = "B"
            url = "https://example.com/b"
            depends_on = ["A"]

            [[repos]]
            name = "C"
            url = "https://example.com/c"
            depends_on = ["A", "B"]
            "#,
        )
        .unwrap()
    }

    #[test]
    fn test_resolve_build_order() {
        let manifest = make_manifest();
        let order = resolve_build_order(&manifest).unwrap();
        let names: Vec<_> = order.iter().map(|r| r.name.as_str()).collect();
        // A must come before B and C; B must come before C
        let a_pos = names.iter().position(|n| *n == "A").unwrap();
        let b_pos = names.iter().position(|n| *n == "B").unwrap();
        let c_pos = names.iter().position(|n| *n == "C").unwrap();
        assert!(a_pos < b_pos);
        assert!(b_pos < c_pos);
    }

    #[test]
    fn test_circular_dependency() {
        let manifest = WorkspaceManifest::parse(
            r#"
            [workspace]
            name = "test"

            [[repos]]
            name = "A"
            url = "https://example.com/a"
            depends_on = ["B"]

            [[repos]]
            name = "B"
            url = "https://example.com/b"
            depends_on = ["A"]
            "#,
        )
        .unwrap();
        assert!(resolve_build_order(&manifest).is_err());
    }

    #[test]
    fn test_collect_deps() {
        let manifest = make_manifest();
        let deps = collect_deps(&manifest, "C");
        assert!(deps.contains(&"A".to_string()));
        assert!(deps.contains(&"B".to_string()));
    }
}
