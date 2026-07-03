//! SPIN/Promela protocol runner.
//!
//! Deep integration per `spin-protocol-runner-v1`: runs the full
//! pipeline per source — `spin -a` (codegen), `cc -O2 -o pan pan.c`
//! (compile), `./pan -a` (verify with acceptance-cycle detection) —
//! inside a per-source temp directory so `pan.*` and `.trail`
//! artifacts never land in the operator's repo. Pan output is parsed
//! (see `pan.rs`); counter-example trails are replayed with
//! `spin -t -p` and rendered as bounded excerpts.

use std::cell::RefCell;
use std::path::Path;
use std::process::Command;

use crate::pan::{self, PanVerdict};
use crate::shell::{Shell, discover_binary, output_head, resolve_binary, sh_quote};
use crate::{
    DiscoveryResult, Outcome, ProtocolViolation, ProtocolViolationKind, SourceRow, Verifier,
    VerifyContext, VerifyDetail, VerifyReport,
};

/// Overrides the spin binary wholesale. Test-and-escape-hatch only.
pub const ENV_SPIN_BIN: &str = "SMCTL_VERIFY_SPIN_BIN";
/// Overrides the C compiler used to build pan. Test-and-escape-hatch only.
pub const ENV_PAN_CC: &str = "SMCTL_VERIFY_PAN_CC";

const CC_INSTALL_HINT: &str = "verify protocol needs a C compiler to build SPIN's pan verifier. Install the Xcode Command Line Tools (`xcode-select --install`) on macOS or your distro's build-essential package";

const SHELL: Shell<'static> = Shell {
    name: "protocol",
    binary: "spin",
    version_args: &["-V"],
    run_args: &[],
    install_hint: "install via `brew install spin` (macOS) or your distro's package manager",
    env_override: Some(ENV_SPIN_BIN),
};

fn resolve_cc() -> String {
    std::env::var(ENV_PAN_CC)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "cc".to_string())
}

fn cc_available() -> bool {
    Command::new(resolve_cc()).arg("--version").output().is_ok()
}

#[derive(Debug, Default)]
pub struct SpinVerifier;

impl SpinVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Verifier for SpinVerifier {
    fn name(&self) -> &'static str {
        SHELL.name
    }

    fn discover(&self) -> DiscoveryResult {
        discover_binary(&SHELL)
    }

    fn run(&self, ctx: &VerifyContext) -> VerifyReport {
        if !discover_binary(&SHELL).is_installed() {
            let mut missing = VerifyReport::empty(SHELL.name, Outcome::ToolMissing);
            missing.diagnostics.push(format!(
                "spin is not installed on PATH. {}",
                SHELL.install_hint
            ));
            return missing;
        }
        if !cc_available() {
            let mut missing = VerifyReport::empty(SHELL.name, Outcome::ToolMissing);
            missing.diagnostics.push(CC_INSTALL_HINT.to_string());
            return missing;
        }

        let spin_bin = resolve_binary(&SHELL);
        let cc_bin = resolve_cc();
        let excerpts: RefCell<Vec<String>> = RefCell::new(Vec::new());

        let mut report = crate::shell::walk_sources("protocol", ctx, &|path| {
            let (row, excerpt) = run_protocol_source(&spin_bin, &cc_bin, path);
            if let Some(e) = excerpt {
                excerpts.borrow_mut().push(e);
            }
            row
        });

        report.diagnostics.extend(excerpts.into_inner());
        report
    }
}

/// Which tool the missing-tool envelope should name for the protocol
/// verb right now: `cc` when spin is present but the compiler isn't.
/// The CLI consults this instead of `discover()` (which only knows
/// about spin) when building the `tool_missing` envelope.
pub fn missing_tool_for_protocol() -> Option<(String, String)> {
    if !discover_binary(&SHELL).is_installed() {
        return Some(("spin".to_string(), SHELL.install_hint.to_string()));
    }
    if !cc_available() {
        return Some(("cc".to_string(), CC_INSTALL_HINT.to_string()));
    }
    None
}

