//! Shared shell-out helpers for the TLA+ / Lean 4 / SPIN verifiers.
//!
//! These wrappers are deliberately thin per `formal-methods-v1`'s
//! Out-of-Scope clause: each tool is invoked, its exit code is read,
//! and a `SourceRow` is produced. Deep output parsing (counter-example
//! rendering, proof-progress streaming) belongs in the per-tool
//! follow-up changes.

use std::path::Path;
use std::process::Command;

use glob::glob;

use crate::{DiscoveryResult, Outcome, SourceRow, VerifyContext, VerifyReport};

/// Configuration for a shell-out verifier. Each per-tool module
/// constructs one of these and hands it to [`discover_binary`] /
/// [`run_against_sources`].
pub struct Shell<'a> {
    /// Verifier name reported in [`Verifier::name`](super::Verifier::name).
    pub name: &'a str,
    /// Default binary to run (e.g. `"tlc"`, `"lake"`, `"spin"`).
    /// Tests can pass a path to a stub.
    pub binary: &'a str,
    /// Args that print a version banner. Used by `discover_binary`.
    pub version_args: &'a [&'a str],
    /// Args used during a real run, formatted with the per-source
    /// path appended at the end.
    pub run_args: &'a [&'a str],
    /// Install hint surfaced when the binary isn't on PATH.
    pub install_hint: &'a str,
}

/// Probe whether `binary` is reachable. Spawns it with
/// `version_args` and treats a successful exit as Found. The version
/// string is the first non-empty line of stdout/stderr (best-effort).
pub fn discover_binary(shell: &Shell<'_>) -> DiscoveryResult {
    let mut cmd = Command::new(shell.binary);
    cmd.args(shell.version_args);
    match cmd.output() {
        Ok(out) => {
            // Some tools print to stderr; combine.
            let mut combined = Vec::with_capacity(out.stdout.len() + out.stderr.len());
            combined.extend_from_slice(&out.stdout);
            combined.extend_from_slice(&out.stderr);
            let banner = String::from_utf8_lossy(&combined);
            let version = banner
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("")
                .trim()
                .to_string();
            DiscoveryResult::Found {
                path: shell.binary.to_string(),
                version,
            }
        }
        Err(_) => DiscoveryResult::NotInstalled {
            tool: shell.binary.to_string(),
            install_hint: shell.install_hint.to_string(),
        },
    }
}

/// Walk the manifest's source patterns, run the binary against each
/// match, and return a fully populated [`VerifyReport`]. Returns a
/// `ToolMissing` report (with no sources) when discovery fails.
pub fn run_against_sources(shell: &Shell<'_>, ctx: &VerifyContext) -> VerifyReport {
    let mut report = VerifyReport::empty(shell.name, Outcome::NoSources);

    if !discover_binary(shell).is_installed() {
        let mut missing = VerifyReport::empty(shell.name, Outcome::ToolMissing);
        missing.diagnostics.push(format!(
            "{} is not installed on PATH. {}",
            shell.binary, shell.install_hint
        ));
        return missing;
    }

    if ctx.manifest.sources.is_empty() {
        return report;
    }

    let mut any_failed = false;
    let mut any_processed = false;

    for (repo_name, repo_path) in &ctx.repos {
        for pattern in &ctx.manifest.sources {
            let absolute = repo_path.join(pattern);
            let glob_pattern = match absolute.to_str() {
                Some(s) => s.to_string(),
                None => {
                    report.diagnostics.push(format!(
                        "skipped non-utf8 glob pattern under repo {repo_name}: {pattern}",
                    ));
                    continue;
                }
            };
            let entries = match glob(&glob_pattern) {
                Ok(it) => it,
                Err(e) => {
                    report.diagnostics.push(format!(
                        "invalid glob '{pattern}' under repo {repo_name}: {e}. Fix the pattern in [verify.{}].sources / specs / roots and re-run.",
                        shell.name,
                    ));
                    any_failed = true;
                    continue;
                }
            };
            for entry in entries {
                let path = match entry {
                    Ok(p) => p,
                    Err(e) => {
                        report
                            .diagnostics
                            .push(format!("could not read entry under repo {repo_name}: {e}.",));
                        any_failed = true;
                        continue;
                    }
                };
                any_processed = true;
                let row = run_one_source(shell, &path);
                if matches!(row.outcome, Outcome::Failed) {
                    any_failed = true;
                    report.diagnostics.push(row.note.clone());
                }
                report.sources.push(row);
            }
        }
    }

    report.outcome = match (any_processed, any_failed) {
        (false, _) => Outcome::NoSources,
        (true, true) => Outcome::Failed,
        (true, false) => Outcome::Passed,
    };
    report
}

