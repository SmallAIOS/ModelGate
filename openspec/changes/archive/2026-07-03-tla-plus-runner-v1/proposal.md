# Proposal: tla-plus-runner-v1

## Why

`smctl verify model` is the exit-code shell-out wrapper `formal-methods-v1` explicitly deferred ("counter-example rendering belongs in `tla-plus-runner-v1`"). Today it cannot serve real operators: it requires a `tlc` binary on PATH (no `java -jar tla2tools.jar` fallback, though TLC is a JVM tool), ignores TLC's `.cfg` model configs, discards everything TLC prints — state counts, invariant violations, counter-example traces — and lets the child's inherited stdout interleave with smctl's own, which corrupts the piped-JSON contract the capability spec mandates. Meanwhile the SmallAIOS kernel repo carries the largest formal corpus of the three deferred tools: 28 `.tla` models with 26 paired `.cfg` configs waiting to be checked.

## What Changes

- Capture child stdout/stderr in the shared shell-out engine (`Command::output()` instead of inherited stdio) so tool output never leaks into smctl's stdout — fixes the latent piped-JSON violation for all shell verifiers, not just TLA+.
- TLC discovery chain: `tlc` on PATH, else `java` plus `tla2tools.jar` resolved from `[verify.model] jar` or the `TLA2TOOLS_JAR` environment variable; only when neither works does the verb report `tool_missing`.
- Correct TLC invocation: run in the spec file's directory (module resolution and the same-named-`.cfg` convention), pass `-config <name>.cfg` when the sibling config exists, `-workers auto`, and point `-metadir` at a temp directory so TLC's `states/` scratch never pollutes the target repo.
- Parse TLC output into structure: states generated / distinct states / queue depth on success; invariant, deadlock, and liveness violations with the violated property name and a capped counter-example trace excerpt on failure; conservative exit-code fallback when text parsing fails.
- Extend `SourceRow` with an optional `detail` object (model-check statistics and violation summary) — additive, flows to MCP consumers automatically.
- Emit the spec-mandated `tool_missing` JSON envelope (`error`, `tool`, install hint fields) from every verify subcommand — closing an existing conformance gap where only the report object is printed today.
- New MSGIDs from the reserved verify range: `SMCTL-0505 VerifyCounterExample` (Error), `SMCTL-0506 VerifyOutputUnparsed` (Warning).
- `[verify.model]` gains optional `jar` and `workers` fields (still `deny_unknown_fields`).
- Hermetic test harness: injectable verifier binary override so fake-`tlc` fixture scripts exercise the full pipeline without a JVM, plus parser unit tests over canned TLC transcripts.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `smctl-verify`: `model` deepens from exit-code wrapper to parsed model checking (new requirement); verifier-missing detection now accounts for the jar fallback chain; the `[verify.model]` schema grows `jar`/`workers`; the MSGID allocation adds 0505/0506; a new requirement pins captured (never interleaved) tool output.

## Impact

- `smctl-verify` crate: `shell.rs` (output capture, injectable binary), `tla.rs` (becomes a real runner), new `tlc.rs` parser module, `lib.rs` (`SourceRow.detail`).
- `smctl-workspace`: `ModelVerifierSection` gains `jar`/`workers`.
- `smctl-log`: two new `MsgId` variants in the 0500 range.
- `smctl/src/main.rs`: `tool_missing` envelope on verify verbs; human rendering of model-check statistics.
- Tests: new fixtures (fake tlc scripts, canned TLC transcripts); existing verify tests unaffected in behavior for Cedar.
- Lean and SPIN wrappers: unchanged semantics, but they inherit output capture (their raw output stops leaking into piped JSON).
