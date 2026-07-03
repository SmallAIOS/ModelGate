//! TLC output parser for the TLA+ model verifier.
//!
//! Parses the human text TLC prints — the de facto stable interface
//! every TLA+ CI harness greps — rather than `-tool` mode's numbered
//! message framing (see `tla-plus-runner-v1/design.md`). When no
//! pattern matches, the caller falls back to the process exit code;
//! text evidence always wins when present.

use crate::{ModelCheckDetail, Violation, ViolationKind};

/// How many leading trace states a rendered excerpt keeps.
const TRACE_HEAD: usize = 4;
/// How many trailing trace states a rendered excerpt keeps.
const TRACE_TAIL: usize = 2;

/// What TLC's text output said about the run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlcVerdict {
    /// "Model checking completed. No error has been found."
    CompletedOk,
    /// A property violation with an optional counter-example trace.
    Violation(Violation),
    /// TLC rejected the spec or config before checking (parse or
    /// semantic errors).
    SpecError,
    /// No known pattern matched.
    Unknown,
}

/// Full analysis of one TLC transcript.
#[derive(Debug, Clone)]
pub struct TlcAnalysis {
    /// Parsed state-space statistics when the summary line appeared.
    pub stats: Option<ModelCheckDetail>,
    pub verdict: TlcVerdict,
    /// Counter-example trace, one entry per `State N:` block
    /// (header line plus its variable lines).
    pub trace: Vec<String>,
    /// Lines that evidence a spec/config error, for quoting.
    pub error_lines: Vec<String>,
}

/// Classification of a TLC exit code, used as fallback when text
/// parsing yields [`TlcVerdict::Unknown`]. Codes per tla2tools
/// `util.ExitStatus`: 0 success, 10 assumption violation, 11
/// deadlock, 12 safety violation, 13 liveness violation.
pub fn violation_kind_from_exit(code: Option<i32>) -> Option<ViolationKind> {
    match code {
        Some(10) => Some(ViolationKind::Assumption),
        Some(11) => Some(ViolationKind::Deadlock),
        Some(12) => Some(ViolationKind::Invariant),
        Some(13) => Some(ViolationKind::Liveness),
        _ => None,
    }
}

/// Parse a full TLC transcript (stdout + stderr, concatenated).
pub fn analyze(output: &str) -> TlcAnalysis {
    let mut stats = None;
    let mut violation: Option<Violation> = None;
    let mut completed_ok = false;
    let mut spec_error = false;
    let mut error_lines = Vec::new();
    let mut trace: Vec<String> = Vec::new();
    let mut current_state: Option<String> = None;

    for line in output.lines() {
        let trimmed = line.trim();

        // Trace blocks: "State N: <...>" header, variable lines
        // follow until a blank line or the next non-indented marker.
        if is_state_header(trimmed) {
            if let Some(block) = current_state.take() {
                trace.push(block);
            }
            current_state = Some(trimmed.to_string());
            continue;
        }
        if let Some(block) = current_state.as_mut() {
            if trimmed.is_empty() {
                trace.push(current_state.take().unwrap());
            } else if trimmed.starts_with("/\\") || trimmed.starts_with("\\/") {
                block.push('\n');
                block.push_str(trimmed);
                continue;
            } else {
                trace.push(current_state.take().unwrap());
                // fall through: the current line may match something below
            }
        }

        if trimmed.contains("states generated")
            && trimmed.contains("distinct states found")
            && let Some(parsed) = parse_stats_line(trimmed)
        {
            stats = Some(parsed);
            continue;
        }

        if trimmed.contains("Model checking completed. No error has been found") {
            completed_ok = true;
            continue;
        }

        if violation.is_none() {
            if let Some(rest) = trimmed.strip_prefix("Error: Invariant ") {
                if let Some(name) = rest.split(" is violated").next()
                    && rest.contains("is violated")
                {
                    violation = Some(Violation {
                        kind: ViolationKind::Invariant,
                        property: Some(name.trim().to_string()),
                        trace_states: 0,
                    });
                    continue;
                }
            } else if trimmed.contains("Error: Deadlock reached") {
                violation = Some(Violation {
                    kind: ViolationKind::Deadlock,
                    property: None,
                    trace_states: 0,
                });
                continue;
            } else if trimmed.contains("Error: Temporal properties were violated") {
                violation = Some(Violation {
                    kind: ViolationKind::Liveness,
                    property: None,
                    trace_states: 0,
                });
                continue;
            } else if trimmed.starts_with("Error: Assumption") {
                violation = Some(Violation {
                    kind: ViolationKind::Assumption,
                    property: None,
                    trace_states: 0,
                });
                continue;
            }
        }

        if trimmed.contains("Parse Error")
            || trimmed.contains("Was expecting")
            || trimmed.contains("Semantic error")
            || trimmed.contains("Fatal errors while parsing TLA+ spec")
            || trimmed.contains("Unknown operator")
        {
            spec_error = true;
            if error_lines.len() < 6 {
                error_lines.push(trimmed.to_string());
            }
        }
    }
    if let Some(block) = current_state.take() {
        trace.push(block);
    }

    if let Some(v) = violation.as_mut() {
        v.trace_states = trace.len();
    }

    let verdict = if let Some(v) = violation {
        TlcVerdict::Violation(v)
    } else if spec_error {
        TlcVerdict::SpecError
    } else if completed_ok {
        TlcVerdict::CompletedOk
    } else {
        TlcVerdict::Unknown
    };

    TlcAnalysis {
        stats,
        verdict,
        trace,
        error_lines,
    }
}

