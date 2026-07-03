//! Pan (SPIN verifier) output parser for the protocol verb.
//!
//! Parses the summary text the compiled pan verifier prints. The
//! `errors: N` count is ground truth for pass/fail; failure classes
//! (assertion violation, acceptance cycle, invalid end state) are
//! parsed best-effort with the caller falling back to
//! `SMCTL-0506 VerifyOutputUnparsed` when nothing matches.

use crate::{ProtocolCheckDetail, ProtocolViolation, ProtocolViolationKind};

/// How many leading trail steps a rendered excerpt keeps.
const TRAIL_HEAD: usize = 4;
/// How many trailing trail steps a rendered excerpt keeps.
const TRAIL_TAIL: usize = 2;

/// What pan's output said about the run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PanVerdict {
    /// `errors: 0` — the protocol verified.
    Passed,
    /// `errors: 0` but pan reported an incomplete search (depth
    /// truncation) — the verification is inconclusive.
    Incomplete,
    /// Non-zero error count with a recognized failure class.
    Violation(ProtocolViolation),
    /// Non-zero error count but no recognized class — still failed
    /// (the count is ground truth), parsing degraded.
    FailedUnclassified { errors: u64 },
    /// No `errors:` line found at all.
    Unknown,
}

/// Full analysis of one pan transcript.
#[derive(Debug, Clone)]
pub struct PanAnalysis {
    /// Parsed state-space statistics when the summary appeared.
    pub stats: Option<ProtocolCheckDetail>,
    pub verdict: PanVerdict,
}

/// Parse a full pan transcript (stdout + stderr, concatenated).
pub fn analyze(output: &str) -> PanAnalysis {
    let mut errors: Option<u64> = None;
    let mut states_stored: Option<u64> = None;
    let mut states_matched: Option<u64> = None;
    let mut depth_reached: Option<u64> = None;
    let mut kind: Option<ProtocolViolationKind> = None;
    let mut violation_detail: Option<String> = None;
    let mut search_incomplete = false;

    for line in output.lines() {
        let trimmed = line.trim();

        // "State-vector 44 byte, depth reached 1024, errors: 1"
        if let Some(idx) = trimmed.find("errors:") {
            if let Some(n) = first_number(&trimmed[idx + "errors:".len()..]) {
                errors = Some(n);
            }
            if let Some(didx) = trimmed.find("depth reached")
                && let Some(d) = first_number(&trimmed[didx + "depth reached".len()..])
            {
                depth_reached = Some(d);
            }
            continue;
        }

        if trimmed.contains("Search not completed")
            || trimmed.contains("max search depth too small")
        {
            search_incomplete = true;
            continue;
        }

        // "  123456 states, stored" / "   7890 states, matched"
        if trimmed.ends_with("states, stored")
            && let Some(n) = first_number(trimmed)
        {
            states_stored = Some(n);
            continue;
        }
        if trimmed.ends_with("states, matched")
            && let Some(n) = first_number(trimmed)
        {
            states_matched = Some(n);
            continue;
        }

        // Failure classes only ever appear on pan-prefixed lines
        // ("pan:1: assertion violated ...", "pan: acceptance cycle ...").
        // The search-for banner lists "invalid end states +" on every
        // run, so a bare substring match would misclassify.
        if kind.is_none() && trimmed.starts_with("pan") {
            // "pan:1: assertion violated (x<=MAX) (at depth 42)"
            if let Some(idx) = trimmed.find("assertion violated") {
                kind = Some(ProtocolViolationKind::Assertion);
                let rest = trimmed[idx + "assertion violated".len()..].trim();
                if !rest.is_empty() {
                    violation_detail = Some(rest.to_string());
                }
            } else if trimmed.contains("acceptance cycle") {
                kind = Some(ProtocolViolationKind::AcceptanceCycle);
            } else if trimmed.contains("invalid end state") {
                kind = Some(ProtocolViolationKind::InvalidEndState);
            }
        }
    }

    let stats = match (states_stored, states_matched, depth_reached) {
        (None, None, None) => None,
        _ => Some(ProtocolCheckDetail {
            states_stored: states_stored.unwrap_or(0),
            states_matched: states_matched.unwrap_or(0),
            depth_reached: depth_reached.unwrap_or(0),
            violation: None,
        }),
    };

    let verdict = match (errors, kind) {
        // errors: 0 only counts as verified when pan actually
        // completed the search — a depth-truncated run is
        // inconclusive, not a pass.
        (Some(0), _) if search_incomplete => PanVerdict::Incomplete,
        (Some(0), _) => PanVerdict::Passed,
        (Some(_), Some(k)) => PanVerdict::Violation(ProtocolViolation {
            kind: k,
            detail: violation_detail,
            trail_steps: 0,
        }),
        (Some(n), None) => PanVerdict::FailedUnclassified { errors: n },
        // A recognized failure class without an errors: line still
        // counts as a violation — pan sometimes aborts before the
        // summary (e.g. assertion with -E).
        (None, Some(k)) => PanVerdict::Violation(ProtocolViolation {
            kind: k,
            detail: violation_detail,
            trail_steps: 0,
        }),
        (None, None) => PanVerdict::Unknown,
    };

    PanAnalysis { stats, verdict }
}

