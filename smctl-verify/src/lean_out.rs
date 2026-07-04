//! Lean 4 output parsing for the proof runner.
//!
//! Two input shapes, one message model: `lean --json` emits one JSON
//! object per stdout line (severity, pos, fileName, data, and — on
//! Lean ≥ 4.15 — a `kind` tag such as `hasSorry`); `lake build`
//! replays compiler messages as text lines
//! `{level}: {file}:{line}:{col}: {msg}`. Both parsers are lenient:
//! unknown fields, stray non-JSON lines (elan download notices,
//! progress output), and position-less messages are tolerated.
//! Classification never trusts exit codes — a `sorry` leaves lean's
//! exit at 0.

use serde::Deserialize;

use crate::{ProofCheckDetail, ProofFailure, ProofFailureKind};

/// Lines of a failing message folded into a diagnostic excerpt
/// before elision (mirrors the TLA+ trace excerpt bounds).
const EXCERPT_HEAD: usize = 4;
const EXCERPT_TAIL: usize = 2;

/// One compiler message, from either input shape.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct LeanMessage {
    pub severity: LeanSeverity,
    #[serde(default)]
    pub pos: Option<LeanPos>,
    #[serde(default, rename = "fileName")]
    pub file_name: Option<String>,
    #[serde(default)]
    pub data: String,
    /// Message kind tag (Lean ≥ 4.15); `hasSorry` marks admitted
    /// proofs on ≥ 4.19. Absent on older toolchains.
    #[serde(default)]
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LeanSeverity {
    Information,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
pub struct LeanPos {
    pub line: u64,
    pub column: u64,
}

impl LeanMessage {
    /// `file:line:col` when the message carries a position.
    pub fn location(&self) -> Option<String> {
        let pos = self.pos?;
        let file = self.file_name.as_deref()?;
        Some(format!("{file}:{}:{}", pos.line, pos.column))
    }

    /// First non-empty line of the message text.
    pub fn headline(&self) -> String {
        self.data
            .lines()
            .find(|l| !l.trim().is_empty())
            .unwrap_or("")
            .trim()
            .to_string()
    }
}

/// Does this message mark an admitted (incomplete) proof? The `kind`
/// tag wins when present; the warning text covers toolchains older
/// than 4.19 and both quote spellings (single quotes ≤ 4.26,
/// backticks from 4.27).
pub fn is_sorry(msg: &LeanMessage) -> bool {
    if msg.kind.as_deref() == Some("hasSorry") {
        return true;
    }
    msg.data.contains("declaration uses 'sorry'") || msg.data.contains("declaration uses `sorry`")
}

/// Parse `lean --json` stdout: one JSON message per line, stray
/// non-JSON lines skipped.
pub fn parse_lean_json(text: &str) -> Vec<LeanMessage> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            if !line.starts_with('{') {
                return None;
            }
            serde_json::from_str::<LeanMessage>(line).ok()
        })
        .collect()
}

/// Parse replayed compiler messages out of a `lake build` text log:
/// `{level}: {file}:{line}:{col}: {msg}` with level in
/// trace|info|warning|error. Level-only lines without a position
/// (e.g. `error: build failed`) become position-less messages.
/// Progress lines (`✔ [3/5] Built Foo`) don't match and are skipped.
pub fn parse_lake_log(text: &str) -> Vec<LeanMessage> {
    text.lines().filter_map(parse_lake_line).collect()
}

fn parse_lake_line(line: &str) -> Option<LeanMessage> {
    let line = line.trim();
    let (severity, rest) = if let Some(r) = line.strip_prefix("error: ") {
        (LeanSeverity::Error, r)
    } else if let Some(r) = line.strip_prefix("warning: ") {
        (LeanSeverity::Warning, r)
    } else if let Some(r) = line.strip_prefix("info: ") {
        (LeanSeverity::Information, r)
    } else if let Some(r) = line.strip_prefix("trace: ") {
        (LeanSeverity::Information, r)
    } else {
        return None;
    };

    // Lake's closing meta line (`error: build failed` on stderr)
    // restates that some target failed; counting it as a message
    // would inflate the error count and misclassify environmental
    // failures as proof errors.
    if rest.trim() == "build failed" {
        return None;
    }

    // Try `{file}:{line}:{col}: {msg}`; fall back to a bare message.
    let mut parts = rest.splitn(4, ':');
    if let (Some(file), Some(l), Some(c), Some(msg)) =
        (parts.next(), parts.next(), parts.next(), parts.next())
        && let (Ok(l), Ok(c)) = (l.trim().parse::<u64>(), c.trim().parse::<u64>())
        && !file.trim().is_empty()
    {
        return Some(LeanMessage {
            severity,
            pos: Some(LeanPos { line: l, column: c }),
            file_name: Some(file.trim().to_string()),
            data: msg.trim().to_string(),
            kind: None,
        });
    }
    Some(LeanMessage {
        severity,
        pos: None,
        file_name: None,
        data: rest.trim().to_string(),
        kind: None,
    })
}

