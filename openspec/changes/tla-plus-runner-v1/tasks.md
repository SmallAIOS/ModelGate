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
