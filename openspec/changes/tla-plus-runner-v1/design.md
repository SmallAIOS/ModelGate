# Design: tla-plus-runner-v1

## Context

`smctl-verify` dispatches four domains through a `Verifier` trait; Cedar runs in-process, while TLA+/Lean/SPIN share `shell.rs`, which spawns `<binary> <run_args> <source>` via `Command::status()` — child stdio is **inherited**, exit code is the only signal. `tla.rs` is a 37-line const (`binary: "tlc"`, `run_args: []`). The capability spec already requires a `tool_missing` JSON envelope with top-level `error`/`tool` fields, which the CLI does not emit (it prints only the `VerifyReport`); the sibling `smctl quality` verbs do emit that envelope. The real corpus is SmallAIOS `formal/tla`: 28 `.tla` + 26 same-named `.cfg`. On this machine no `tlc`, `lean`, or `spin` exists; Java 11 (Corretto) is present.

## Goals / Non-Goals

**Goals:**

- `smctl verify model` performs real TLC model checking with parsed, structured results: statistics on pass, violated-property name plus a bounded trace excerpt on failure.
- Discovery works the way TLA+ is actually distributed: PATH `tlc` or `java -jar tla2tools.jar`.
- Child tool output is captured everywhere in `shell.rs`, keeping piped stdout valid JSON for all shell verifiers.
- The `tool_missing` envelope conformance gap closes for every verify subcommand.
- The whole TLC path is testable without a JVM.

**Non-Goals:**

- Lean and SPIN deep parsing (`lean-proof-runner-v1`, `spin-protocol-runner-v1`). They only inherit output capture.
- Auto-discovery of `.tla` files (v2 concern per formal-methods-v1).
- TLC `-tool` mode message-code parsing (see Decisions).
- Wiring SmallAIOS CI to run `smctl verify model` — that lands in the kernel repo.

## Decisions

**Parse TLC's human text, not `-tool` mode message codes.** TLC's plain output lines ("`Error: Invariant TypeOK is violated.`", "`2048 states generated, 512 distinct states found, 0 states left on queue.`", numbered `State N:` trace blocks, "`Model checking completed. No error has been found.`") have been stable across TLC2 releases for years and map one-to-one onto what we report. `-tool` mode's `@!@!@STARTMSG <code>` framing is more machine-oriented but its message-code table is an implementation detail of tla2tools that would need pinning and re-verification per release; the text patterns are the de facto interface every TLA+ CI harness greps. A conservative fallback covers drift: if the process fails but no pattern matches, outcome is still `failed` (exit code is ground truth) with `SMCTL-0506 VerifyOutputUnparsed` at Warning and the first raw lines quoted in the diagnostic.

**Exit-code fallback mapping.** TLC2's documented exit statuses: 0 success; 10 assumption violation; 11 deadlock; 12 safety violation; 13 liveness violation; anything else is a tool/spec error. Text evidence wins when present; the code classifies the failure when text parsing comes up empty. The mapping lives in one function with the TLC `ExitStatus` reference cited.

**Discovery chain: PATH `tlc`, then `java` + jar.** Resolution order: (1) `tlc` on PATH — unchanged fast path; (2) jar from `[verify.model] jar`, then the `TLA2TOOLS_JAR` environment variable, run as `java -XX:+UseParallelGC -jar <jar>` after probing `java -version`; (3) `tool_missing`, whose install hint now names all three options (tlc, jar + env var, jar + workspace field). The envelope's `tool` field stays `"tlc"` per the existing spec scenario. No filesystem guessing of jar locations — explicit configuration only, consistent with formal-methods-v1's "the `[verify]` workspace section drives selection" stance.

**Invocation shape.** `current_dir` = the spec file's parent (TLA+ module resolution and the same-named-`.cfg` convention both depend on it); bare filename as the argument; explicit `-config <stem>.cfg` when the sibling exists (silence over convention-magic); `-workers auto` unless `[verify.model] workers = N`; `-metadir <tempdir>` so TLC's `states/` scratch lands in a per-run temp directory instead of the operator's repo. `-cleanup` is not used — the metadir tempdir is dropped wholesale.

