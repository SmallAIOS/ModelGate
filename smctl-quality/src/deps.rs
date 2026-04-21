//! `smctl quality deps` — unused-dependency detection via `cargo machete`.
//!
//! Wraps the `cargo-machete` binary and converts its output into a crate-local
//! [`DepsReport`] struct. Emits MSGID-tagged tracing events at start, per
//! unused-dependency finding, and on completion/failure.

use std::path::Path;
use std::process::Command;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use smctl_log::MsgId;
use thiserror::Error;

/// A single unused-dependency finding in the resulting report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnusedDependency {
    /// The workspace crate that declares the unused dependency.
    pub crate_name: String,
    /// Path to the `Cargo.toml` that lists the dependency, relative to the
    /// workspace root when cargo-machete reports a path, otherwise just the
    /// bare file name.
    pub manifest: String,
    /// Name of the unused dependency as written in `Cargo.toml`.
    pub dependency: String,
}

/// Structured result of a `smctl quality deps` run.
///
/// Serialises to stable JSON. Consumers should treat the shape as a
/// public contract.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct DepsReport {
    /// Root of the workspace (or crate) that was scanned.
    pub root: String,
    /// Elapsed wall time of the deps run in milliseconds.
    pub duration_ms: u128,
    /// Number of unused-dependency findings. Same as `unused.len()`.
    pub unused_count: usize,
    /// The unused-dependency findings, one entry per `(crate, dep)` pair.
    pub unused: Vec<UnusedDependency>,
    /// True when the run found at least `fail_on_count` unused dependencies.
    /// The caller decides exit status from this.
    pub fail: bool,
}

/// Errors raised by [`run_deps`].
///
/// Every variant carries a three-part message (what happened / what it means
/// / what to do next) so that CLI callers can surface it directly without
/// rewrapping.
#[derive(Debug, Error)]
pub enum DepsError {
    #[error(
        "cargo-machete is not installed on PATH. smctl quality deps cannot run without it. Install it with: cargo install cargo-machete"
    )]
    ToolMissing,

    #[error(
        "cargo-machete exited abnormally ({status}). The deps result is unreliable. Re-run `cargo machete` directly to inspect the failure: {stderr}"
    )]
    ToolFailed { status: String, stderr: String },
}

/// Invoke `cargo machete` in `root`, parse the output, emit MSGIDs.
///
/// This is the primary public entry point of the crate's deps module. The
/// returned report is independent of CLI formatting concerns — the caller
/// renders it human-readable or as JSON.
pub fn run_deps(root: &Path) -> Result<DepsReport, DepsError> {
    let root_str = root.display().to_string();
    let started = Instant::now();

    tracing::info!(
        msgid = %MsgId::QualityCheckStarted,
        verb = "deps",
        repo = %root_str,
        scope = "workspace",
        "deps started"
    );

    let (stdout, stderr) = invoke_cargo_machete(root)?;
    let unused = parse_machete_output(&stdout, &stderr);

    for finding in &unused {
        tracing::warn!(
            msgid = %MsgId::DependencyUnused,
            crate_name = %finding.crate_name,
            manifest = %finding.manifest,
            dependency = %finding.dependency,
            "unused dependency found"
        );
    }

    let duration_ms = started.elapsed().as_millis();
    let unused_count = unused.len();

    Ok(DepsReport {
        root: root_str,
        duration_ms,
        unused_count,
        unused,
        fail: false,
    })
}

/// Decide whether the report should fail the check, given a minimum
/// unused-dependency count. The report fails when `unused_count >=
/// fail_on_count`. A `fail_on_count` of 0 disables the gate entirely
/// (useful for report-only runs).
///
/// Emits the terminal completion MSGID (either `QualityCheckCompleted` or
/// `QualityCheckFailed`). The caller should invoke this exactly once per
/// run, after [`run_deps`] returns.
pub fn finalise_report(mut report: DepsReport, fail_on_count: usize) -> DepsReport {
    let fail = fail_on_count > 0 && report.unused_count >= fail_on_count;
    report.fail = fail;

    if fail {
        let remediation = report
            .unused
            .iter()
            .map(|u| format!("remove `{}` from {}", u.dependency, u.manifest))
            .collect::<Vec<_>>()
            .join("; ");
        tracing::error!(
            msgid = %MsgId::QualityCheckFailed,
            verb = "deps",
            duration_ms = report.duration_ms as u64,
            violation_count = report.unused_count,
            remediation = %remediation,
            "deps failed"
        );
    } else {
        tracing::info!(
            msgid = %MsgId::QualityCheckCompleted,
            verb = "deps",
            duration_ms = report.duration_ms as u64,
            violation_count = report.unused_count as u64,
            "deps completed"
        );
    }

    report
}