fn run_one_source(shell: &Shell<'_>, source: &Path) -> SourceRow {
    let mut cmd = Command::new(shell.binary);
    cmd.args(shell.run_args).arg(source);
    let display = source.display().to_string();
    match cmd.status() {
        Ok(status) if status.success() => SourceRow {
            source: display,
            outcome: Outcome::Passed,
            note: String::new(),
        },
        Ok(status) => SourceRow {
            source: display.clone(),
            outcome: Outcome::Failed,
            note: format!(
                "{} exited with {} on {display}. Re-run `{} {} {display}` to see the tool's full output, then fix the reported issue.",
                shell.binary,
                status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "<signal>".into()),
                shell.binary,
                shell.run_args.join(" "),
            ),
        },
        Err(e) => SourceRow {
            source: display.clone(),
            outcome: Outcome::Failed,
            note: format!(
                "could not spawn {} for {display}: {e}. {}",
                shell.binary, shell.install_hint
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use crate::VerifyManifest;

    fn ctx_with_no_sources() -> VerifyContext {
        VerifyContext {
            workspace_root: PathBuf::from("/tmp"),
            repos: BTreeMap::new(),
            manifest: VerifyManifest::default(),
            strict: false,
            verifier_filter: None,
        }
    }

    /// `/bin/sh` is reliably present on macOS + Linux runners. Use it
    /// as a stand-in tool for the discovery-Found path. We accept that
    /// the version banner is whatever `sh --version` produces (or
    /// nothing — sh on macOS doesn't recognise --version).
    #[test]
    fn discover_finds_a_real_binary() {
        let shell = Shell {
            name: "test",
            binary: "/bin/sh",
            version_args: &["-c", "echo sh-fake-1.0"],
            run_args: &[],
            install_hint: "irrelevant for this test",
        };
        let d = discover_binary(&shell);
        assert!(d.is_installed(), "/bin/sh should be discoverable");
        match d {
            DiscoveryResult::Found { version, .. } => {
                assert!(version.contains("sh-fake-1.0"), "version: {version}");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn discover_missing_returns_not_installed() {
        let shell = Shell {
            name: "test",
            binary: "smctl-verify-not-a-real-binary-xyzzy",
            version_args: &["--version"],
            run_args: &[],
            install_hint: "install via cargo install xyzzy",
        };
        let d = discover_binary(&shell);
        assert!(!d.is_installed());
        match d {
            DiscoveryResult::NotInstalled { tool, install_hint } => {
                assert!(tool.contains("xyzzy"));
                assert!(install_hint.contains("cargo install"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn run_returns_tool_missing_when_binary_absent() {
        let shell = Shell {
            name: "test",
            binary: "smctl-verify-not-a-real-binary-xyzzy",
            version_args: &["--version"],
            run_args: &[],
            install_hint: "install via cargo install xyzzy",
        };
        let report = run_against_sources(&shell, &ctx_with_no_sources());
        assert_eq!(report.outcome, Outcome::ToolMissing);
        assert!(report.diagnostics.iter().any(|d| d.contains("xyzzy")));
    }

    #[test]
    fn run_returns_no_sources_when_manifest_empty() {
        let shell = Shell {
            name: "test",
            binary: "/bin/sh",
            version_args: &["-c", "echo ok"],
            run_args: &["-c", ":"],
            install_hint: "irrelevant",
        };
        let report = run_against_sources(&shell, &ctx_with_no_sources());
        assert_eq!(report.outcome, Outcome::NoSources);
    }
}