/// Extract the step lines from a `spin -t -p` trail replay. Steps are
/// the numbered transition lines (`  1:    proc  0 (main:1) ...`).
pub fn trail_steps(replay_output: &str) -> Vec<String> {
    replay_output
        .lines()
        .map(str::trim)
        .filter(|l| {
            let Some((num, rest)) = l.split_once(':') else {
                return false;
            };
            !num.is_empty()
                && num.chars().all(|c| c.is_ascii_digit())
                && rest.trim_start().starts_with("proc")
        })
        .map(str::to_string)
        .collect()
}

/// Render a bounded trail excerpt: first [`TRAIL_HEAD`] and last
/// [`TRAIL_TAIL`] steps with an elision marker, closed by a
/// three-part line pointing at the reproduce commands.
pub fn render_trail_excerpt(steps: &[String], reproduce: &str) -> String {
    let mut out = String::from("Counter-example trail:\n");
    let truncated = steps.len() > TRAIL_HEAD + TRAIL_TAIL;
    if !truncated {
        for step in steps {
            out.push_str(step);
            out.push('\n');
        }
        out.push_str(&format!(
            "The trail above is the full counter-example, showing how the protocol reaches the failure. Re-run `{reproduce}` to reproduce it."
        ));
    } else {
        for step in &steps[..TRAIL_HEAD] {
            out.push_str(step);
            out.push('\n');
        }
        out.push_str(&format!(
            "… {} steps elided …\n",
            steps.len() - TRAIL_HEAD - TRAIL_TAIL
        ));
        for step in &steps[steps.len() - TRAIL_TAIL..] {
            out.push_str(step);
            out.push('\n');
        }
        out.push_str(&format!(
            "The trail above is truncated. It shows how the protocol reaches the failure. Re-run `{reproduce}` for the full counter-example."
        ));
    }
    out
}