/// Run the full SPIN pipeline against one `.pml` spec.
/// Returns the row plus an optional trail excerpt for the report
/// diagnostics.
fn run_protocol_source(spin_bin: &str, cc_bin: &str, source: &Path) -> (SourceRow, Option<String>) {
    let source_display = source.display().to_string();
    let abs_source = source
        .canonicalize()
        .unwrap_or_else(|_| source.to_path_buf());

    let workdir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            return (
                SourceRow::plain(
                    source_display.clone(),
                    Outcome::Failed,
                    format!(
                        "could not create a temp work directory for SPIN: {e}. The pipeline needs scratch space for pan artifacts. Free disk space or set TMPDIR, then re-run `smctl verify protocol`."
                    ),
                ),
                None,
            );
        }
    };
    let wd = workdir.path();

    let reproduce = format!(
        "cd $(mktemp -d) && {spin} -a {src} && {cc} -O2 -o pan pan.c && ./pan -a",
        spin = sh_quote(spin_bin),
        src = sh_quote(&abs_source.display().to_string()),
        cc = sh_quote(cc_bin),
    );

    // Step 1: spin -a — Promela parse + pan.c codegen.
    let gen_out = match Command::new(spin_bin)
        .arg("-a")
        .arg(&abs_source)
        .current_dir(wd)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return (
                SourceRow::plain(
                    source_display.clone(),
                    Outcome::Failed,
                    format!(
                        "could not spawn spin for {source_display}: {e}. {}",
                        SHELL.install_hint
                    ),
                ),
                None,
            );
        }
    };
    // spin -a writes pan.c on success; a syntax error leaves it
    // missing (spin's exit code is unreliable across versions, so the
    // artifact is the signal).
    if !wd.join("pan.c").is_file() {
        let head = output_head(&gen_out.stdout, &gen_out.stderr);
        return (
            SourceRow::plain(
                source_display.clone(),
                Outcome::Failed,
                format!(
                    "spin could not generate a verifier for {source_display}: {head}. The Promela source has a syntax or semantic error at the reported location. Re-run `{spin} -a {src}` and fix it.",
                    spin = sh_quote(spin_bin),
                    src = sh_quote(&abs_source.display().to_string()),
                ),
            ),
            None,
        );
    }

    // Step 2: compile pan.
    let cc_out = match Command::new(cc_bin)
        .args(["-O2", "-o", "pan", "pan.c"])
        .current_dir(wd)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return (
                SourceRow::plain(
                    source_display.clone(),
                    Outcome::Failed,
                    format!(
                        "could not spawn {cc_bin} for {source_display}: {e}. {CC_INSTALL_HINT}."
                    ),
                ),
                None,
            );
        }
    };
    if !cc_out.status.success() {
        let head = output_head(&cc_out.stdout, &cc_out.stderr);
        return (
            SourceRow::plain(
                source_display.clone(),
                Outcome::Failed,
                format!(
                    "{cc_bin} failed to build the pan verifier for {source_display}: {head}. The generated pan.c did not compile, which usually indicates an unsupported Promela construct. Re-run `{reproduce}` to inspect the full compiler output.",
                ),
            ),
            None,
        );
    }

    // Step 3: run pan with acceptance-cycle detection.
    let pan_out = match Command::new(wd.join("pan"))
        .arg("-a")
        .current_dir(wd)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return (
                SourceRow::plain(
                    source_display.clone(),
                    Outcome::Failed,
                    format!(
                        "could not run the compiled pan verifier for {source_display}: {e}. Re-run `{reproduce}` to reproduce the failure."
                    ),
                ),
                None,
            );
        }
    };

    let mut text = String::from_utf8_lossy(&pan_out.stdout).into_owned();
    text.push('\n');
    text.push_str(&String::from_utf8_lossy(&pan_out.stderr));
    let analysis = pan::analyze(&text);

    match analysis.verdict {
        PanVerdict::Passed => {
            let note = analysis
                .stats
                .as_ref()
                .map(|s| {
                    format!(
                        "{} states stored, {} matched, depth {}",
                        s.states_stored, s.states_matched, s.depth_reached
                    )
                })
                .unwrap_or_default();
            (
                SourceRow {
                    source: source_display,
                    outcome: Outcome::Passed,
                    note,
                    detail: analysis.stats.map(VerifyDetail::Protocol),
                },
                None,
            )
        }
        PanVerdict::Violation(mut violation) => {
            // Replay the trail for a bounded counter-example excerpt.
            let replay = sh_quote(spin_bin);
            let replay_cmd = format!(
                "{replay} -t -p {}",
                sh_quote(&abs_source.display().to_string())
            );
            let excerpt = if trail_exists(wd) {
                let steps = Command::new(spin_bin)
                    .args(["-t", "-p"])
                    .arg(&abs_source)
                    .current_dir(wd)
                    .output()
                    .map(|o| {
                        let mut t = String::from_utf8_lossy(&o.stdout).into_owned();
                        t.push('\n');
                        t.push_str(&String::from_utf8_lossy(&o.stderr));
                        pan::trail_steps(&t)
                    })
                    .unwrap_or_default();
                violation.trail_steps = steps.len();
                (!steps.is_empty()).then(|| pan::render_trail_excerpt(&steps, &replay_cmd))
            } else {
                None
            };
            (
                violation_row(source_display, violation, analysis.stats, &reproduce),
                excerpt,
            )
        }
        PanVerdict::FailedUnclassified { errors } => {
            tracing::warn!(
                msgid = %smctl_log::MsgId::VerifyOutputUnparsed,
                source = %source_display,
                errors,
                "pan reported errors but no known failure class matched",
            );
            let head = output_head(&pan_out.stdout, &pan_out.stderr);
            (
                SourceRow {
                    source: source_display.clone(),
                    outcome: Outcome::Failed,
                    note: format!(
                        "pan reported {errors} error(s) for {source_display} but the failure class matched no known pattern (output: {head}). The failure is real (pan's error count is ground truth) but smctl could not classify it. Re-run `{reproduce}` to inspect the full output.",
                    ),
                    detail: analysis.stats.map(VerifyDetail::Protocol),
                },
                None,
            )
        }
        PanVerdict::Unknown if pan_out.status.success() => {
            // No errors: line but a clean exit — treat as passed with
            // whatever stats surfaced; nothing to warn about beyond
            // the missing summary.
            (
                SourceRow {
                    source: source_display,
                    outcome: Outcome::Passed,
                    note: String::new(),
                    detail: analysis.stats.map(VerifyDetail::Protocol),
                },
                None,
            )
        }
        PanVerdict::Unknown => {
            tracing::warn!(
                msgid = %smctl_log::MsgId::VerifyOutputUnparsed,
                source = %source_display,
                exit = pan_out.status.code().unwrap_or(-1),
                "pan output did not match any known pattern",
            );
            let head = output_head(&pan_out.stdout, &pan_out.stderr);
            (
                SourceRow::plain(
                    source_display.clone(),
                    Outcome::Failed,
                    format!(
                        "pan exited with {} on {source_display} but its output matched no known pattern{}. The failure is real (non-zero exit) but smctl could not classify it. Re-run `{reproduce}` to inspect the full output.",
                        pan_out
                            .status
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

fn trail_exists(workdir: &Path) -> bool {
    std::fs::read_dir(workdir)
        .map(|entries| {
            entries
                .flatten()
                .any(|e| e.path().extension().is_some_and(|x| x == "trail"))
        })
        .unwrap_or(false)
}

fn violation_row(
    source_display: String,
    violation: ProtocolViolation,
    stats: Option<crate::ProtocolCheckDetail>,
    reproduce: &str,
) -> SourceRow {
    tracing::error!(
        msgid = %smctl_log::MsgId::VerifyCounterExample,
        source = %source_display,
        kind = ?violation.kind,
        detail = violation.detail.as_deref().unwrap_or(""),
        trail_steps = violation.trail_steps as u64,
        "counter-example found",
    );
    let what = match &violation.kind {
        ProtocolViolationKind::Assertion => match &violation.detail {
            Some(d) => format!("assertion {d} is violated"),
            None => "an assertion is violated".to_string(),
        },
        ProtocolViolationKind::AcceptanceCycle => {
            "an acceptance cycle exists (liveness violation)".to_string()
        }
        ProtocolViolationKind::InvalidEndState => {
            "the protocol can reach an invalid end state (deadlock)".to_string()
        }
    };
    let trail_part = if violation.trail_steps > 0 {
        format!(" after {} trail steps", violation.trail_steps)
    } else {
        String::new()
    };
    let note = format!(
        "{what}{trail_part}. Pan found a counter-example. Re-run `{reproduce}` for the full trail."
    );
    let mut detail = stats.unwrap_or_default();
    detail.violation = Some(violation);
    SourceRow {
        source: source_display,
        outcome: Outcome::Failed,
        note,
        detail: Some(VerifyDetail::Protocol(detail)),
    }
}