/// Shell out to `cargo machete`. Returns `(stdout, stderr)` as UTF-8 strings
/// on success; maps ENOENT to [`DepsError::ToolMissing`] so the CLI surface
/// can render the three-part install message.
///
/// cargo-machete exits non-zero when it finds unused dependencies. That
/// condition is not an invocation failure — parse the output in both cases
/// and only raise `ToolFailed` when the exit status is non-zero AND no
/// findings were parseable from stdout/stderr.
fn invoke_cargo_machete(root: &Path) -> Result<(String, String), DepsError> {
    let output = Command::new("cargo")
        .arg("machete")
        .arg("--skip-target-dir")
        .current_dir(root)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                DepsError::ToolMissing
            } else {
                DepsError::ToolFailed {
                    status: e.kind().to_string(),
                    stderr: e.to_string(),
                }
            }
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // If cargo itself reports the subcommand is absent, surface ToolMissing
    // rather than ToolFailed so the CLI can print the install command.
    let looks_missing = stderr.contains("no such command")
        || stderr.contains("no such subcommand")
        || stderr.contains("is not installed");
    if looks_missing {
        return Err(DepsError::ToolMissing);
    }

    // cargo-machete exits non-zero when it finds unused deps. Accept that
    // case as long as there is usable output to parse. The caller handles
    // the empty-findings case by simply returning an empty report.
    if !output.status.success() && stdout.trim().is_empty() && stderr.trim().is_empty() {
        return Err(DepsError::ToolFailed {
            status: output
                .status
                .code()
                .map_or_else(|| "signal".to_string(), |c| c.to_string()),
            stderr,
        });
    }

    Ok((stdout, stderr))
}

/// Parse cargo-machete's human-readable output into a list of
/// [`UnusedDependency`] findings.
///
/// cargo-machete prints one block per manifest that has unused dependencies.
/// A block looks like:
///
/// ```text
/// cargo-machete found the following unused dependencies in /path/to/Cargo.toml:
/// smctl-build -- /path/to/smctl-build/Cargo.toml:
///         regex
///         once_cell
/// ```
///
/// We read the output line by line. A line that ends with `:` and contains
/// `--` (the separator between crate name and manifest path) starts a new
/// block. Subsequent indented lines (leading tab or spaces) are dependency
/// names. Blank lines and summary lines are skipped.
fn parse_machete_output(stdout: &str, stderr: &str) -> Vec<UnusedDependency> {
    // cargo-machete writes to both streams depending on version; scan both.
    let combined = format!("{stdout}\n{stderr}");
    let mut findings = Vec::new();
    let mut current: Option<(String, String)> = None;

    for raw_line in combined.lines() {
        let line = raw_line.trim_end();

        // Block header: `<crate> -- <path>:` with no leading whitespace.
        if !line.starts_with(char::is_whitespace)
            && line.ends_with(':')
            && line.contains("--")
            && !line.to_lowercase().starts_with("cargo-machete found")
            && !line.to_lowercase().starts_with("error")
        {
            let without_colon = line.trim_end_matches(':');
            if let Some((crate_name, manifest)) = without_colon.split_once("--") {
                current = Some((crate_name.trim().to_string(), manifest.trim().to_string()));
                continue;
            }
        }

        // Indented dependency name beneath a block header.
        if line.starts_with(char::is_whitespace) && !line.trim().is_empty() {
            if let Some((ref crate_name, ref manifest)) = current {
                let dep = line.trim().to_string();
                // cargo-machete may append notes; take just the token.
                let dep = dep.split_whitespace().next().unwrap_or("").to_string();
                if !dep.is_empty() {
                    findings.push(UnusedDependency {
                        crate_name: crate_name.clone(),
                        manifest: manifest.clone(),
                        dependency: dep,
                    });
                }
            }
            continue;
        }

        // Blank line or unrelated prose ends the current block.
        if line.trim().is_empty() {
            current = None;
        }
    }

    findings
}

