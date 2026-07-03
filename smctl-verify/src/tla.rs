//! TLA+ model-checker runner.
//!
//! Deep integration per `tla-plus-runner-v1`: resolves TLC via PATH
//! binary or `java -jar tla2tools.jar`, invokes it with the sibling
//! `.cfg` convention honored and `-metadir` pointed at a temp
//! directory, captures output, and parses statistics, violations,
//! and counter-example traces (see `tlc.rs`).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::shell::output_head;
use crate::tlc::{self, TlcVerdict};
use crate::{
    DiscoveryResult, Outcome, SourceRow, Verifier, VerifyContext, VerifyManifest, VerifyReport,
    Violation,
};

/// Overrides the TLC binary wholesale. Test-and-escape-hatch only.
pub const ENV_TLC_BIN: &str = "SMCTL_VERIFY_TLC_BIN";
/// Points at a `tla2tools.jar` for the `java -jar` fallback.
pub const ENV_TLA2TOOLS_JAR: &str = "TLA2TOOLS_JAR";

const INSTALL_HINT: &str = "verify model needs TLC. Install `tlc` on PATH, set TLA2TOOLS_JAR to a tla2tools.jar, or declare `jar = \"path/to/tla2tools.jar\"` under [verify.model] in workspace.toml";

/// How TLC gets launched once resolved.
#[derive(Debug, Clone)]
enum Launcher {
    /// A `tlc` binary (PATH or env override).
    Direct(String),
    /// `java -jar <tla2tools.jar>`.
    Jar { jar: PathBuf },
}

impl Launcher {
    fn command(&self) -> Command {
        match self {
            Launcher::Direct(bin) => Command::new(bin),
            Launcher::Jar { jar } => {
                let mut cmd = Command::new("java");
                cmd.arg("-XX:+UseParallelGC").arg("-jar").arg(jar);
                cmd
            }
        }
    }

    /// Human-readable command prefix for reproduce hints.
    fn display(&self) -> String {
        match self {
            Launcher::Direct(bin) => bin.clone(),
            Launcher::Jar { jar } => format!("java -jar {}", jar.display()),
        }
    }
}

fn spawns_ok(binary: &str, args: &[&str]) -> bool {
    Command::new(binary).args(args).output().is_ok()
}

fn banner_first_line(binary: &str, args: &[&str]) -> String {
    Command::new(binary)
        .args(args)
        .output()
        .map(|out| {
            let mut combined = out.stdout;
            combined.extend_from_slice(&out.stderr);
            String::from_utf8_lossy(&combined)
                .lines()
                .find(|l| !l.trim().is_empty())
                .unwrap_or("")
                .trim()
                .to_string()
        })
        .unwrap_or_default()
}

/// Resolve a TLC launcher. Chain: env override → PATH `tlc` →
/// `[verify.model] jar` (relative to the workspace root) →
/// `TLA2TOOLS_JAR` — the jar paths require a working `java`.
/// `manifest` is `None` in [`Verifier::discover`], which has no
/// context; the run path passes the real manifest.
fn resolve_launcher(
    workspace_root: Option<&Path>,
    manifest: Option<&VerifyManifest>,
) -> Option<(Launcher, String)> {
    if let Ok(bin) = std::env::var(ENV_TLC_BIN)
        && !bin.trim().is_empty()
    {
        if spawns_ok(&bin, &["-h"]) {
            let version = banner_first_line(&bin, &["-h"]);
            return Some((Launcher::Direct(bin), version));
        }
        return None;
    }

    if spawns_ok("tlc", &["-h"]) {
        let version = banner_first_line("tlc", &["-h"]);
        return Some((Launcher::Direct("tlc".into()), version));
    }

    let configured_jar = manifest.and_then(|m| m.jar.as_ref()).map(|j| {
        let p = PathBuf::from(j);
        if p.is_absolute() {
            p
        } else {
            workspace_root.map(|r| r.join(&p)).unwrap_or(p)
        }
    });
    let env_jar = std::env::var(ENV_TLA2TOOLS_JAR)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from);

    for jar in [configured_jar, env_jar].into_iter().flatten() {
        if jar.is_file() && spawns_ok("java", &["-version"]) {
            let version = format!("tla2tools at {}", jar.display());
            return Some((Launcher::Jar { jar }, version));
        }
    }

    None
}

