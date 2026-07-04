//! Lean 4 proof runner.
//!
//! Deep integration per `lean-proof-runner-v1`: each `[verify.proof]`
//! root is classified automatically — a directory carrying
//! `lakefile.lean` / `lakefile.toml` is a Lake package checked with
//! `lake build`; anything else is a loose-file tree whose `.lean`
//! files are checked one row each with `lean --json`, cwd set to the
//! root so elan's `lean-toolchain` resolution follows the corpus.
//! Classification never trusts exit codes: an admitted proof
//! (`sorry`) leaves lean's exit at 0 and still fails its row.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::lean_out::{self, LeanMessage};
use crate::shell::{anchor_override, output_head, sh_quote};
use crate::{
    DiscoveryResult, Outcome, ProofFailure, ProofFailureKind, SourceRow, Verifier, VerifyContext,
    VerifyDetail, VerifyReport,
};

/// Overrides the lean binary wholesale. Test-and-escape-hatch only.
pub const ENV_LEAN_BIN: &str = "SMCTL_VERIFY_LEAN_BIN";
/// Overrides the lake binary wholesale. Test-and-escape-hatch only.
pub const ENV_LAKE_BIN: &str = "SMCTL_VERIFY_LAKE_BIN";

const INSTALL_HINT: &str =
    "install Lean 4 via elan: https://leanprover.github.io/lean4/doc/setup.html";

#[derive(Debug, Default)]
pub struct LeanVerifier;

impl LeanVerifier {
    pub fn new() -> Self {
        Self
    }
}

fn resolve_tool(env_var: &str, default: &str) -> String {
    anchor_override(
        std::env::var(env_var)
            .ok()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| default.to_string()),
    )
}

/// Version-probe a tool, requiring a successful exit. elan installs
/// PATH shims that spawn fine but exit non-zero when no toolchain is
/// configured — a mere spawn success is not evidence the tool works
/// (same hazard as macOS's `/usr/bin/cc` shim, see `spin::cc_available`).
fn probe(bin: &str) -> Option<String> {
    let out = Command::new(bin).arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let mut combined = Vec::with_capacity(out.stdout.len() + out.stderr.len());
    combined.extend_from_slice(&out.stdout);
    combined.extend_from_slice(&out.stderr);
    let banner = String::from_utf8_lossy(&combined);
    Some(
        banner
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim()
            .to_string(),
    )
}

impl Verifier for LeanVerifier {
    fn name(&self) -> &'static str {
        "proof"
    }

    /// Probes `lean` — the foundational checker; lake and lean ship
    /// together in every elan toolchain, so one probe answers the
    /// install question for both.
    fn discover(&self) -> DiscoveryResult {
        let bin = resolve_tool(ENV_LEAN_BIN, "lean");
        match probe(&bin) {
            Some(version) => DiscoveryResult::Found { path: bin, version },
            None => DiscoveryResult::NotInstalled {
                tool: "lean".to_string(),
                install_hint: INSTALL_HINT.to_string(),
            },
        }
    }

    fn run(&self, ctx: &VerifyContext) -> VerifyReport {
        // An unconfigured workspace reports `no sources configured`
        // without probing tools (same rationale as the SPIN runner).
        if ctx.manifest.sources.is_empty() {
            return VerifyReport::empty("proof", Outcome::NoSources);
        }

        let mut report = VerifyReport::empty("proof", Outcome::NoSources);
        let targets = collect_targets(ctx, &mut report.diagnostics);
        let glob_failed = !report.diagnostics.is_empty();
        if targets.is_empty() {
            if glob_failed {
                report.outcome = Outcome::Failed;
            }
            return report;
        }

        let needs_lean = targets
            .iter()
            .any(|t| matches!(t, Target::LooseFile { .. }));
        let needs_lake = targets
            .iter()
            .any(|t| matches!(t, Target::LakePackage { .. }));
        let lean_bin = resolve_tool(ENV_LEAN_BIN, "lean");
        let lake_bin = resolve_tool(ENV_LAKE_BIN, "lake");
        if let Some((tool, hint)) = missing_tool(needs_lean, needs_lake, &lean_bin, &lake_bin) {
            let mut missing = VerifyReport::empty("proof", Outcome::ToolMissing);
            missing
                .diagnostics
                .push(format!("{tool} is not installed on PATH. {hint}"));
            return missing;
        }

        let mut any_failed = glob_failed;
        let mut excerpts: Vec<String> = Vec::new();
        for target in &targets {
            let (row, excerpt) = match target {
                Target::LooseFile { root, file } => run_loose_file(&lean_bin, root, file),
                Target::LakePackage { root } => run_lake_package(&lake_bin, root),
            };
            if matches!(row.outcome, Outcome::Failed) {
                any_failed = true;
                report.diagnostics.push(row.note.clone());
            }
            if let Some(e) = excerpt {
                excerpts.push(e);
            }
            report.sources.push(row);
        }
        report.diagnostics.extend(excerpts);
        report.outcome = if any_failed {
            Outcome::Failed
        } else {
            Outcome::Passed
        };
        report
    }
}

