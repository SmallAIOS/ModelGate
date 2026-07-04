# Design — lean-proof-runner-v1

## Context

`smctl verify proof` today is a 37-line `Shell` config (`smctl-verify/src/lean.rs`): probe `lake --version`, glob `[verify.proof] roots`, spawn `lake build <matched-path>` from smctl's cwd, map exit code to passed/failed. Two prior changes established the deep-runner pattern this change follows: `tla-plus-runner-v1` (discovery chain, env override, captured output, text parsing into `SourceRow.detail`, three-part diagnostics, hermetic fake-tool tests) and `spin-protocol-runner-v1` (per-source workdir hygiene, multi-tool missing-tool attribution, exit-success probe gating).

Two facts about the real deployment drive the design. First, the SmallAIOS kernel proof corpus is `formal/lean4/` — six loose, self-contained `.lean` files with no `lakefile.lean`, `lakefile.toml`, or `lean-toolchain` anywhere; the shipped `lake build` invocation is simply wrong for it. Second, one corpus file carries a live `sorry` (`CapabilityNonForgery.lean:144`), and neither `lean` nor `lake` reflects warnings in the exit code — the current wrapper reports that file as verified.

Toolchain reality: `lean --json` (available since Lean 4.8) emits one JSON message object per line on stdout with `severity`, `pos`, `endPos`, `fileName`, `data`, and (since 4.15) `kind`, where incomplete proofs are tagged `hasSorry` (since 4.19). `lake build` has no JSON log mode in any version; its text log replays compiler messages as `{level}: {file}:{line}:{col}: {msg}`. elan installs PATH shims that spawn successfully but exit non-zero when no toolchain is configured, and resolves toolchains by walking up from the process cwd, not from the target file's directory.

## Goals / Non-Goals

**Goals:**

- Correct invocation for both corpus shapes: Lake packages and loose `.lean` trees, classified automatically per configured root.
- A proof containing `sorry` is not verified: incomplete proofs fail their source row with evidence.
- Structured `SourceRow.detail` parity with the TLA+/SPIN runners in `--json` output.
- Three-part failure diagnostics with bounded excerpts and executable reproduce commands.
- Hermetic testability without a Lean toolchain installed (env overrides + fake scripts, as TLA+/SPIN).

**Non-Goals:**

- `#print axioms` / `sorryAx` auditing (covers the suppressed-warning corner where errors coexist with sorries; those rows already fail on the errors). Deferred to a follow-up.
- Timeout enforcement (TLA+/SPIN precedent: none; recorded as an accepted limitation).
- `workspace.toml` schema changes: `roots` / `fail_on` keep their shape; `fail_on` remains parsed-but-unread everywhere, and this change does not invent semantics for it.
- Streaming tool output (captured, per the shared verify contract).

## Decisions

**D1 — Root classification is automatic, not configured.** A root directory containing `lakefile.lean` or `lakefile.toml` (Lake's own detection triggers, checked in Lake's preference order) is a Lake package; any other directory is a loose-file tree; a glob pattern that matches files yields loose-file rows directly. *Alternative rejected:* a `[verify.proof] kind = "lake"|"loose"` field — schema churn behind `deny_unknown_fields` for something the filesystem already knows, and the shipped spec example (`roots = ["formal/lean"]`) stays valid unchanged.