/// Detect whether cargo-machete is available on PATH.
///
/// Used by the CLI surface to skip gracefully in environments without the
/// tool installed, surfacing the three-part remediation message instead of
/// attempting a run that will fail.
pub fn cargo_machete_available() -> bool {
    match Command::new("cargo")
        .arg("machete")
        .arg("--version")
        .output()
    {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Empty cargo-machete output produces a zero-finding report.
    #[test]
    fn empty_machete_output_yields_no_findings() {
        let findings = parse_machete_output("", "");
        assert_eq!(findings.len(), 0);
    }

    /// A single block with two unused deps parses into two findings.
    #[test]
    fn single_block_parses() {
        let stdout = "cargo-machete found the following unused dependencies in /tmp:
smctl-build -- /tmp/smctl-build/Cargo.toml:
\tregex
\tonce_cell
";

        let findings = parse_machete_output(stdout, "");
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].crate_name, "smctl-build");
        assert_eq!(findings[0].manifest, "/tmp/smctl-build/Cargo.toml");
        assert_eq!(findings[0].dependency, "regex");
        assert_eq!(findings[1].dependency, "once_cell");
    }

    /// Two blocks for different crates each contribute their findings.
    #[test]
    fn multiple_blocks_parse() {
        let stdout = "smctl-a -- /tmp/a/Cargo.toml:
\tfoo

smctl-b -- /tmp/b/Cargo.toml:
\tbar
\tbaz
";
        let findings = parse_machete_output(stdout, "");
        assert_eq!(findings.len(), 3);
        assert_eq!(findings[0].crate_name, "smctl-a");
        assert_eq!(findings[0].dependency, "foo");
        assert_eq!(findings[1].crate_name, "smctl-b");
        assert_eq!(findings[1].dependency, "bar");
        assert_eq!(findings[2].dependency, "baz");
    }

    /// `UnusedDependency` round-trips through JSON without field drift.
    #[test]
    fn unused_roundtrips_through_json() {
        let u = UnusedDependency {
            crate_name: "c".to_string(),
            manifest: "/tmp/c/Cargo.toml".to_string(),
            dependency: "regex".to_string(),
        };
        let rendered = serde_json::to_string(&u).unwrap();
        let back: UnusedDependency = serde_json::from_str(&rendered).unwrap();
        assert_eq!(u, back);
    }

    /// `finalise_report` flips `fail` when unused count meets the threshold.
    #[test]
    fn finalise_report_fails_at_or_above_threshold() {
        let report = DepsReport {
            root: "/tmp".to_string(),
            duration_ms: 1,
            unused_count: 1,
            unused: vec![UnusedDependency {
                crate_name: "c".to_string(),
                manifest: "/tmp/c/Cargo.toml".to_string(),
                dependency: "regex".to_string(),
            }],
            fail: false,
        };
        let finalised = finalise_report(report, 1);
        assert!(finalised.fail);
    }

    /// `finalise_report` leaves `fail` false when the report is below the
    /// threshold or when `fail_on_count` is 0.
    #[test]
    fn finalise_report_passes_below_threshold() {
        let report = DepsReport {
            root: "/tmp".to_string(),
            duration_ms: 1,
            unused_count: 0,
            unused: vec![],
            fail: false,
        };
        let finalised = finalise_report(report.clone(), 1);
        assert!(!finalised.fail);

        // fail_on_count = 0 disables the gate even when findings exist.
        let report = DepsReport {
            unused_count: 5,
            unused: (0..5)
                .map(|i| UnusedDependency {
                    crate_name: format!("c{i}"),
                    manifest: "/tmp/c/Cargo.toml".to_string(),
                    dependency: "regex".to_string(),
                })
                .collect(),
            ..report
        };
        let finalised = finalise_report(report, 0);
        assert!(!finalised.fail);
    }
}