#[derive(Debug, Default)]
pub struct TlaVerifier;

impl TlaVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Verifier for TlaVerifier {
    fn name(&self) -> &'static str {
        "model"
    }

    /// Context-free probe: env override → PATH `tlc` →
    /// `TLA2TOOLS_JAR`. The `[verify.model] jar` field is only
    /// visible to [`Verifier::run`], which re-resolves with it.
    fn discover(&self) -> DiscoveryResult {
        match resolve_launcher(None, None) {
            Some((launcher, version)) => DiscoveryResult::Found {
                path: launcher.display(),
                version,
            },
            None => DiscoveryResult::NotInstalled {
                tool: "tlc".to_string(),
                install_hint: INSTALL_HINT.to_string(),
            },
        }
    }

    fn run(&self, ctx: &VerifyContext) -> VerifyReport {
        let Some((launcher, _version)) =
            resolve_launcher(Some(&ctx.workspace_root), Some(&ctx.manifest))
        else {
            let mut missing = VerifyReport::empty("model", Outcome::ToolMissing);
            missing.diagnostics.push(INSTALL_HINT.to_string());
            return missing;
        };

        // Trace excerpts are collected out-of-band because
        // `walk_sources` only mirrors the one-line row note into the
        // report diagnostics.
        let excerpts: RefCell<Vec<String>> = RefCell::new(Vec::new());
        let workers = ctx
            .manifest
            .workers
            .map(|w| w.to_string())
            .unwrap_or_else(|| "auto".to_string());

        let mut report = crate::shell::walk_sources("model", ctx, &|path| {
            let (row, excerpt) = run_model_source(&launcher, &workers, path);
            if let Some(e) = excerpt {
                excerpts.borrow_mut().push(e);
            }
            row
        });

        report.diagnostics.extend(excerpts.into_inner());
        report
    }
}

