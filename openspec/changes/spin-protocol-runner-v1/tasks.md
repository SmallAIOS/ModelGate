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
- [ ] 5.3 Open PR from `change/spin-protocol-runner-v1` into `develop`; CI Gate green; adversarial review pass on the diff
