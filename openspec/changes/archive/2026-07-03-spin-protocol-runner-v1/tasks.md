# Tasks: spin-protocol-runner-v1

## 1. Detail type generalization

- [x] 1.1 Introduce `#[serde(untagged)] enum VerifyDetail { Model(ModelCheckDetail), Protocol(ProtocolCheckDetail) }`; `SourceRow.detail` becomes `Option<VerifyDetail>`; add `ProtocolCheckDetail { states_stored, states_matched, depth_reached, violation: Option<ProtocolViolation { kind, detail, trail_steps }> }` with `kind ∈ {assertion, acceptance_cycle, invalid_end_state}`
- [x] 1.2 Serde tests: model rows serialize byte-identically to the pre-enum shape; protocol rows round-trip; deserialization disambiguates on field names

## 2. Pan output parser (pan.rs)

- [x] 2.1 Parse: `errors: N` (ground truth), `assertion violated`, `acceptance cycle`, `invalid end state`, `N states, stored`, `M states, matched`, `depth reached D`
- [x] 2.2 Trail-listing parser for `spin -t -p` output (step lines) and a bounded excerpt renderer (first 4 + last 2 + elision + three-part reproduce line)
- [x] 2.3 Fixture transcripts: pass-with-stats, assertion violation, acceptance cycle, invalid end state, unparseable garbage; unit tests over each

## 3. SPIN runner (spin.rs)

- [x] 3.1 Discovery: `spin -V` via `SMCTL_VERIFY_SPIN_BIN` override or PATH (existing Shell probe); separate `cc --version` probe honoring `SMCTL_VERIFY_PAN_CC`
- [x] 3.2 Missing `cc` yields the `tool_missing` envelope with `tool: "cc"` and the CLT/build-essential hint; missing `spin` keeps `tool: "spin"`
- [x] 3.3 Per-source pipeline in a tempdir: `spin -a <abs spec>` (fail fast on syntax error, quote output), `cc -O2 -o pan pan.c` (fail with captured stderr head), `./pan -a` (capture + parse)
- [x] 3.4 Trail replay when `.trail` exists: `spin -t -p <abs spec>` in the same tempdir, bounded excerpt into report diagnostics; emit SMCTL-0505 on violations, SMCTL-0506 on unparsed failures
- [x] 3.5 Reproduce hints quote spaced paths and give the generate/compile/run sequence plus the trail-replay command

## 4. Integration tests

- [x] 4.1 Fixture scripts: fake `spin` (writes pan.c; prints canned trail on `-t`), fake `cc` (writes executable `pan` script emitting canned pan output)
- [x] 4.2 Tests: pass path with stats in JSON; assertion violation with trail excerpt and exit 1; cc-missing envelope (`tool: "cc"`); spin syntax error fails at generation; no `pan.*` artifacts left outside the tempdir (assert repo dir clean after run)

## 5. Validation and PR

- [x] 5.1 `openspec validate --all --strict` passes
- [x] 5.2 `cargo fmt`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` all pass
- [x] 5.3 Open PR from `change/spin-protocol-runner-v1` into `develop`; CI Gate green; adversarial review pass on the diff — PR #34, CI green three times (initial + two review-fix waves), 9 review-confirmed defects fixed, squash-merged 2026-07-03

## 6. Adversarial review fixes (empirically verified against SPIN 6.5.2)

- [x] 6.1 Trail replay fixed: pan writes `<basename>.trail` into the workdir while the spec path is absolute, so `spin -t -p <abs>` reported "cannot find trail file" — replay now passes `-k <trailfile>` (verified against real spin; the fixture had masked this)
- [x] 6.2 Depth-truncated searches (`Search not completed` / `max search depth too small`) with `errors: 0` now report failed-as-inconclusive with a `-m<N>` remediation instead of a false `passed`
- [x] 6.3 Failure-class matchers restricted to pan-prefixed lines — the search-for banner's "invalid end states +" no longer misclassifies unrecognized failures as deadlocks
- [x] 6.4 End-to-end validated with real spin + clang: assertion counter-examples caught with replayed trails, clean model passes with parsed stats

## 7. Adversarial review fixes, wave 2

- [x] 7.1 `states, stored (N visited)` acceptance-run suffix parses (containment, not suffix match); pan's `%g` scientific-notation counts on large runs parse correctly
- [x] 7.2 Trail-step extraction stops at `spin: trail ends` — the final state dump no longer inflates `trail_steps` (real-spin E2E: count corrected from 3 to 2)
- [x] 7.3 Relative `SMCTL_VERIFY_SPIN_BIN` / `SMCTL_VERIFY_PAN_CC` overrides anchored via shared `shell::anchor_override` (same class the TLA review confirmed); `cc_available` now requires exit success, catching the macOS no-CLT `/usr/bin/cc` shim
- [x] 7.4 `no sources configured` short-circuits before tool probes in both protocol and model verbs — cc-less hosts with unconfigured workspaces stay benign
- [x] 7.5 Unparseable pan output is never a pass (pan exits 0 even on violations); trail-excerpt reproduce is the full runnable chain; violation note reworded to satisfy the lowercase-tool-name voice rule