/// Which tool the missing-tool envelope should name for the proof
/// verb right now: classification decides whether `lean` (loose-file
/// trees) or `lake` (Lake packages) is actually required. The CLI
/// consults this instead of `discover()` when building the
/// `tool_missing` envelope, mirroring `spin::missing_tool_for_protocol`.
pub fn missing_tool_for_proof(ctx: &VerifyContext) -> Option<(String, String)> {
    let mut sink = Vec::new();
    let targets = collect_targets(ctx, &mut sink);
    let needs_lean = targets.is_empty()
        || targets
            .iter()
            .any(|t| matches!(t, Target::LooseFile { .. }));
    let needs_lake = targets
        .iter()
        .any(|t| matches!(t, Target::LakePackage { .. }));
    let lean_bin = resolve_tool(ENV_LEAN_BIN, "lean");
    let lake_bin = resolve_tool(ENV_LAKE_BIN, "lake");
    missing_tool(needs_lean, needs_lake, &lean_bin, &lake_bin)
}

fn missing_tool(
    needs_lean: bool,
    needs_lake: bool,
    lean_bin: &str,
    lake_bin: &str,
) -> Option<(String, String)> {
    if needs_lean && probe(lean_bin).is_none() {
        return Some(("lean".to_string(), INSTALL_HINT.to_string()));
    }
    if needs_lake && probe(lake_bin).is_none() {
        return Some(("lake".to_string(), INSTALL_HINT.to_string()));
    }
    None
}

/// One unit of proof-checking work after root classification.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Target {
    /// A single `.lean` file checked with `lean --json`, cwd `root`.
    LooseFile { root: PathBuf, file: PathBuf },
    /// A Lake package built with `lake build`, cwd `root`.
    LakePackage { root: PathBuf },
}

/// Expand the manifest's roots across every registered repo into
/// classified targets. Mirrors `shell::walk_sources`' glob semantics
/// (including its diagnostics for bad patterns) but flat-maps
/// directory matches: a Lake package is one target, a loose tree one
/// target per discovered `.lean` file.
fn collect_targets(ctx: &VerifyContext, diagnostics: &mut Vec<String>) -> Vec<Target> {
    let mut targets = Vec::new();
    for (repo_name, repo_path) in &ctx.repos {
        for pattern in &ctx.manifest.sources {
            let absolute = repo_path.join(pattern);
            let glob_pattern = match absolute.to_str() {
                Some(s) => s.to_string(),
                None => {
                    diagnostics.push(format!(
                        "skipped non-utf8 glob pattern under repo {repo_name}: {pattern}",
                    ));
                    continue;
                }
            };
            let entries = match glob::glob(&glob_pattern) {
                Ok(it) => it,
                Err(e) => {
                    diagnostics.push(format!(
                        "invalid glob '{pattern}' under repo {repo_name}: {e}. Fix the pattern in [verify.proof].roots and re-run.",
                    ));
                    continue;
                }
            };
            for entry in entries {
                let path = match entry {
                    Ok(p) => p,
                    Err(e) => {
                        diagnostics
                            .push(format!("could not read entry under repo {repo_name}: {e}.",));
                        continue;
                    }
                };
                classify(&path, &mut targets);
            }
        }
    }
    targets
}

fn classify(path: &Path, targets: &mut Vec<Target>) {
    if path.is_dir() {
        if path.join("lakefile.lean").is_file() || path.join("lakefile.toml").is_file() {
            targets.push(Target::LakePackage {
                root: path.to_path_buf(),
            });
            return;
        }
        let mut files = Vec::new();
        collect_lean_files(path, &mut files);
        files.sort();
        targets.extend(files.into_iter().map(|file| Target::LooseFile {
            root: path.to_path_buf(),
            file,
        }));
        return;
    }
    // A glob that matches files directly yields loose rows as-is; the
    // operator's pattern is authoritative.
    if path.is_file() {
        let root = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        targets.push(Target::LooseFile {
            root,
            file: path.to_path_buf(),
        });
    }
}

/// Recursively gather `.lean` files, skipping hidden entries — which
/// also covers Lake's `.lake/` build directory.
fn collect_lean_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_lean_files(&path, out);
        } else if path.extension().is_some_and(|x| x == "lean") {
            out.push(path);
        }
    }
}