/// First unsigned integer in `s`, tolerating commas and leading text.
fn first_number(s: &str) -> Option<u64> {
    let cleaned = s.replace(',', "");
    let start = cleaned.find(|c: char| c.is_ascii_digit())?;
    let digits: String = cleaned[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASS_TRANSCRIPT: &str = "\
(Spin Version 6.5.2 -- 6 December 2019)
	+ Partial Order Reduction

Full statespace search for:
	never claim         	- (none specified)
	assertion violations	+
	acceptance   cycles 	+ (fairness disabled)
	invalid end states	+

State-vector 44 byte, depth reached 999, errors: 0
     8192 states, stored
     2048 states, matched
    10240 transitions (= stored+matched)
pan: elapsed time 0.01 seconds
";

    const ASSERT_TRANSCRIPT: &str = "\
pan:1: assertion violated (nfull(ch)) (at depth 42)
pan: wrote model.pml.trail

(Spin Version 6.5.2 -- 6 December 2019)
Warning: Search not completed

State-vector 36 byte, depth reached 42, errors: 1
      120 states, stored
       14 states, matched
";

    const CYCLE_TRANSCRIPT: &str = "\
pan: acceptance cycle (at depth 17)
pan: wrote live.pml.trail
State-vector 28 byte, depth reached 17, errors: 1
       50 states, stored
";

    const DEADLOCK_TRANSCRIPT: &str = "\
pan: invalid end state (at depth 9)
pan: wrote dead.pml.trail
State-vector 20 byte, depth reached 9, errors: 1
       12 states, stored
";

    const TRAIL_REPLAY: &str = "\
using statement merging
  1:	proc  0 (init:1) model.pml:12 (state 1)	[ch!msg]
  2:	proc  1 (recv:1) model.pml:20 (state 3)	[ch?m]
  3:	proc  0 (init:1) model.pml:13 (state 2)	[ch!msg]
spin: trail ends after 3 steps
#processes: 2
";

    #[test]
    fn pass_transcript_yields_passed_with_stats() {
        let a = analyze(PASS_TRANSCRIPT);
        assert_eq!(a.verdict, PanVerdict::Passed);
        let stats = a.stats.expect("stats");
        assert_eq!(stats.states_stored, 8192);
        assert_eq!(stats.states_matched, 2048);
        assert_eq!(stats.depth_reached, 999);
    }

    #[test]
    fn assertion_transcript_classifies_with_detail() {
        let a = analyze(ASSERT_TRANSCRIPT);
        match &a.verdict {
            PanVerdict::Violation(v) => {
                assert_eq!(v.kind, ProtocolViolationKind::Assertion);
                assert!(v.detail.as_deref().unwrap().contains("nfull(ch)"));
            }
            other => panic!("expected assertion violation, got {other:?}"),
        }
        assert_eq!(a.stats.expect("stats").depth_reached, 42);
    }

    #[test]
    fn cycle_and_deadlock_classify_distinctly() {
        match analyze(CYCLE_TRANSCRIPT).verdict {
            PanVerdict::Violation(v) => {
                assert_eq!(v.kind, ProtocolViolationKind::AcceptanceCycle)
            }
            other => panic!("{other:?}"),
        }
        match analyze(DEADLOCK_TRANSCRIPT).verdict {
            PanVerdict::Violation(v) => {
                assert_eq!(v.kind, ProtocolViolationKind::InvalidEndState)
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn unclassified_nonzero_errors_still_fail() {
        let a = analyze("State-vector 20 byte, depth reached 5, errors: 3\n");
        assert_eq!(a.verdict, PanVerdict::FailedUnclassified { errors: 3 });
    }

    #[test]
    fn banner_lines_do_not_classify_failures() {
        // Every pan run prints the search-for banner, which contains
        // "invalid end states +". An unclassified real failure must
        // stay unclassified rather than reading as a deadlock.
        let t = "Full statespace search for:\n\tassertion violations\t+\n\tinvalid end states\t+\n\nState-vector 20 byte, depth reached 5, errors: 2\n";
        let a = analyze(t);
        assert_eq!(a.verdict, PanVerdict::FailedUnclassified { errors: 2 });
    }

    #[test]
    fn incomplete_search_is_not_a_pass() {
        let t = "Warning: Search not completed\nState-vector 20 byte, depth reached 9999, errors: 0\n  500 states, stored\n";
        let a = analyze(t);
        assert_eq!(a.verdict, PanVerdict::Incomplete);
        assert_eq!(a.stats.expect("stats").states_stored, 500);
    }

    #[test]
    fn max_depth_warning_is_incomplete() {
        let t = "error: max search depth too small\nState-vector 20 byte, depth reached 10000, errors: 0\n";
        assert_eq!(analyze(t).verdict, PanVerdict::Incomplete);
    }

    #[test]
    fn garbage_yields_unknown() {
        let a = analyze("nothing pan-like here\n");
        assert_eq!(a.verdict, PanVerdict::Unknown);
        assert!(a.stats.is_none());
    }

    #[test]
    fn trail_replay_extracts_numbered_proc_steps() {
        let steps = trail_steps(TRAIL_REPLAY);
        assert_eq!(steps.len(), 3);
        assert!(steps[0].contains("proc  0"));
        assert!(steps[2].contains("state 2"));
    }

    #[test]
    fn short_trail_renders_in_full() {
        let steps = vec!["1: proc 0 a".to_string(), "2: proc 1 b".to_string()];
        let excerpt = render_trail_excerpt(&steps, "spin -t -p m.pml");
        assert!(excerpt.contains("1: proc 0 a"));
        assert!(!excerpt.contains("elided"));
        assert!(excerpt.contains("full counter-example"));
    }

    #[test]
    fn long_trail_is_bounded_with_elision() {
        let steps: Vec<String> = (1..=20).map(|i| format!("{i}: proc 0 s{i}")).collect();
        let excerpt = render_trail_excerpt(&steps, "spin -t -p m.pml");
        assert!(excerpt.contains("1: proc 0 s1"));
        assert!(excerpt.contains("… 14 steps elided …"));
        assert!(excerpt.contains("20: proc 0 s20"));
        assert!(excerpt.contains("truncated"));
    }
}
