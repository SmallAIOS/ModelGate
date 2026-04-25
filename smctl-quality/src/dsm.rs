//! `smctl quality dsm` — Design Structure Matrix / module-dependency
//! analysis via `cargo modules`.
//!
//! Wraps the `cargo-modules` binary and builds a module-level adjacency
//! map per workspace crate. Runs a depth-first search over each graph to
//! surface back-edges (cycles) and emits SMCTL-0410 DsmCycleDetected per
//! cycle found.
//!
//! Scope: this module treats `cargo modules structure --package <p>` as a
//! line-oriented source of `module -> submodule` edges. The tool does not
//! report cycles natively in stable output, so cycle detection is a small
//! DFS in Rust over the adjacency map we extract. Richer analyses (SVG
//! rendering, crate-level dep graph via cargo-depgraph, coupling depth
//! limits) are deferred to follow-up changes.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use smctl_log::MsgId;
use thiserror::Error;

/// A single module-dependency cycle in the resulting report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DsmCycle {
    /// Workspace crate that contains the cycle.
    pub crate_name: String,
    /// Starting module of the back-edge pair.
    pub module_a: String,
    /// Ending module of the back-edge pair.
    pub module_b: String,
    /// Module path observed while walking from `module_a` back to
    /// `module_b`. Joined with ` -> `.
    pub via: String,
}

/// Structured result of a `smctl quality dsm` run.
///
/// Serialises to stable JSON. Consumers should treat the shape as a
/// public contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DsmReport {
    /// Root of the workspace that was analysed.
    pub root: String,
    /// Elapsed wall time of the dsm run in milliseconds.
    pub duration_ms: u128,
    /// Workspace crates that cargo-modules was invoked against.
    pub crates_scanned: Vec<String>,
    /// Number of cycles detected. Same as `cycles.len()`.
    pub cycle_count: usize,
    /// The cycles themselves, one entry per back-edge.
    pub cycles: Vec<DsmCycle>,
    /// True when the report crossed the configured fail threshold. The
    /// caller decides exit status from this.
    pub fail: bool,
}

/// Errors raised by [`run_dsm`].
///
/// Every variant carries a three-part message (what happened / what it means
/// / what to do next) so that CLI callers can surface it directly without
/// rewrapping.
#[derive(Debug, Error)]
pub enum DsmError {
    #[error(
        "cargo-modules is not installed on PATH. smctl quality dsm cannot run without it. Install it with: cargo install cargo-modules"
    )]
    ToolMissing,

    #[error(
        "cargo-modules exited abnormally for `{package}` ({status}). The dsm result is unreliable. Re-run `cargo modules structure --package {package}` directly to inspect the failure: {stderr}"
    )]
    ToolFailed {
        package: String,
        status: String,
        stderr: String,
    },

    #[error(
        "cargo metadata could not enumerate workspace members. Run `cargo metadata --no-deps --format-version 1` directly in the workspace root to inspect the failure: {stderr}"
    )]
    MetadataFailed { stderr: String },

    #[error(
        "cargo metadata produced JSON that smctl could not parse. The Cargo version in use may emit a different shape. Upgrade Cargo or file an issue. Parser error: {source}"
    )]
    MalformedMetadata {
        #[source]
        source: serde_json::Error,
    },
}

/// Invoke `cargo modules structure` for each workspace member, build
/// adjacency maps, detect cycles, emit MSGIDs.
///
/// This is the primary public entry point of the dsm module.
pub fn run_dsm(root: &Path) -> Result<DsmReport, DsmError> {
    let root_str = root.display().to_string();
    let started = Instant::now();

    tracing::info!(
        msgid = %MsgId::QualityCheckStarted,
        verb = "dsm",
        repo = %root_str,
        scope = "workspace",
        "dsm started"
    );

    let members = workspace_members(root)?;
    let mut cycles = Vec::new();

    for package in &members {
        let output = invoke_cargo_modules(root, package)?;
        let graph = parse_modules_output(&output);
        let found = detect_cycles(package, &graph);
        for cycle in &found {
            tracing::error!(
                msgid = %MsgId::DsmCycleDetected,
                crate_name = %cycle.crate_name,
                module_a = %cycle.module_a,
                module_b = %cycle.module_b,
                via = %cycle.via,
                "module dependency cycle detected"
            );
        }
        cycles.extend(found);
    }

    let duration_ms = started.elapsed().as_millis();
    let cycle_count = cycles.len();

    Ok(DsmReport {
        root: root_str,
        duration_ms,
        crates_scanned: members,
        cycle_count,
        cycles,
        fail: false,
    })
}