**`SourceRow.detail` is a typed, optional struct.** `ModelCheckDetail { states_generated, distinct_states, queue_remaining, violation: Option<Violation { kind: invariant|deadlock|liveness|assumption, property: Option<String>, trace_states: usize }> }`, `#[serde(skip_serializing_if = "Option::is_none")]` on the row field. Additive: Cedar/Lean/SPIN rows simply omit it, existing JSON consumers see no change, MCP gets structure for free. The alternative — encoding stats only into the human `note` string — would force MCP clients to re-parse prose.

**Trace excerpts are bounded.** Human diagnostics render at most the first 4 and last 2 trace states with an elision marker and a three-part closing line telling the operator how to re-run TLC directly for the full trace. `detail.violation.trace_states` carries the true count. Full traces can reach thousands of states; a report object is not the place for them.

**Injectable binary via environment override.** `Shell` gains `env_override: Option<&'static str>` — `SMCTL_VERIFY_TLC_BIN` for the model domain (Lean/SPIN get theirs in their own follow-ups). When set, discovery and invocation use that path verbatim. This is the smallest hook that lets integration tests substitute a fixture script that replays canned TLC transcripts; documented as test-and-escape-hatch only. Alternative rejected: making the whole `Shell` const injectable through `VerifyContext` — more surface, same test power.

**Envelope emission mirrors `smctl quality`.** On `tool_missing` in JSON mode, the CLI prints `{"error": "tool_missing", "tool": "<binary>", "install_hint": "<hint>"}` (the shape the spec scenario pins) instead of the bare report, exits 0 unless `--strict` — matching both the capability spec and the sibling command family's established shape. Human mode keeps the current three-part message.

**MSGID allocation.** `SMCTL-0505 VerifyCounterExample` (Error) fires once per violated source with the property name in the structured fields; `SMCTL-0506 VerifyOutputUnparsed` (Warning) fires when the fallback path engages. Range stays within the reserved 0500..0599; the smctl-log catalog (authoritative per CLAUDE.md) gains both rows.

## Risks / Trade-offs

- [TLC text output drifts in a future tla2tools release] → Conservative fallback: exit code still classifies pass/fail, 0506 warns that parsing degraded, first raw lines are quoted. Parser fixtures pin the currently supported shapes.
- [No TLC on dev/CI machines makes the deep path untestable] → All pipeline tests run against fixture scripts via `SMCTL_VERIFY_TLC_BIN`; parser unit tests run on canned transcripts; nothing in CI needs a JVM.
- [Capturing output changes UX for Lean/SPIN (operators previously saw raw tool spew)] → Their failure notes already embed a re-run command; captured stderr's first lines are appended to the failure note so signal is not lost. Net improvement: piped JSON stops being corrupted.
- [`-workers auto` nondeterminism in fixtures] → Fixture scripts ignore arguments; parser tests use static transcripts.

**Accepted limitations (recorded during review).** (1) `verify discover` cannot see `[verify.model] jar` — the `Verifier::discover` trait method takes no context — so a host configured solely via the workspace jar shows `model` as not installed in the discover listing even though `verify model` runs; the run path is authoritative. (2) The `tool_missing` JSON envelope is a CLI contract; smctl-mcp continues to return the bare `VerifyReport`, matching its existing behavior for all outcomes. (3) In human mode, Lean/SPIN success output is no longer streamed (captured instead); their failure notes carry the captured head, and deep rendering arrives in their own follow-up changes. (4) An invariant violated by the initial state yields `trace_states: 0` with no excerpt — TLC prints that state without a `State N:` header; acceptable until a real-world case motivates parsing it.

## Migration Plan

Single PR from `change/tla-plus-runner-v1` into `develop`. Additive JSON fields only; `VerifyReport` envelope unchanged. Rollback is a revert.

## Open Questions

None.
