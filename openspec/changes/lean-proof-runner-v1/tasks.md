# Tasks — lean-proof-runner-v1

## 1. MSGID and shared types

- [x] 1.1 Add `MsgId::ProofIncomplete` = SMCTL-0507 (Error) to `smctl-log/src/msgid.rs`: variant, `code()`, `default_severity()`, `Display`, range test
- [x] 1.2 Add `VerifyDetail::Proof(ProofCheckDetail)` to `smctl-verify/src/lib.rs`: `ProofCheckDetail { errors, warnings, sorries, failure: Option<ProofFailure> }`, `ProofFailure { kind: error|sorry|build, location: Option<String>, message: String }`; serde round-trip test asserting untagged discrimination against Model and Protocol variants

## 2. Parser module

- [x] 2.1 Create `smctl-verify/src/lean_out.rs`: parse `lean --json` stdout line-by-line into typed messages (severity, pos, endPos, fileName, data, optional kind), tolerating non-JSON lines
- [x] 2.2 Sorry classification: `kind == "hasSorry"` first, then `declaration uses 'sorry'` and backtick spelling in message text; unit tests pin all three forms plus kind-less messages
- [x] 2.3 Parse `lake build` text logs: replayed `{level}: {file}:{line}:{col}: {msg}` lines, success line, failed-targets list; unit tests over canned transcripts (clean build, compile error, sorry replay, garbage)
- [x] 2.4 Analysis summarizer: message stream → `ProofCheckDetail` (counts, first failure with kind/location/message) with bounded excerpt rendering for diagnostics

## 3. Runner

- [x] 3.1 Root classification in `lean.rs`: lakefile.lean/lakefile.toml → Lake package; matched directory without → loose tree (recursive `.lean` discovery skipping hidden dirs and `.lake/`); file glob matches → loose rows
- [x] 3.2 Tool resolution: `SMCTL_VERIFY_LEAN_BIN` / `SMCTL_VERIFY_LAKE_BIN` overrides (anchored), PATH probe gated on exit success; `missing_tool_for_proof` helper naming lean vs lake per classified need
- [x] 3.3 Loose-file execution: `lean --json <abs-file>` with cwd = root, captured output, one SourceRow per file with detail + three-part diagnostic + reproduce command
- [x] 3.4 Lake package execution: `lake build` with cwd = package root, one SourceRow per package
- [x] 3.5 MSGID emissions: 0505 on evidenced error rows, 0507 on sorry rows, 0506 on unparsed non-zero exits
- [x] 3.6 Switch `discover()` to probe `lean` with the exit-success gate; keep elan install hint

## 4. CLI and MCP plumbing

- [x] 4.1 Wire `missing_tool_for_proof` into the `tool_missing` envelope dispatch in `smctl/src/main.rs` (pattern: protocol's spin-vs-cc at main.rs:2433-2448)
- [x] 4.2 Refresh `smctl_verify_proof` MCP tool description in `smctl-mcp/src/server.rs`

## 5. Integration tests (hermetic, no toolchain)

- [x] 5.1 Fake-lean scripts via `SMCTL_VERIFY_LEAN_BIN`: clean pass, sorry warning (exit 0 → row fails), error with position, kind-less message, garbage output with non-zero exit
- [x] 5.2 Fake-lake script via `SMCTL_VERIFY_LAKE_BIN` against a temp dir with `lakefile.toml`: clean build and failing build
- [x] 5.3 Loose-tree expansion test: temp root with nested `.lean` files (plus a `.lake/` decoy) → per-file rows
- [x] 5.4 Broken-shim discovery test: override pointing at a script that exits 1 from `--version` → discover reports not installed
- [x] 5.5 Missing-lean envelope test: loose corpus, no lean → `tool_missing` JSON names `lean`; piped-JSON purity test (whole stdout parses as one serde_json document)
- [x] 5.6 Arg/cwd capture tests: fake script records `$PWD` and `"$@"` → asserts cwd = root and `--json` + absolute file arg

## 6. Documentation

- [x] 6.1 Rewrite the capability spec Purpose paragraph (openspec/specs/smctl-verify/spec.md line 5): Cedar end-to-end, TLA+/SPIN/Lean deep, none remain exit-code wrappers
- [x] 6.2 Update README verify rows (lines ~174, ~256, ~266): loose-file and Lake semantics for `verify proof`
- [x] 6.3 Remove stale "exit-code level" language from `lean.rs` / `shell.rs` doc comments; fix the stale MSGID-catalog path in `msgid.rs` doc comment and CLAUDE.md (points at archived logging spec)

## 7. Verification

- [ ] 7.1 `cargo test --workspace`, `cargo clippy --workspace -- -D warnings`, `cargo fmt -- --check` all green
- [x] 7.2 `openspec validate lean-proof-runner-v1 --type change --strict` passes
- [x] 7.3 Live-corpus smoke run: install elan (or use an existing toolchain), point a scratch workspace.toml at the kernel's `formal/lean4/` copy, confirm the live sorry in CapabilityNonForgery.lean fails its row with kind `sorry`