/// Check one loose `.lean` file with `lean --json`.
fn run_loose_file(lean_bin: &str, root: &Path, file: &Path) -> (SourceRow, Option<String>) {
    let display = file.display().to_string();
    let abs_file = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
    let rel = file
        .strip_prefix(root)
        .unwrap_or(file)
        .display()
        .to_string();
    let reproduce = format!(
        "cd {root} && {lean} {rel}",
        root = sh_quote(&root.display().to_string()),
        lean = sh_quote(lean_bin),
        rel = sh_quote(&rel),
    );

    let out = match Command::new(lean_bin)
        .arg("--json")
        .arg(&abs_file)
        .current_dir(root)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return (
                SourceRow::plain(
                    display.clone(),
                    Outcome::Failed,
                    format!("could not spawn lean for {display}: {e}. {INSTALL_HINT}."),
                ),
                None,
            );
        }
    };

    // Messages arrive on stdout, one JSON object per line; stderr
    // carries elan noise (toolchain downloads) and is never parsed
    // as messages.
    let messages = lean_out::parse_lean_json(&String::from_utf8_lossy(&out.stdout));
    let detail = lean_out::summarize(&messages);
    finish_row(
        display,
        detail,
        &messages,
        out.status.success(),
        &output_head(&out.stdout, &out.stderr),
        out.status.code(),
        &reproduce,
        "lean",
    )
}

/// Build one Lake package with `lake build`, parsing replayed
/// compiler messages out of the text log.
fn run_lake_package(lake_bin: &str, root: &Path) -> (SourceRow, Option<String>) {
    let display = root.display().to_string();
    let reproduce = format!(
        "cd {root} && {lake} build",
        root = sh_quote(&display),
        lake = sh_quote(lake_bin),
    );

    let out = match Command::new(lake_bin)
        .arg("build")
        .current_dir(root)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return (
                SourceRow::plain(
                    display.clone(),
                    Outcome::Failed,
                    format!("could not spawn lake for {display}: {e}. {INSTALL_HINT}."),
                ),
                None,
            );
        }
    };

    // Lake replays compiler messages on stdout; its own `error: build
    // failed` marker lands on stderr. Parse both so a lakefile-level
    // failure still yields a message.
    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push('\n');
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    let messages = lean_out::parse_lake_log(&text);
    let detail = lean_out::summarize(&messages);
    finish_row(
        display,
        detail,
        &messages,
        out.status.success(),
        &output_head(&out.stdout, &out.stderr),
        out.status.code(),
        &reproduce,
        "lake",
    )
}

/// Shared verdict mapping: message evidence first, exit code last.
#[allow(clippy::too_many_arguments)]
fn finish_row(
    source_display: String,
    detail: crate::ProofCheckDetail,
    messages: &[LeanMessage],
    exit_ok: bool,
    head: &str,
    exit_code: Option<i32>,
    reproduce: &str,
    tool: &str,
) -> (SourceRow, Option<String>) {
    match &detail.failure {
        Some(f) if f.kind == ProofFailureKind::Error => {
            tracing::error!(
                msgid = %smctl_log::MsgId::VerifyCounterExample,
                source = %source_display,
                errors = detail.errors,
                location = f.location.as_deref().unwrap_or(""),
                "proof error",
            );
            let loc_part = f
                .location
                .as_deref()
                .map(|l| format!(" at {l}"))
                .unwrap_or_default();
            let note = format!(
                "{msg}{loc_part} ({errs} error(s) in {source_display}). The proof does not check — {tool} rejected it. Re-run `{reproduce}` and fix the first error.",
                msg = f.message,
                errs = detail.errors,
            );
            let excerpt = first_matching(messages, |m| m.severity == lean_out::LeanSeverity::Error)
                .map(|m| lean_out::render_message_excerpt(&source_display, m, reproduce));
            (failed_row(source_display, note, detail), excerpt)
        }
        Some(f) if f.kind == ProofFailureKind::Sorry => {
            tracing::error!(
                msgid = %smctl_log::MsgId::ProofIncomplete,
                source = %source_display,
                sorries = detail.sorries,
                location = f.location.as_deref().unwrap_or(""),
                "proof admitted via sorry",
            );
            let loc_part = f
                .location
                .as_deref()
                .map(|l| format!(" at {l}"))
                .unwrap_or_default();
            let note = format!(
                "proof admitted via sorry{loc_part} ({n} marker(s) in {source_display}). The theorem is accepted on trust, not proved — sorry introduces the sorryAx axiom. Complete the proof, then re-run `{reproduce}` to confirm.",
                n = detail.sorries,
            );
            let excerpt = first_matching(messages, lean_out::is_sorry)
                .map(|m| lean_out::render_message_excerpt(&source_display, m, reproduce));
            (failed_row(source_display, note, detail), excerpt)
        }
        _ if exit_ok => {
            let note = if detail.warnings > 0 {
                format!("{} warning(s)", detail.warnings)
            } else {
                String::new()
            };
            (
                SourceRow {
                    source: source_display,
                    outcome: Outcome::Passed,
                    note,
                    detail: Some(VerifyDetail::Proof(detail)),
                },
                None,
            )
        }
        _ => {
            // Non-zero exit with no message evidence: the failure is
            // real (exit code is ground truth) but unclassifiable.
            tracing::warn!(
                msgid = %smctl_log::MsgId::VerifyOutputUnparsed,
                source = %source_display,
                exit = exit_code.unwrap_or(-1),
                "proof tool output did not match any known pattern",
            );
            let head_part = if head.is_empty() {
                String::new()
            } else {
                format!(" (output: {head})")
            };
            let note = format!(
                "{tool} exited with {code} on {source_display} but reported no parseable message{head_part}. The failure is likely environmental (lakefile error, missing import, toolchain fault). Re-run `{reproduce}` to inspect the full output.",
                code = exit_code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "<signal>".into()),
            );
            let mut detail = detail;
            detail.failure = Some(ProofFailure {
                kind: ProofFailureKind::Build,
                location: None,
                message: head.to_string(),
            });
            (failed_row(source_display, note, detail), None)
        }
    }
}