/// Run TLC against one `.tla` spec and parse the transcript.
/// Returns the row plus an optional multi-line trace excerpt for the
/// report diagnostics.
fn run_model_source(
    launcher: &Launcher,
    workers: &str,
    source: &Path,
) -> (SourceRow, Option<String>) {
    let source_display = source.display().to_string();
    let parent = source.parent().unwrap_or_else(|| Path::new("."));
    let file_name = source
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| source_display.clone());

    // TLC scratch (states/, fingerprints) goes into a temp dir, never
    // the operator's repo.
    let metadir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            return (
                SourceRow::plain(
                    source_display.clone(),
                    Outcome::Failed,
                    format!(
                        "could not create a temp directory for TLC metadata: {e}. The run needs scratch space. Free disk space or set TMPDIR, then re-run `smctl verify model`."
                    ),
                ),
                None,
            );
        }
    };

    let mut args: Vec<String> = vec![
        "-workers".into(),
        workers.to_string(),
        "-metadir".into(),
        metadir.path().display().to_string(),
    ];
    let cfg = source.with_extension("cfg");
    if cfg.is_file()
        && let Some(cfg_name) = cfg.file_name()
    {
        args.push("-config".into());
        args.push(cfg_name.to_string_lossy().to_string());
    }
    args.push(file_name.clone());

    let reproduce = format!(
        "cd {} && {} {}",
        parent.display(),
        launcher.display(),
        args.iter()
            .map(|a| a.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let mut cmd = launcher.command();
    cmd.args(&args).current_dir(parent);
    let out = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return (
                SourceRow::plain(
                    source_display.clone(),
                    Outcome::Failed,
                    format!("could not spawn TLC for {source_display}: {e}. {INSTALL_HINT}"),
                ),
                None,
            );
        }
    };

    let mut text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.push('\n');
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    let analysis = tlc::analyze(&text);

    match analysis.verdict {
        TlcVerdict::Violation(violation) => {
            let excerpt = (!analysis.trace.is_empty())
                .then(|| tlc::render_trace_excerpt(&analysis.trace, &reproduce));
            (
                violation_row(source_display, violation, analysis.stats, &reproduce),
                excerpt,
            )
        }
        TlcVerdict::SpecError => {
            let quoted = if analysis.error_lines.is_empty() {
                output_head(&out.stdout, &out.stderr)
            } else {
                analysis.error_lines.join(" | ")
            };
            (
                SourceRow::plain(
                    source_display.clone(),
                    Outcome::Failed,
                    format!(
                        "TLC could not parse {source_display}: {quoted}. The spec or its config has a syntax or semantic error at the reported location. Re-run `{reproduce}` and fix it."
                    ),
                ),
                None,
            )
        }
        TlcVerdict::CompletedOk | TlcVerdict::Unknown if out.status.success() => {
            let note = analysis
                .stats
                .as_ref()
                .map(|s| {
                    format!(
                        "{} states generated, {} distinct",
                        s.states_generated, s.distinct_states
                    )
                })
                .unwrap_or_default();
            (
                SourceRow {
                    source: source_display,
                    outcome: Outcome::Passed,
                    note,
                    detail: analysis.stats,
                },
                None,
            )
        }
        TlcVerdict::CompletedOk | TlcVerdict::Unknown => {
            // Non-zero exit with no (or contradictory) text evidence:
            // the exit code is ground truth. Classify what we can,
            // warn that parsing degraded.
            if let Some(kind) = tlc::violation_kind_from_exit(out.status.code()) {
                let violation = Violation {
                    kind,
                    property: None,
                    trace_states: analysis.trace.len(),
                };
                (
                    violation_row(source_display, violation, analysis.stats, &reproduce),
                    None,
                )
            } else {
                tracing::warn!(
                    msgid = %smctl_log::MsgId::VerifyOutputUnparsed,
                    source = %source_display,
                    exit = out.status.code().unwrap_or(-1),
                    "tlc output did not match any known pattern",
                );
                let head = output_head(&out.stdout, &out.stderr);
                (
                    SourceRow::plain(
                        source_display.clone(),
                        Outcome::Failed,
                        format!(
                            "TLC exited with {} on {source_display} but its output matched no known pattern{}. The failure is real (non-zero exit) but smctl could not classify it. Re-run `{reproduce}` to inspect the full output.",
                            out.status
                                .code()
                                .map(|c| c.to_string())
                                .unwrap_or_else(|| "<signal>".into()),
                            if head.is_empty() {
                                String::new()
                            } else {
                                format!(" (output: {head})")
                            },
                        ),
                    ),
                    None,
                )
            }
        }
    }
}

fn violation_row(
    source_display: String,
    violation: Violation,
    stats: Option<crate::ModelCheckDetail>,
    reproduce: &str,
) -> SourceRow {
    tracing::error!(
        msgid = %smctl_log::MsgId::VerifyCounterExample,
        source = %source_display,
        kind = ?violation.kind,
        property = violation.property.as_deref().unwrap_or(""),
        trace_states = violation.trace_states as u64,
        "counter-example found",
    );
    let what = match (&violation.kind, &violation.property) {
        (crate::ViolationKind::Invariant, Some(name)) => {
            format!("invariant {name} is violated")
        }
        (crate::ViolationKind::Invariant, None) => "a safety invariant is violated".to_string(),
        (crate::ViolationKind::Deadlock, _) => "the model deadlocks".to_string(),
        (crate::ViolationKind::Liveness, _) => "a temporal property is violated".to_string(),
        (crate::ViolationKind::Assumption, _) => "an ASSUME clause is false".to_string(),
    };
    let trace_part = if violation.trace_states > 0 {
        format!(" after {} states", violation.trace_states)
    } else {
        String::new()
    };
    let note = format!(
        "{what}{trace_part}. TLC found a counter-example. Re-run `{reproduce}` for the full trace."
    );
    let mut detail = stats.unwrap_or_default();
    detail.violation = Some(violation);
    SourceRow {
        source: source_display,
        outcome: Outcome::Failed,
        note,
        detail: Some(detail),
    }
}