/// Decide whether the report should fail the check. Any detected cycle
/// flips `fail` to true when `enforce_no_cycles` is enabled; otherwise
/// the report stays informational (report-only mode).
///
/// Emits the terminal completion MSGID (either `QualityCheckCompleted` or
/// `QualityCheckFailed`). The caller should invoke this exactly once per
/// run, after [`run_dsm`] returns.
pub fn finalise_report(mut report: DsmReport, enforce_no_cycles: bool) -> DsmReport {
    let fail = enforce_no_cycles && report.cycle_count > 0;
    report.fail = fail;

    if fail {
        let remediation = report
            .cycles
            .iter()
            .map(|c| {
                format!(
                    "break cycle {} <-> {} in `{}`",
                    c.module_a, c.module_b, c.crate_name
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        tracing::error!(
            msgid = %MsgId::QualityCheckFailed,
            verb = "dsm",
            duration_ms = report.duration_ms as u64,
            violation_count = report.cycle_count,
            remediation = %remediation,
            "dsm failed"
        );
    } else {
        tracing::info!(
            msgid = %MsgId::QualityCheckCompleted,
            verb = "dsm",
            duration_ms = report.duration_ms as u64,
            violation_count = report.cycle_count as u64,
            "dsm completed"
        );
    }

    report
}

/// Enumerate workspace member package names via `cargo metadata --no-deps`.
///
/// Returns the `packages[].name` list in alphabetical order so runs are
/// reproducible across machines.
fn workspace_members(root: &Path) -> Result<Vec<String>, DsmError> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version")
        .arg("1")
        .current_dir(root)
        .output()
        .map_err(|e| DsmError::MetadataFailed {
            stderr: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(DsmError::MetadataFailed {
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }

    let parsed: CargoMetadata = serde_json::from_slice(&output.stdout)
        .map_err(|source| DsmError::MalformedMetadata { source })?;
    let mut names: Vec<String> = parsed.packages.into_iter().map(|p| p.name).collect();
    names.sort();
    Ok(names)
}

/// Shell out to `cargo modules structure --package <p>`. Returns the
/// stdout text on success; maps ENOENT to [`DsmError::ToolMissing`] so
/// the CLI surface can render the three-part install message.
fn invoke_cargo_modules(root: &Path, package: &str) -> Result<String, DsmError> {
    let output = Command::new("cargo")
        .arg("modules")
        .arg("structure")
        .arg("--package")
        .arg(package)
        .current_dir(root)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                DsmError::ToolMissing
            } else {
                DsmError::ToolFailed {
                    package: package.to_string(),
                    status: e.kind().to_string(),
                    stderr: e.to_string(),
                }
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Cargo itself reports the subcommand is absent; surface ToolMissing.
    let looks_missing = stderr.contains("no such command")
        || stderr.contains("no such subcommand")
        || stderr.contains("is not installed");
    if looks_missing {
        return Err(DsmError::ToolMissing);
    }

    if !output.status.success() && stdout.trim().is_empty() {
        return Err(DsmError::ToolFailed {
            package: package.to_string(),
            status: output
                .status
                .code()
                .map_or_else(|| "signal".to_string(), |c| c.to_string()),
            stderr,
        });
    }

    Ok(stdout)
}

/// Parse `cargo modules structure` output into a module adjacency map.
///
/// cargo-modules prints a tree where each line looks like
/// `<prefix>mod <name>: <visibility>`. The prefix encodes depth via box-
/// drawing characters and spaces. We extract the module name per line,
/// track a stack of ancestors indexed by indentation depth, and build
/// parent-to-child edges.
///
/// Nested indentation uses 4-column box-drawing groups in modern cargo-
/// modules output; the parser counts leading non-text columns instead of
/// parsing the glyphs themselves, so format drift in the drawing style
/// does not break the map.
fn parse_modules_output(raw: &str) -> BTreeMap<String, BTreeSet<String>> {
    let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    // Stack entries are (indent_columns, module_name).
    let mut stack: Vec<(usize, String)> = Vec::new();

    for line in raw.lines() {
        // Skip blank lines and prose headers.
        let trimmed = line.trim_end();
        if trimmed.is_empty() {
            continue;
        }

        // Find the first ASCII letter — that is where the `mod` or `crate`
        // token begins. Everything before it is indent decoration.
        let indent = trimmed
            .char_indices()
            .find(|&(_, c)| c.is_ascii_alphabetic())
            .map(|(i, _)| i)
            .unwrap_or(0);
        let payload = &trimmed[indent..];

        // Accept `mod <name>` or `crate <name>` tokens; ignore others.
        let (kind, rest) = match payload.split_once(' ') {
            Some(parts) => parts,
            None => continue,
        };
        if kind != "mod" && kind != "crate" {
            continue;
        }
        // `<name>[: <visibility>]` — strip an optional trailing `:` suffix.
        let name = rest.split(':').next().unwrap_or("").trim();
        if name.is_empty() {
            continue;
        }

        // Pop deeper-or-equal entries from the stack — we are at `indent`
        // columns now, so any entry at >= indent columns is no longer an
        // ancestor.
        while let Some(&(top_indent, _)) = stack.last() {
            if top_indent >= indent {
                stack.pop();
            } else {
                break;
            }
        }

        if let Some((_, parent)) = stack.last() {
            graph
                .entry(parent.clone())
                .or_default()
                .insert(name.to_string());
        } else {
            graph.entry(name.to_string()).or_default();
        }

        stack.push((indent, name.to_string()));
    }

    graph
}

/// Depth-first search over the adjacency map to surface back-edges.
///
/// A back-edge is an edge from a descendant to an ancestor in the current
/// traversal. Each back-edge becomes one [`DsmCycle`] record with the
/// ancestor chain joined by ` -> ` in the `via` field.
fn detect_cycles(package: &str, graph: &BTreeMap<String, BTreeSet<String>>) -> Vec<DsmCycle> {
    let mut cycles = Vec::new();
    let mut visited: BTreeSet<String> = BTreeSet::new();
    let mut path: Vec<String> = Vec::new();
    let mut on_path: BTreeSet<String> = BTreeSet::new();

    let roots: Vec<String> = graph.keys().cloned().collect();
    for root in roots {
        if !visited.contains(&root) {
            dfs(
                package,
                &root,
                graph,
                &mut visited,
                &mut path,
                &mut on_path,
                &mut cycles,
            );
        }
    }

    cycles
}

fn dfs(
    package: &str,
    node: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    visited: &mut BTreeSet<String>,
    path: &mut Vec<String>,
    on_path: &mut BTreeSet<String>,
    cycles: &mut Vec<DsmCycle>,
) {
    visited.insert(node.to_string());
    path.push(node.to_string());
    on_path.insert(node.to_string());

    if let Some(children) = graph.get(node) {
        for child in children {
            if on_path.contains(child) {
                // Back-edge from `node` to `child`. The ancestor chain is
                // the slice of `path` from `child` to `node`.
                let start = path.iter().position(|m| m == child).unwrap_or(0);
                let via = path[start..].join(" -> ");
                cycles.push(DsmCycle {
                    crate_name: package.to_string(),
                    module_a: node.to_string(),
                    module_b: child.to_string(),
                    via,
                });
            } else if !visited.contains(child) {
                dfs(package, child, graph, visited, path, on_path, cycles);
            }
        }
    }

    path.pop();
    on_path.remove(node);
}

/// Detect whether cargo-modules is available on PATH.
///
/// Used by the CLI surface to skip gracefully in environments without the
/// tool installed, surfacing the three-part remediation message instead of
/// attempting a run that will fail.
pub fn cargo_modules_available() -> bool {
    match Command::new("cargo")
        .arg("modules")
        .arg("--version")
        .output()
    {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

// ── cargo metadata JSON shape ────────────────────────────────────────
//
// We only need the per-package name from `--no-deps` output.

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    #[serde(default)]
    packages: Vec<CargoPackage>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Empty cargo-modules output yields no nodes and no cycles.
    #[test]
    fn empty_modules_output_yields_empty_graph() {
        let graph = parse_modules_output("");
        assert!(graph.is_empty());
        let cycles = detect_cycles("demo", &graph);
        assert_eq!(cycles.len(), 0);
    }

    /// A simple nested tree parses into the expected adjacency map and
    /// reports no cycles.
    #[test]
    fn nested_tree_parses_without_cycles() {
        // Mirrors cargo-modules indentation; the parser ignores the exact
        // glyphs and treats leading non-letters as indent.
        let raw = "crate demo
 mod a: pub
     mod b: pub
     mod c: pub
";
        let graph = parse_modules_output(raw);
        assert!(graph.contains_key("demo"));
        assert!(graph["demo"].contains("a"));
        assert!(graph["a"].contains("b"));
        assert!(graph["a"].contains("c"));

        let cycles = detect_cycles("demo", &graph);
        assert_eq!(cycles.len(), 0);
    }

    /// A hand-built adjacency map with a back-edge yields one cycle.
    #[test]
    fn back_edge_is_detected() {
        let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        graph
            .entry("a".to_string())
            .or_default()
            .insert("b".to_string());
        graph
            .entry("b".to_string())
            .or_default()
            .insert("c".to_string());
        graph
            .entry("c".to_string())
            .or_default()
            .insert("a".to_string());

        let cycles = detect_cycles("demo", &graph);
        assert_eq!(cycles.len(), 1);
        assert_eq!(cycles[0].crate_name, "demo");
        assert_eq!(cycles[0].module_a, "c");
        assert_eq!(cycles[0].module_b, "a");
        assert!(cycles[0].via.contains("a -> b -> c"));
    }

    /// `DsmCycle` round-trips through JSON without field drift.
    #[test]
    fn cycle_roundtrips_through_json() {
        let c = DsmCycle {
            crate_name: "demo".to_string(),
            module_a: "a".to_string(),
            module_b: "b".to_string(),
            via: "a -> b".to_string(),
        };
        let rendered = serde_json::to_string(&c).unwrap();
        let back: DsmCycle = serde_json::from_str(&rendered).unwrap();
        assert_eq!(c, back);
    }

    /// `finalise_report` flips `fail` when cycles are present and
    /// `enforce_no_cycles` is true.
    #[test]
    fn finalise_report_fails_when_cycle_present_and_enforced() {
        let report = DsmReport {
            root: "/tmp".to_string(),
            duration_ms: 1,
            crates_scanned: vec!["demo".to_string()],
            cycle_count: 1,
            cycles: vec![DsmCycle {
                crate_name: "demo".to_string(),
                module_a: "a".to_string(),
                module_b: "b".to_string(),
                via: "a -> b -> a".to_string(),
            }],
            fail: false,
        };
        let finalised = finalise_report(report, true);
        assert!(finalised.fail);
    }

    /// `finalise_report` leaves `fail` false in report-only mode even
    /// with cycles present, and below threshold when no cycles exist.
    #[test]
    fn finalise_report_passes_without_enforcement_or_without_cycles() {
        let with_cycles = DsmReport {
            root: "/tmp".to_string(),
            duration_ms: 1,
            crates_scanned: vec!["demo".to_string()],
            cycle_count: 1,
            cycles: vec![DsmCycle {
                crate_name: "demo".to_string(),
                module_a: "a".to_string(),
                module_b: "b".to_string(),
                via: "a -> b".to_string(),
            }],
            fail: false,
        };
        let finalised = finalise_report(with_cycles, false);
        assert!(!finalised.fail);

        let clean = DsmReport {
            root: "/tmp".to_string(),
            duration_ms: 1,
            crates_scanned: vec!["demo".to_string()],
            cycle_count: 0,
            cycles: vec![],
            fail: false,
        };
        let finalised = finalise_report(clean, true);
        assert!(!finalised.fail);
    }
}