fn first_matching(
    messages: &[LeanMessage],
    pred: impl Fn(&LeanMessage) -> bool,
) -> Option<&LeanMessage> {
    messages.iter().find(|m| pred(m))
}

fn failed_row(display: String, note: String, detail: crate::ProofCheckDetail) -> SourceRow {
    SourceRow {
        source: display,
        outcome: Outcome::Failed,
        note,
        detail: Some(VerifyDetail::Proof(detail)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(path: &Path) {
        std::fs::write(path, "-- lean source\n").unwrap();
    }

    #[test]
    fn classify_detects_lake_package_by_lakefile() {
        for lakefile in ["lakefile.lean", "lakefile.toml"] {
            let dir = tempfile::tempdir().unwrap();
            std::fs::write(dir.path().join(lakefile), "").unwrap();
            touch(&dir.path().join("Main.lean"));
            let mut targets = Vec::new();
            classify(dir.path(), &mut targets);
            assert_eq!(
                targets,
                vec![Target::LakePackage {
                    root: dir.path().to_path_buf()
                }],
                "{lakefile} must mark a Lake package"
            );
        }
    }

    #[test]
    fn classify_expands_loose_tree_per_file_skipping_hidden() {
        let dir = tempfile::tempdir().unwrap();
        touch(&dir.path().join("B.lean"));
        std::fs::create_dir(dir.path().join("nested")).unwrap();
        touch(&dir.path().join("nested/A.lean"));
        std::fs::write(dir.path().join("notes.md"), "").unwrap();
        // `.lake` build dir and other hidden entries must be skipped.
        std::fs::create_dir(dir.path().join(".lake")).unwrap();
        touch(&dir.path().join(".lake/Decoy.lean"));

        let mut targets = Vec::new();
        classify(dir.path(), &mut targets);
        let files: Vec<String> = targets
            .iter()
            .map(|t| match t {
                Target::LooseFile { file, .. } => {
                    file.strip_prefix(dir.path()).unwrap().display().to_string()
                }
                other => panic!("expected loose files, got {other:?}"),
            })
            .collect();
        assert_eq!(files, vec!["B.lean", "nested/A.lean"]);
    }

    #[test]
    fn classify_passes_direct_file_matches_through() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join("Single.lean");
        touch(&f);
        let mut targets = Vec::new();
        classify(&f, &mut targets);
        assert_eq!(
            targets,
            vec![Target::LooseFile {
                root: dir.path().to_path_buf(),
                file: f
            }]
        );
    }

    #[test]
    fn probe_rejects_binaries_that_exit_nonzero() {
        // /bin/sh spawns fine; `sh --version -c 'exit 1'` … we need a
        // binary that exits non-zero from `--version`: a shim script.
        let dir = tempfile::tempdir().unwrap();
        let shim = dir.path().join("broken-lean");
        std::fs::write(
            &shim,
            "#!/bin/sh\necho 'error: no default toolchain' >&2\nexit 1\n",
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&shim, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert_eq!(probe(&shim.display().to_string()), None);
    }

    #[test]
    fn probe_accepts_healthy_binaries() {
        let dir = tempfile::tempdir().unwrap();
        let ok = dir.path().join("ok-lean");
        std::fs::write(
            &ok,
            "#!/bin/sh\necho 'Lean (version 4.15.0, x86_64, commit abc, Release)'\n",
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&ok, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let version = probe(&ok.display().to_string()).expect("healthy probe must succeed");
        assert!(version.contains("4.15.0"), "{version}");
    }
}