/// Fold a message stream into the structured proof detail. Failure
/// precedence: the first positioned error (a proof error pointing at
/// a declaration), else the first sorry marker, else the first
/// position-less error (environment-level: lakefile, dependency,
/// toolchain — classified `build`, not `error`).
pub fn summarize(messages: &[LeanMessage]) -> ProofCheckDetail {
    let errors = messages
        .iter()
        .filter(|m| m.severity == LeanSeverity::Error)
        .count() as u64;
    let warnings = messages
        .iter()
        .filter(|m| m.severity == LeanSeverity::Warning)
        .count() as u64;
    let sorry_msgs: Vec<&LeanMessage> = messages.iter().filter(|m| is_sorry(m)).collect();

    let failure = messages
        .iter()
        .find(|m| m.severity == LeanSeverity::Error && m.location().is_some())
        .map(|m| ProofFailure {
            kind: ProofFailureKind::Error,
            location: m.location(),
            message: m.headline(),
        })
        .or_else(|| {
            sorry_msgs.first().map(|m| ProofFailure {
                kind: ProofFailureKind::Sorry,
                location: m.location(),
                message: m.headline(),
            })
        })
        .or_else(|| {
            messages
                .iter()
                .find(|m| m.severity == LeanSeverity::Error)
                .map(|m| ProofFailure {
                    kind: ProofFailureKind::Build,
                    location: None,
                    message: m.headline(),
                })
        });

    ProofCheckDetail {
        errors,
        warnings,
        sorries: sorry_msgs.len() as u64,
        failure,
    }
}

