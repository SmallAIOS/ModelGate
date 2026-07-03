# Proposal: spin-protocol-runner-v1

## Why

`smctl verify protocol` currently tells the most misleading lie in the verify suite: it runs `spin -a <spec.pml>`, which only *generates* the pan.c verifier source — it never compiles or runs pan, so `passed` means "Promela parsed", not "protocol verified". Worse, the generated `pan.*` artifacts land in whatever directory smctl was invoked from, polluting the operator's repo. `formal-methods-v1` explicitly deferred the real pipeline and counter-example rendering to this change. The SmallAIOS kernel repo carries 7 Promela models across `formal/spin/` and `formal/promela/` (TCP state machine, QUIC handshake, pub/sub routing, scheduler fairness, lockout, inference pipeline) waiting to actually be verified.

## What Changes

- `smctl verify protocol` runs the full SPIN pipeline per source: `spin -a` (codegen) → `cc -o pan pan.c` (compile) → `./pan -a` (verify, with acceptance-cycle detection for LTL liveness), all inside a per-run temp work directory so `pan.*` artifacts never touch the operator's repo.
- Parse pan's output into structure: verdict (`errors: 0` pass; assertion violation, acceptance cycle, invalid end state on failure), state-space statistics (states stored/matched, depth reached), and the error count.
- Counter-example rendering: when pan writes a `.trail` file, replay it with `spin -t -p` and render a bounded excerpt (same first-4/last-2 shape as the TLA+ runner) plus the exact reproduce commands.
- `cc` discovery: the compile step probes for a C compiler; a missing compiler reports the `tool_missing` envelope with `tool: "cc"` and a platform install hint, distinct from missing `spin`.
- Reuse the existing verify MSGIDs: `SMCTL-0505 VerifyCounterExample` on a violated source, `SMCTL-0506 VerifyOutputUnparsed` when pan output matches no known pattern — no new allocations.
- Extend `SourceRow.detail` to carry protocol statistics via an untagged detail enum — model rows serialize exactly as before; protocol rows gain `states_stored`, `states_matched`, `depth_reached`, and `violation`.
- Hermetic test harness: `SMCTL_VERIFY_SPIN_BIN` and `SMCTL_VERIFY_PAN_CC` overrides so fixture scripts fake the whole generate/compile/run pipeline without SPIN or a C compiler.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `smctl-verify`: `protocol` deepens from parse-only wrapper to full verification pipeline (new requirement); the structured `detail` object generalizes to protocol rows.

## Impact

- `smctl-verify` crate: `spin.rs` becomes a real runner; new `pan.rs` output parser; `lib.rs` detail enum (`VerifyDetail { Model, Protocol }`, untagged serde — model JSON unchanged).
- `smctl/tests/cli.rs`: hermetic pipeline tests via fixture scripts.
- No workspace-schema, MSGID, or CLI-dispatch changes.
- Lean runner: untouched (`lean-proof-runner-v1` remains the last deferred follow-up).