**D2 — Loose files are the structured backbone, checked per file with `lean --json`.** Each `.lean` file under a loose root (recursive walk, skipping hidden directories and any `.lake/`) becomes one `SourceRow`, checked by `lean --json <abs-file>` with cwd set to the root (elan resolves `lean-toolchain` by walking up from cwd, so cwd placement makes toolchain resolution follow the corpus, not smctl's incidental location). Stdout is parsed line-by-line as JSON message objects; non-JSON lines are tolerated and retained for the unparsed-output fallback. *Alternatives rejected:* `lake env lean` (requires a Lake workspace and pre-built imports — loose trees have neither); plain-text parsing as primary (the JSON mode exists precisely for tools like this; text remains the fallback).

**D3 — Lake packages are checked per package with `lake build`, text-parsed.** One `SourceRow` per package root, cwd = the package root, parsing replayed compiler messages (`{level}: {file}:{line}:{col}: {msg}`), the `Build completed successfully` line, and the failed-targets list. *Alternative rejected:* per-file `lake lean <f> -- --json` rows inside packages — requires target mapping, rebuilds per file, and Lake's own log already replays every message; per-package granularity matches how operators fix Lake builds.

**D4 — Sorry means failed.** Classification per message: `kind == "hasSorry"`, or message text containing `declaration uses 'sorry'` (≤ 4.26) or ``declaration uses `sorry` `` (4.27+). Any sorry, or any error-severity message, fails the row; the runner never relies on exit codes for the incomplete case and never relies on `-E hasSorry` promotion (version-gated ≥ 4.19; real corpora pin older toolchains, e.g. 4.14). Rationale: the verifier's contract is *proven*; a `sorry` is direct evidence the theorem is admitted, not proved. This flips the real kernel corpus row for `CapabilityNonForgery.lean` from passed to failed — which is the truth the current wrapper hides.

**D5 — `VerifyDetail::Proof(ProofCheckDetail)` with required count fields.** `ProofCheckDetail { errors: usize, warnings: usize, sorries: usize, failure: Option<ProofFailure> }`, `ProofFailure { kind: error|sorry|build, location: Option<String>, message: String }`. The three counts are required and their names are disjoint from both existing untagged variants' required fields (`states_generated`/`distinct_states`/`queue_remaining`, `states_stored`/`states_matched`/`depth_reached`), so untagged deserialization stays unambiguous and an empty object can never match the Proof variant. Human output flows through note text and diagnostics (the renderer is detail-blind by design, per precedent).

**D6 — Discovery probes require exit success, and `discover` probes `lean`.** `discover_binary`'s any-spawn-is-Found behavior is wrong for elan shims (spawn fine, exit non-zero, error text as "version"); the proof prober gates on `status.success()` exactly as SPIN's `cc_available()` does for the macOS `cc` shim. `smctl verify discover` switches the proof row from `lake` to `lean`: it is the foundational checker in this design, its `--version` string carries the Lean version (Lake's has to be dug out of parentheses), and the two ship together in every elan toolchain so a single probe answers the install question either way. At run time each root checks the tool it actually needs (`lean` for loose trees, `lake` for packages); `missing_tool_for_proof` names the specific missing binary in the `tool_missing` envelope, following SPIN's spin-vs-cc helper.

**D7 — Env overrides `SMCTL_VERIFY_LEAN_BIN` and `SMCTL_VERIFY_LAKE_BIN`.** Anchored via `shell::anchor_override` like `SMCTL_VERIFY_TLC_BIN` / `SMCTL_VERIFY_SPIN_BIN` / `SMCTL_VERIFY_PAN_CC`. All integration tests inject `#!/bin/sh` fakes through them; no test requires a Lean toolchain.

**D8 — MSGIDs: allocate SMCTL-0507 ProofIncomplete (Error); reuse 0505 and 0506.** Evidenced proof *errors* emit `SMCTL-0505 VerifyCounterExample` (the failing message plus location is the evidence, same class as a TLC violation or pan trail); sorries emit the new `SMCTL-0507 ProofIncomplete` rather than 0505 — an admitted theorem is not a counter-example, and operators filtering 0505 for counter-witnesses should not receive incompleteness events. Non-zero exits with no parseable message reuse `SMCTL-0506 VerifyOutputUnparsed` (Warning). Severity for 0507 is Error under the established rule (artifact definitively fails its contract, with evidence). The 0507 row lands in the `smctl-verify` capability spec's MSGID requirement (the live catalog for the 05xx range) plus `smctl-log/src/msgid.rs`; the archived logging spec is immutable and untouched.

**D9 — Row expansion lives in `lean.rs`, not `shell.rs`.** `walk_sources`' contract is one row per glob match (`&dyn Fn(&Path) -> SourceRow`); loose-tree roots need one row per file discovered *under* a match. The Lean runner implements its own walk (same repo-iteration and glob semantics, flat-mapping directory matches into per-file rows) rather than changing the shared closure signature under the other three verifiers. `shell.rs` keeps `anchor_override`, `sh_quote`, and `output_head` reuse.

**D10 — Fail closed on everything the walk cannot vouch for** *(added after adversarial review)*. Five review findings shared one root cause: paths where the runner could stay silent while proofs went unchecked. All now fail loudly: (a) a mixed corpus with one tool absent runs the checkable targets and fails each unverifiable row naming the missing tool — whole-run `tool_missing` only when nothing can be checked; (b) unreadable directories push a fatal diagnostic instead of being skipped; (c) a root that classifies to nothing checkable (corpus moved, only hidden files) fails the run; (d) nested Lake packages inside loose trees become `lake build` targets rather than mis-checked bare files, and direct file matches inside a package resolve to the enclosing package (deduplicated); (e) lake's closing `error: build failed` meta line is excluded from message parsing, and position-less error lines classify as `build` (environmental) rather than `error` (proof evidence), so SMCTL-0505 fires only on real proof errors. A canonical-path visited set guards symlink cycles in the loose walk. The non-UTF-8 pattern skip stays non-fatal, matching `shell::walk_sources`. As a drive-by from the same hazard class, `spin::cc_available` now requires probe exit success, which its own doc comment already claimed.

## Risks / Trade-offs

- [Lake text-log format drifts across versions] → parse leniently (message-line regex only), keep the exit code as ground truth, and fall back to SMCTL-0506 with an `output_head` excerpt when nothing matches.
- [Sorry warning spelling changes (4.27 switches to backticks)] → match the `hasSorry` kind tag first, both text spellings second; parser unit tests pin all three forms.
- [JSON messages without `kind` (Lean < 4.15)] → sorry classification falls back to message text; tests cover kind-less messages.
- [elan auto-downloads a missing toolchain on first invocation (network, minutes of latency, stderr noise)] → accepted limitation, documented; stderr is never parsed as messages on the JSON path, and hermetic tests never reach a real toolchain.
- [Corpus without `lean-toolchain` resolves the default toolchain silently] → accepted limitation; the resolved `lean --version` already surfaces in `verify discover`.
- [Untagged-enum mismatch could misroute detail objects] → required disjoint count fields plus a serde round-trip test asserting Model/Protocol/Proof discrimination.
- [Per-file `lean` startup (~200 ms) on large loose trees] → acceptable at corpus scale (six files); Lake packages batch through `lake build`; recorded as a limitation rather than optimized speculatively.

## Migration Plan

Purely additive: no `workspace.toml` change, JSON detail is `skip_serializing_if` none, MCP flows the new variant through serde automatically. Operators with Lake-package corpora see identical semantics with richer failure rows; operators with loose corpora see correct per-file checking for the first time. Rollback is reverting the change; no data or config migration in either direction.

## Open Questions

None blocking. `#print axioms` auditing and a timeout knob are explicitly deferred (see Non-Goals). One review finding is deferred to a follow-up rather than fixed here: `shell::discover_binary` still treats any successful spawn as Found (the spawn-only hazard fixed for lean and cc), because tightening it changes TLC discovery behavior (`tlc -h` exit codes vary across wrapper scripts) and deserves its own validated change.
