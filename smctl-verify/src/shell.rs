//! Shared shell-out helpers for the TLA+ / Lean 4 / SPIN verifiers.
//!
//! The Lean and SPIN wrappers remain exit-code level per
//! `formal-methods-v1`'s Out-of-Scope clause; deep output parsing for
//! them belongs in `lean-proof-runner-v1` / `spin-protocol-runner-v1`.
//! The TLA+ runner drives [`walk_sources`] with its own per-source
//! closure (see `tla.rs`). Child stdout/stderr is always captured —
//! never inherited — so tool output cannot interleave with smctl's
//! own stdout and corrupt the piped-JSON contract.

use std::path::Path;
use std::process::Command;

use glob::glob;

use crate::{DiscoveryResult, Outcome, SourceRow, VerifyContext, VerifyReport};

/// How many leading lines of captured tool output are folded into a
/// failure note before truncation.
const CAPTURED_NOTE_LINES: usize = 4;

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
    /// Environment variable that overrides the binary path when set
    /// (e.g. `SMCTL_VERIFY_TLC_BIN`). Test-and-escape-hatch only.
    pub env_override: Option<&'a str>,
}

/// Resolve the binary to spawn, honoring the env override when set.
pub fn resolve_binary(shell: &Shell<'_>) -> String {
    shell
        .env_override
        .and_then(|var| std::env::var(var).ok())
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| shell.binary.to_string())
}

/// Probe whether `binary` is reachable. Spawns it with
/// `version_args` and treats a successful exit as Found. The version
/// string is the first non-empty line of stdout/stderr (best-effort).
pub fn discover_binary(shell: &Shell<'_>) -> DiscoveryResult {
    let binary = resolve_binary(shell);
    let mut cmd = Command::new(&binary);
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
                path: binary,
                version,
            }
        }
        Err(_) => DiscoveryResult::NotInstalled {
            tool: shell.binary.to_string(),
            install_hint: shell.install_hint.to_string(),
        },
    }
}

/// Walk the manifest's source patterns across every registered repo,
/// producing one [`SourceRow`] per match via `run_one`, and aggregate
/// into a [`VerifyReport`]. Glob errors become diagnostics; failed
/// rows have their notes mirrored into the report diagnostics.
pub fn walk_sources(
    name: &str,
    ctx: &VerifyContext,
    run_one: &dyn Fn(&Path) -> SourceRow,
) -> VerifyReport {
    let mut report = VerifyReport::empty(name, Outcome::NoSources);

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
                        "invalid glob '{pattern}' under repo {repo_name}: {e}. Fix the pattern in [verify.{name}].sources / specs / roots and re-run.",
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
                let row = run_one(&path);
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

/// Generic shell-out run: discovery gate, then [`walk_sources`] with
/// the exit-code-level [`run_one_source`]. Lean and SPIN use this
/// path unchanged.
pub fn run_against_sources(shell: &Shell<'_>, ctx: &VerifyContext) -> VerifyReport {
    if !discover_binary(shell).is_installed() {
        let mut missing = VerifyReport::empty(shell.name, Outcome::ToolMissing);
        missing.diagnostics.push(format!(
            "{} is not installed on PATH. {}",
            shell.binary, shell.install_hint
        ));
        return missing;
    }

    walk_sources(shell.name, ctx, &|path| run_one_source(shell, path))
}

/// First lines of captured tool output, for folding into a failure
/// note without flooding the report. stderr is preferred (most tools
/// put the actionable error there), falling back to stdout.
pub fn output_head(stdout: &[u8], stderr: &[u8]) -> String {
    let pick = if stderr.iter().any(|b| !b.is_ascii_whitespace()) {
        stderr
    } else {
        stdout
    };
    let text = String::from_utf8_lossy(pick);
    let lines: Vec<&str> = text
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(CAPTURED_NOTE_LINES)
        .collect();
    lines.join(" | ")
}

fn run_one_source(shell: &Shell<'_>, source: &Path) -> SourceRow {
    let binary = resolve_binary(shell);
    let mut cmd = Command::new(&binary);
    cmd.args(shell.run_args).arg(source);
    let display = source.display().to_string();
    match cmd.output() {
        Ok(out) if out.status.success() => {
            SourceRow::plain(display, Outcome::Passed, String::new())
        }
        Ok(out) => {
            let head = output_head(&out.stdout, &out.stderr);
            let head_part = if head.is_empty() {
                String::new()
            } else {
                format!(" Tool output: {head}.")
            };
            SourceRow::plain(
                display.clone(),
                Outcome::Failed,
                format!(
                    "{} exited with {} on {display}.{head_part} Re-run `{} {} {display}` to see the tool's full output, then fix the reported issue.",
                    shell.binary,
                    out.status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "<signal>".into()),
                    shell.binary,
                    shell.run_args.join(" "),
                ),
            )
        }
        Err(e) => SourceRow::plain(
            display.clone(),
            Outcome::Failed,
            format!(
                "could not spawn {} for {display}: {e}. {}",
                shell.binary, shell.install_hint
            ),
        ),
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
            env_override: None,
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
            env_override: None,
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
    fn env_override_redirects_discovery() {
        // Deliberately unique var name to avoid cross-test interference.
        let var = "SMCTL_VERIFY_TEST_OVERRIDE_DISCOVERY";
        // SAFETY (test-only): no other thread in this test binary
        // reads this uniquely named variable concurrently.
        unsafe { std::env::set_var(var, "/bin/sh") };
        let shell = Shell {
            name: "test",
            binary: "smctl-verify-not-a-real-binary-xyzzy",
            version_args: &["-c", "echo overridden-9.9"],
            run_args: &[],
            install_hint: "irrelevant",
            env_override: Some(var),
        };
        let d = discover_binary(&shell);
        unsafe { std::env::remove_var(var) };
        match d {
            DiscoveryResult::Found { path, version } => {
                assert_eq!(path, "/bin/sh");
                assert!(version.contains("overridden-9.9"), "version: {version}");
            }
            other => panic!("expected Found via override, got {other:?}"),
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
            env_override: None,
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
            env_override: None,
        };
        let report = run_against_sources(&shell, &ctx_with_no_sources());
        assert_eq!(report.outcome, Outcome::NoSources);
    }

    #[test]
    fn failure_note_folds_captured_output_head() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("input.txt");
        std::fs::write(&src, "x").unwrap();
        let shell = Shell {
            name: "test",
            binary: "/bin/sh",
            version_args: &["-c", "echo ok"],
            // Print a recognizable error and fail, ignoring the
            // appended source path ("$0" consumes it).
            run_args: &["-c", "echo boom-diagnostic-line >&2; exit 3"],
            install_hint: "irrelevant",
            env_override: None,
        };
        let mut ctx = ctx_with_no_sources();
        ctx.repos.insert(".".into(), dir.path().to_path_buf());
        ctx.manifest = VerifyManifest {
            sources: vec!["input.txt".into()],
            ..VerifyManifest::default()
        };
        let report = run_against_sources(&shell, &ctx);
        assert_eq!(report.outcome, Outcome::Failed);
        let row = &report.sources[0];
        assert!(
            row.note.contains("boom-diagnostic-line"),
            "captured output head missing from note: {}",
            row.note
        );
    }

    #[test]
    fn output_head_prefers_stderr_and_caps_lines() {
        let head = output_head(b"out1\nout2\n", b"err1\n\nerr2\nerr3\nerr4\nerr5\n");
        assert_eq!(head, "err1 | err2 | err3 | err4");
        let head = output_head(b"only-stdout\n", b"   \n");
        assert_eq!(head, "only-stdout");
    }
}
