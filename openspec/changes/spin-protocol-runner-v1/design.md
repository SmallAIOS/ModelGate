# Design: spin-protocol-runner-v1

## Context

`spin.rs` is a 38-line const `Shell` wrapper: `spin -a <spec>` with inherited-then-captured output and exit-code semantics. `spin -a` is codegen only — SPIN's actual verifier is the generated pan.c, which must be compiled and executed. The verify plumbing this change needs already exists from `tla-plus-runner-v1`: captured output in `shell.rs`, `walk_sources` with a per-source closure, bounded-excerpt rendering, the `tool_missing` envelope, MSGIDs 0505/0506, and the fixture-script test pattern. The real corpus is SmallAIOS `formal/spin/*.pml` (5) plus `formal/promela/*.pml` (2) — note the two directories mean `[verify.protocol] specs` needs both globs, which the existing config already supports.

## Goals / Non-Goals

**Goals:**

- `verify protocol` performs real verification: generate → compile → run pan, with parsed verdicts, statistics, and bounded counter-example excerpts.
- Zero artifacts in the operator's repo — the whole pipeline runs in a temp work directory.
- Missing `spin` and missing `cc` are distinguishable, each with an actionable hint.
- The full pipeline is testable without SPIN or a C compiler installed.

**Non-Goals:**

- Lean deep integration (`lean-proof-runner-v1`).
- pan tuning knobs (`-m` depth, `-DSAFETY` builds, multicore `-DNCORE`) in workspace config — defaults first; a config follow-up can add them when a real model needs it.
- LTL property management (`spin -f` formula compilation) — properties are expected inline in the `.pml` files, as the SmallAIOS corpus does.

## Decisions

**Pipeline shape: `spin -a` → `cc -O2 -o pan pan.c` → `./pan -a`, cwd = per-source temp dir.** `spin -a` writes `pan.[chmbt]` into its cwd, so the runner creates a `tempfile::tempdir` per source, runs all three steps inside it, and passes the spec as an absolute path. `pan -a` enables acceptance-cycle detection so inline LTL liveness properties are actually checked; plain safety models are unaffected by the flag. `-O2` keeps large state spaces tolerable without meaningfully slowing the tiny compiles.

**Verdict parsing from pan's text.** pan's summary is stable and line-structured: `State-vector X byte, depth reached N, errors: K`, `M states, stored`, `L states, matched`. Failure classes parsed: `assertion violated <expr>`, `acceptance cycle` (liveness), `invalid end state` (deadlock), `pan: elapsed time` fallbacks. `errors: 0` is the pass signal; a non-zero `errors:` count with an unrecognized class still fails correctly (count is ground truth). No known pattern at all + non-zero exit → `SMCTL-0506` with the output head quoted, mirroring the TLA+ fallback contract.

**Trail replay for counter-examples.** pan writes `<spec>.trail` next to its cwd artifacts on error. When present, the runner replays with `spin -t -p <abs spec>` (same cwd), captures the step listing, and renders the bounded first-4/last-2 excerpt via the existing `tlc::render_trace_excerpt`-style helper — steps are `proc N` transition lines rather than TLA state blocks, but the shape and elision contract are identical. The reproduce hint gives both commands (`spin -a && cc && pan -a`, then `spin -t -p`) with quoted paths.

**`cc` as a second discoverable tool.** Compile-step discovery probes `cc --version` (the POSIX alias covers clang and gcc). Missing `spin` → `tool_missing` envelope with `tool: "spin"` (unchanged). Present `spin` but missing `cc` → `tool_missing` with `tool: "cc"` and hint "install the Xcode Command Line Tools (`xcode-select --install`) or your distro's build-essential package". The generic envelope shape already supports any tool name; no spec change to the missing-tool requirement is needed — the new requirement's scenarios pin the cc case.

**`VerifyDetail` untagged enum instead of a second field.** `SourceRow.detail` becomes `Option<VerifyDetail>` with `#[serde(untagged)] enum VerifyDetail { Model(ModelCheckDetail), Protocol(ProtocolCheckDetail) }`. Untagged serde means model rows serialize byte-identically to 0.3.0 output — no consumer breakage — and protocol rows get their own field names (`states_stored`, `states_matched`, `depth_reached`, `violation { kind: assertion|acceptance_cycle|invalid_end_state, detail, trail_steps }`). Deserialization disambiguates on field names, which are disjoint. Alternative rejected: a parallel `protocol_detail` field — two optional fields where exactly one is ever set is a worse contract.

**Env overrides for hermetic tests.** `SMCTL_VERIFY_SPIN_BIN` (mirrors the TLC precedent) and `SMCTL_VERIFY_PAN_CC` for the compiler. Fixture flow: fake `spin` writes a placeholder `pan.c` (and on `-t` prints a canned trail listing); fake `cc` writes an executable `pan` script that prints a canned pan transcript. The whole pipeline — tempdir, three spawns, parsing, trail replay — runs for real in tests.

**MSGID reuse.** 0505/0506 are verify-range generic ("counter-example found", "output unparsed"), not TLA-specific; the structured fields carry `verifier = "protocol"`. No catalog change.

## Risks / Trade-offs

- [pan output format drift across SPIN versions] → `errors: N` count is the ground-truth gate; class parsing is best-effort with the 0506 fallback quoting raw output. Fixtures pin SPIN 6.x shapes.
- [Compile step is slow or fails on exotic models] → failure note quotes cc's captured stderr head with the full reproduce command; compile time is dwarfed by pan runtime on real models.
- [`pan -a` explores more than a pure-safety run needs] → correctness over speed for v1; a `[verify.protocol]` tuning follow-up can add `safety = true` if runtime hurts.
- [Trail replay doubles spawns on failure] → only on failure, bounded output, and it is exactly the counter-example the operator needs.

## Migration Plan

Single PR from `change/spin-protocol-runner-v1` into `develop`. JSON is additive (untagged detail preserves model shape). Rollback is a revert.

## Open Questions

None.