/// Render a bounded counter-example excerpt: first [`TRACE_HEAD`] and
/// last [`TRACE_TAIL`] states with an elision marker, closed by a
/// three-part line pointing at the exact reproduce command.
pub fn render_trace_excerpt(trace: &[String], reproduce: &str) -> String {
    let mut out = String::from("Counter-example trace:\n");
    if trace.len() <= TRACE_HEAD + TRACE_TAIL {
        for state in trace {
            out.push_str(state);
            out.push('\n');
        }
    } else {
        for state in &trace[..TRACE_HEAD] {
            out.push_str(state);
            out.push('\n');
        }
        out.push_str(&format!(
            "… {} states elided …\n",
            trace.len() - TRACE_HEAD - TRACE_TAIL
        ));
        for state in &trace[trace.len() - TRACE_TAIL..] {
            out.push_str(state);
            out.push('\n');
        }
    }
    out.push_str(&format!(
        "The trace above is truncated. It shows how the model reaches the violating state. Re-run `{reproduce}` for the full counter-example."
    ));
    out
}

fn is_state_header(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("State ") else {
        return false;
    };
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    !digits.is_empty() && rest[digits.len()..].starts_with(':')
}

/// Parse "`N states generated, M distinct states found, K states left
/// on queue.`" — commas inside numbers are tolerated.
fn parse_stats_line(line: &str) -> Option<ModelCheckDetail> {
    let number_before = |marker: &str| -> Option<u64> {
        let idx = line.find(marker)?;
        let head = &line[..idx];
        let token = head.split_whitespace().last()?;
        token.replace(',', "").parse().ok()
    };
    Some(ModelCheckDetail {
        states_generated: number_before(" states generated")?,
        distinct_states: number_before(" distinct states found")?,
        queue_remaining: number_before(" states left on queue")?,
        violation: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASS_TRANSCRIPT: &str = "\
TLC2 Version 2.19 of 08 August 2024
Running breadth-first search Model-Checking with fp 105 and seed -1234 with 4 workers.
Parsing file /specs/Scheduler.tla
Semantic processing of module Scheduler
Starting... (2026-07-02 10:00:00)
Computing initial states...
Finished computing initial states: 3 distinct states generated.
Model checking completed. No error has been found.
  Estimates of the probability that TLC did not check all reachable states
2048 states generated, 512 distinct states found, 0 states left on queue.
The depth of the complete state graph search is 14.
Finished in 3s at (2026-07-02 10:00:03)
";

    const INVARIANT_TRANSCRIPT: &str = "\
TLC2 Version 2.19 of 08 August 2024
Parsing file /specs/Lock.tla
Error: Invariant MutualExclusion is violated.
Error: The behavior up to this point is:
State 1: <Initial predicate>
/\\ owner = NoThread
/\\ waiting = {}

State 2: <Acquire line 22, col 3 to line 25, col 20 of module Lock>
/\\ owner = t1
/\\ waiting = {}

State 3: <Acquire line 22, col 3 to line 25, col 20 of module Lock>
/\\ owner = t2
/\\ waiting = {t1}

142 states generated, 37 distinct states found, 11 states left on queue.
";

    const DEADLOCK_TRANSCRIPT: &str = "\
TLC2 Version 2.19 of 08 August 2024
Error: Deadlock reached.
Error: The behavior up to this point is:
State 1: <Initial predicate>
/\\ x = 0

State 2: <Next line 9, col 9 to line 9, col 30 of module Dead>
/\\ x = 1

7 states generated, 5 distinct states found, 0 states left on queue.
";

    const LIVENESS_TRANSCRIPT: &str = "\
Error: Temporal properties were violated.
Error: The following behavior constitutes a counter-example:
State 1: <Initial predicate>
/\\ p = FALSE
";

    const PARSE_ERROR_TRANSCRIPT: &str = "\
TLC2 Version 2.19 of 08 August 2024
Parsing file /specs/Broken.tla
***Parse Error***
Was expecting \"==== or more module body\"
Encountered \"Invarian\" at line 14, column 1
Fatal errors while parsing TLA+ spec in file Broken
";

    #[test]
    fn pass_transcript_yields_stats_and_ok() {
        let a = analyze(PASS_TRANSCRIPT);
        assert_eq!(a.verdict, TlcVerdict::CompletedOk);
        let stats = a.stats.expect("stats parsed");
        assert_eq!(stats.states_generated, 2048);
        assert_eq!(stats.distinct_states, 512);
        assert_eq!(stats.queue_remaining, 0);
        assert!(a.trace.is_empty());
    }

    #[test]
    fn invariant_transcript_yields_violation_with_trace() {
        let a = analyze(INVARIANT_TRANSCRIPT);
        match &a.verdict {
            TlcVerdict::Violation(v) => {
                assert_eq!(v.kind, ViolationKind::Invariant);
                assert_eq!(v.property.as_deref(), Some("MutualExclusion"));
                assert_eq!(v.trace_states, 3);
            }
            other => panic!("expected violation, got {other:?}"),
        }
        assert_eq!(a.trace.len(), 3);
        assert!(a.trace[0].contains("owner = NoThread"));
        assert!(a.trace[2].contains("waiting = {t1}"));
        let stats = a.stats.expect("stats parsed even on failure");
        assert_eq!(stats.states_generated, 142);
        assert_eq!(stats.queue_remaining, 11);
    }

    #[test]
    fn deadlock_transcript_yields_deadlock() {
        let a = analyze(DEADLOCK_TRANSCRIPT);
        match &a.verdict {
            TlcVerdict::Violation(v) => {
                assert_eq!(v.kind, ViolationKind::Deadlock);
                assert_eq!(v.property, None);
                assert_eq!(v.trace_states, 2);
            }
            other => panic!("expected deadlock, got {other:?}"),
        }
    }

    #[test]
    fn liveness_transcript_yields_liveness() {
        let a = analyze(LIVENESS_TRANSCRIPT);
        match &a.verdict {
            TlcVerdict::Violation(v) => assert_eq!(v.kind, ViolationKind::Liveness),
            other => panic!("expected liveness, got {other:?}"),
        }
    }

    #[test]
    fn parse_error_transcript_yields_spec_error_with_lines() {
        let a = analyze(PARSE_ERROR_TRANSCRIPT);
        assert_eq!(a.verdict, TlcVerdict::SpecError);
        assert!(a.error_lines.iter().any(|l| l.contains("Parse Error")));
        assert!(a.error_lines.iter().any(|l| l.contains("Was expecting")));
    }

    #[test]
    fn garbage_yields_unknown() {
        let a = analyze("something completely unrelated\nnope\n");
        assert_eq!(a.verdict, TlcVerdict::Unknown);
        assert!(a.stats.is_none());
    }

    #[test]
    fn stats_tolerate_thousands_separators() {
        let a = analyze(
            "1,048,576 states generated, 262,144 distinct states found, 1,024 states left on queue.\n",
        );
        let stats = a.stats.expect("stats");
        assert_eq!(stats.states_generated, 1_048_576);
        assert_eq!(stats.distinct_states, 262_144);
        assert_eq!(stats.queue_remaining, 1_024);
    }

    #[test]
    fn exit_codes_classify_per_tla2tools() {
        assert_eq!(violation_kind_from_exit(Some(0)), None);
        assert_eq!(
            violation_kind_from_exit(Some(10)),
            Some(ViolationKind::Assumption)
        );
        assert_eq!(
            violation_kind_from_exit(Some(11)),
            Some(ViolationKind::Deadlock)
        );
        assert_eq!(
            violation_kind_from_exit(Some(12)),
            Some(ViolationKind::Invariant)
        );
        assert_eq!(
            violation_kind_from_exit(Some(13)),
            Some(ViolationKind::Liveness)
        );
        assert_eq!(violation_kind_from_exit(Some(255)), None);
        assert_eq!(violation_kind_from_exit(None), None);
    }

    #[test]
    fn short_trace_renders_in_full() {
        let trace = vec!["State 1: a".to_string(), "State 2: b".to_string()];
        let excerpt = render_trace_excerpt(&trace, "tlc Lock.tla");
        assert!(excerpt.contains("State 1: a"));
        assert!(excerpt.contains("State 2: b"));
        assert!(!excerpt.contains("elided"));
        assert!(excerpt.contains("tlc Lock.tla"));
    }

    #[test]
    fn long_trace_is_bounded_with_elision() {
        let trace: Vec<String> = (1..=20).map(|i| format!("State {i}: s{i}")).collect();
        let excerpt = render_trace_excerpt(&trace, "tlc Big.tla");
        assert!(excerpt.contains("State 1: s1"));
        assert!(excerpt.contains("State 4: s4"));
        assert!(!excerpt.contains("State 5: s5"));
        assert!(excerpt.contains("… 14 states elided …"));
        assert!(excerpt.contains("State 19: s19"));
        assert!(excerpt.contains("State 20: s20"));
    }
}