/// Render a bounded multi-line excerpt of the failing message for the
/// report diagnostics, closed by a runnable reproduce command.
pub fn render_message_excerpt(source: &str, msg: &LeanMessage, reproduce: &str) -> String {
    let lines: Vec<&str> = msg.data.lines().filter(|l| !l.trim().is_empty()).collect();
    let header = match msg.location() {
        Some(loc) => format!("proof failure at {loc}:"),
        None => format!("proof failure in {source}:"),
    };
    let mut out = vec![header];
    if lines.len() <= EXCERPT_HEAD + EXCERPT_TAIL {
        out.extend(lines.iter().map(|l| format!("  {l}")));
    } else {
        out.extend(lines[..EXCERPT_HEAD].iter().map(|l| format!("  {l}")));
        out.push(format!(
            "  … {} lines elided …",
            lines.len() - EXCERPT_HEAD - EXCERPT_TAIL
        ));
        out.extend(
            lines[lines.len() - EXCERPT_TAIL..]
                .iter()
                .map(|l| format!("  {l}")),
        );
    }
    out.push(format!("Re-run `{reproduce}` for the full message."));
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real shape emitted by Lean ≥ 4.19 (kind + isSilent present).
    const JSON_ERROR: &str = r#"{"severity":"error","pos":{"line":2,"column":51},"data":"`grind` failed\ncase grind\np q : Prop\nh : p\n⊢ q","kind":"[anonymous]","endPos":{"line":2,"column":56},"fileName":"/tmp/test.lean","keepFullRange":false,"isSilent":false,"caption":""}"#;
    const JSON_SORRY_KIND: &str = r#"{"severity":"warning","pos":{"line":144,"column":2},"data":"declaration uses 'sorry'","kind":"hasSorry","fileName":"CapabilityNonForgery.lean"}"#;
    /// Pre-4.15 toolchains emit no `kind` field.
    const JSON_SORRY_KINDLESS: &str = r#"{"severity":"warning","pos":{"line":9,"column":0},"data":"declaration uses 'sorry'","fileName":"Old.lean"}"#;

    #[test]
    fn parses_real_json_error_message() {
        let msgs = parse_lean_json(JSON_ERROR);
        assert_eq!(msgs.len(), 1);
        let m = &msgs[0];
        assert_eq!(m.severity, LeanSeverity::Error);
        assert_eq!(m.location().as_deref(), Some("/tmp/test.lean:2:51"));
        assert_eq!(m.headline(), "`grind` failed");
        assert!(!is_sorry(m));
    }

    #[test]
    fn skips_non_json_noise_lines() {
        let text = format!("info: downloading component 'lean'\n{JSON_ERROR}\nnot json either\n");
        let msgs = parse_lean_json(&text);
        assert_eq!(msgs.len(), 1);
    }

    #[test]
    fn sorry_detected_by_kind_text_and_backticks() {
        let by_kind = &parse_lean_json(JSON_SORRY_KIND)[0];
        assert!(is_sorry(by_kind));

        let kindless = &parse_lean_json(JSON_SORRY_KINDLESS)[0];
        assert!(kindless.kind.is_none());
        assert!(is_sorry(kindless));

        // 4.27+ spelling.
        let backticks = LeanMessage {
            severity: LeanSeverity::Warning,
            pos: None,
            file_name: None,
            data: "declaration uses `sorry`".into(),
            kind: None,
        };
        assert!(is_sorry(&backticks));
    }

    #[test]
    fn lake_log_extracts_replayed_messages() {
        let log = "\
✔ [1/3] Built Proofs.Basic
error: Proofs/Bad.lean:12:4: unknown identifier 'foo'
warning: Proofs/Sad.lean:3:0: declaration uses 'sorry'
Some required targets logged failures:
- Proofs.Bad
error: build failed
";
        let msgs = parse_lake_log(log);
        // The closing `error: build failed` meta line is skipped: it
        // restates the replayed failure and must not inflate counts.
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].severity, LeanSeverity::Error);
        assert_eq!(msgs[0].location().as_deref(), Some("Proofs/Bad.lean:12:4"));
        assert!(is_sorry(&msgs[1]));
    }

    #[test]
    fn lake_meta_only_failure_classifies_as_build() {
        // A lakefile-level failure replays no compiler message; only
        // position-less error lines appear.
        let log = "error: no lakefile.lean or lakefile.toml found\nerror: build failed\n";
        let msgs = parse_lake_log(log);
        assert_eq!(msgs.len(), 1);
        let d = summarize(&msgs);
        assert_eq!(d.errors, 1);
        let f = d.failure.unwrap();
        assert_eq!(f.kind, ProofFailureKind::Build);
        assert_eq!(f.location, None);
        assert!(f.message.contains("no lakefile"), "{}", f.message);
    }

    #[test]
    fn clean_lake_log_yields_no_messages() {
        let log = "✔ [1/2] Built Proofs.Basic\nBuild completed successfully (2 jobs).\n";
        assert!(parse_lake_log(log).is_empty());
    }

    #[test]
    fn summarize_counts_and_prefers_error_over_sorry() {
        let msgs = parse_lean_json(&format!("{JSON_SORRY_KIND}\n{JSON_ERROR}"));
        let d = summarize(&msgs);
        assert_eq!((d.errors, d.warnings, d.sorries), (1, 1, 1));
        let f = d.failure.unwrap();
        assert_eq!(f.kind, ProofFailureKind::Error);
        assert_eq!(f.location.as_deref(), Some("/tmp/test.lean:2:51"));
    }

    #[test]
    fn summarize_sorry_only_fails_as_sorry() {
        let msgs = parse_lean_json(JSON_SORRY_KIND);
        let d = summarize(&msgs);
        assert_eq!((d.errors, d.warnings, d.sorries), (0, 1, 1));
        let f = d.failure.unwrap();
        assert_eq!(f.kind, ProofFailureKind::Sorry);
        assert_eq!(
            f.location.as_deref(),
            Some("CapabilityNonForgery.lean:144:2")
        );
    }

    #[test]
    fn summarize_clean_run_has_no_failure() {
        let d = summarize(&[]);
        assert_eq!((d.errors, d.warnings, d.sorries), (0, 0, 0));
        assert!(d.failure.is_none());
    }

    #[test]
    fn excerpt_is_bounded_with_elision_marker() {
        let msg = LeanMessage {
            severity: LeanSeverity::Error,
            pos: Some(LeanPos { line: 1, column: 0 }),
            file_name: Some("Big.lean".into()),
            data: (1..=10)
                .map(|i| format!("line{i}"))
                .collect::<Vec<_>>()
                .join("\n"),
            kind: None,
        };
        let e = render_message_excerpt("Big.lean", &msg, "lean Big.lean");
        assert!(e.starts_with("proof failure at Big.lean:1:0:"));
        assert!(e.contains("line1") && e.contains("line4"), "{e}");
        assert!(e.contains("… 4 lines elided …"), "{e}");
        assert!(!e.contains("line5"), "elided line leaked: {e}");
        assert!(e.contains("line9") && e.contains("line10"), "{e}");
        assert!(e.ends_with("Re-run `lean Big.lean` for the full message."));
    }
}
