# Tasks: tla-plus-runner-v1

## 1. Shared plumbing (shell.rs, report types)

- [x] 1.1 Switch `shell.rs` `run_one_source` from `Command::status()` to `Command::output()`; fold the leading lines of captured stderr/stdout into failure notes; keep three-part shaping
- [x] 1.2 Add `env_override: Option<&'static str>` to `Shell`; discovery and invocation honor the override path verbatim (`SMCTL_VERIFY_TLC_BIN` for model)
- [x] 1.3 Add `ModelCheckDetail`/`Violation` types and the optional `detail` field on `SourceRow` (`skip_serializing_if` none); unit-test serde shape (omitted when absent)

## 2. Workspace schema

- [x] 2.1 Add optional `jar: Option<String>` and `workers: Option<u32>` to `ModelVerifierSection` (keep `deny_unknown_fields`); thread through `manifest_from_workspace` to the model runner
- [x] 2.2 Tests: parse with/without new fields; typo (`jars`) fails at parse time

## 3. MSGIDs

- [x] 3.1 Add `MsgId::VerifyCounterExample` (505, Error) and `MsgId::VerifyOutputUnparsed` (506, Warning) to smctl-log with tests
- [x] 3.2 Catalog updated: the live smctl-log spec allocates only per-crate ranges; the per-ID verify rows live in the smctl-verify capability spec, covered by this change's delta (MSGID range allocation requirement)

## 4. TLC runner (tla.rs)

- [x] 4.1 Discovery chain: PATH `tlc` → `java -version` probe + jar from `[verify.model] jar` / `TLA2TOOLS_JAR` → `tool_missing`; install hint names all three options
- [x] 4.2 Invocation: `current_dir` = spec parent, bare filename arg, `-config <stem>.cfg` when sibling exists, `-workers auto` (or configured), `-metadir` into per-run tempdir
- [x] 4.3 Wire parsed results into `SourceRow` (`detail`, note like `passed — 2,048 states, 512 distinct`), emit SMCTL-0505/0506 where the spec says

## 5. TLC output parser (tlc.rs)

- [x] 5.1 Parse: summary stats line; `Error: Invariant <name> is violated`; deadlock; temporal/liveness violation; assumption violation; `State N:` trace blocks; completion line
- [x] 5.2 Exit-code fallback mapping (0/10/11/12/13/other) in one documented function
- [x] 5.3 Bounded trace rendering (first 4 + last 2 + elision + three-part reproduce line)
- [x] 5.4 Fixture transcripts: pass-with-stats, invariant violation with trace, deadlock, liveness, parse error, unparseable garbage; unit tests over each

## 6. CLI envelope and rendering

- [x] 6.1 Emit `{"error":"tool_missing","tool":...,"install_hint":...}` from all verify subcommands in JSON mode (mirror quality verbs); exit 0 unless `--strict`
- [x] 6.2 Human rendering for model stats and violation summaries
- [x] 6.3 Integration tests via `SMCTL_VERIFY_TLC_BIN` fixture scripts: pass path, violation path, tool-missing envelope, piped-JSON validity while the fake tool prints noise

## 7. Validation and PR

- [x] 7.1 `openspec validate --all --strict` passes
- [x] 7.2 `cargo fmt`, `cargo clippy --workspace -- -D warnings` pass; `cargo test --workspace` passes except the pre-existing `test_quality_audit_json_output_is_structurally_valid` failure whose fix ships in PR #30 (security-hygiene-v1) — green after that merges and this branch syncs with develop
- [ ] 7.3 Open PR from `change/tla-plus-runner-v1` into `develop`; CI Gate green

## 8. Adversarial review fixes (post-PR round)

- [x] 8.1 Anchor relative launcher and jar paths before TLC runs with `current_dir` = spec dir: env-override binaries containing a separator, `TLA2TOOLS_JAR`, `[verify.model] jar`, and a relative workspace root are all absolutized (confirmed medium)
- [x] 8.2 Unparsed non-zero exits now emit `SMCTL-0506` and quote the output head even when exit codes 10-13 classify the violation kind; `SMCTL-0505` is reserved for text-evidenced violations (confirmed medium — delta scenario now actually met)
- [x] 8.3 Trace excerpt footer no longer claims truncation when the full trace rendered (confirmed low)
- [x] 8.4 Jar fallback covered end-to-end by a hermetic test (fake `java` on a controlled PATH + placeholder jar) (confirmed low)
- [x] 8.5 Parser keeps single-variable state lines (`x = 0`) and wrapped values in trace blocks; TLC marker lines terminate unblanked blocks
- [x] 8.6 Bare-filename sources no longer produce an empty `current_dir`; reproduce hints drop the ephemeral `-metadir` and quote paths containing spaces; discovery probes spawn once instead of twice
