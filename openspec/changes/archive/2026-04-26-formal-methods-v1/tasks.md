# Formal methods — Tasks

## 1. Crate Setup

- [x] 1.1 Create `smctl-verify/` crate with `Cargo.toml` (cedar-policy 4.x, smctl-log, smctl-workspace deps)
- [x] 1.2 Add `smctl-verify` as a workspace member in root `Cargo.toml`
- [x] 1.3 Add `smctl-verify` dep to `smctl/Cargo.toml`
- [x] 1.4 `cargo build --workspace` passes with the new empty crate

## 2. Verifier Trait + Registry

- [x] 2.1 Define `Verifier` trait: `fn name(&self) -> &'static str`, `fn discover(&self) -> DiscoveryResult`, `fn run(&self, ctx: &VerifyContext) -> VerifyReport` (Send + Sync bound for future concurrent dispatch)
- [x] 2.2 Define `VerifyContext` (workspace root, repo paths, manifest, `--strict` flag, `--verifier` filter) — owned data, no lifetimes
- [x] 2.3 Define `VerifyReport` (verifier name, per-source rows, `Outcome { Passed, Failed, NoSources, ToolMissing }`, diagnostics)
- [x] 2.4 Define `DiscoveryResult` as a tagged enum (`Found { path, version }` / `NotInstalled { tool, install_hint }`) — serializes with `kind` discriminator
- [x] 2.5 `Registry` with `register / find / iter / len`, dedupes by stable name, dispatches in registration order

## 3. Cedar end-to-end (the headline integration)

- [x] 3.1 Implement `cedar::CedarVerifier` using the `cedar-policy` Rust SDK
- [x] 3.2 Discovery: always `Found` (Rust dep, no PATH lookup)
- [x] 3.3 Glob `[verify.policy] sources` against each registered repo via the `glob` crate; per-repo absolute resolution
- [x] 3.4 Parse each policy file with `PolicySet::from_str`. Schema-aware `Validator::validate` is deferred (needs per-repo schema discovery convention) — noted in the cedar.rs module doc.
- [x] 3.5 Three-part remediation on every failure path (read error, UTF-8 error, parse error, empty-policy-set, glob error)
- [x] 3.6 Six unit tests with fixture policies — well-formed, complex (RBAC + condition), malformed (syntax error), empty (comment only), missing files, sources-empty manifest

## 4. Shell-out runners (TLA+, Lean, SPIN)

- [x] 4.1 `tla::TlaVerifier` — discover `tlc -h`, shell out, parse exit code
- [x] 4.2 `lean::LeanVerifier` — discover `lake --version`, shell out at each `[verify.proof] roots` with `lake build`
- [x] 4.3 `spin::SpinVerifier` — discover `spin -V`, shell out at each `[verify.protocol] specs` with `spin -a`
- [x] 4.4 Shared `shell.rs` helpers (`Shell` config struct, `discover_binary`, `run_against_sources`) so each per-tool module is ~15 lines. Unit tests use `/bin/sh` for the Found path and a deliberately bogus binary name for the NotInstalled path. `Registry::with_default_verifiers()` constructor preloads the canonical four in CLI order.

## 5. CLI Integration

- [x] 5.1 `Commands::Verify { verifier, strict, command: VerifyCommands }` added to smctl
- [x] 5.2 `VerifyCommands::{Policy, Model, Proof, Protocol, Discover}` enum, each with `--json` per-verb
- [x] 5.3 `--verifier <name>` and `--strict` are verify-level flags propagating to subverbs via `global = true`
- [x] 5.4 Dispatch builds `VerifyContext` from the workspace manifest, calls `Verifier::run`, renders the report
- [x] 5.5 TTY-aware JSON fallback applied (verb `--json` || global `--json` || stdout-not-a-TTY)
- [x] 5.6 Three-part remediation on the `verifier_not_registered` error path; per-verifier failures inherit from each verifier's own three-part diagnostics
- [x] 5.7 `--dry-run` prints "would run verifier 'X' against N source pattern(s)" and exits with `DRY_RUN`. `Outcome::ToolMissing` exits 0 by default; `--strict` promotes it to a failure

## 6. Workspace.toml [verify] section

- [x] 6.1 `VerifyManifestSection` + four subsection structs added to `smctl-workspace/src/lib.rs`
- [x] 6.2 `[verify.policy] / [verify.model] / [verify.proof] / [verify.protocol]` — each `deny_unknown_fields`, each independently optional. Field name per domain: `sources` / `specs` / `roots` / `specs`.
- [x] 6.3 Six new tests: all subsections parse, absent-is-None, single-subsection-others-default-to-None, rejects unknown subsection, rejects unknown field within subsection, default `fail_on` is `"any"`
- [x] 6.4 README workspace.toml reference gains four `[verify.<domain>]` blocks

CLI dispatch in §5 now consults `manifest.verify.<verb>` to populate the per-call `VerifyManifest`. Policy / Model / Proof / Protocol each pull from their own subsection and fall back to default (empty sources, "any" fail_on) when the subsection is absent.

## 7. Logging MSGIDs

- [x] 7.1 Range `SMCTL-0500..0599` documented in the smctl-log MsgId range table
- [x] 7.2 `VerifyStarted (501)`, `VerifySucceeded (502)`, `VerifyFailed (503)`, `VerifierMissing (504)` allocated
- [x] 7.3 Default severities: Informational / Informational / Error / Warning
- [x] 7.4 Three new smctl-log tests: `verify_codes_and_display_match_spec_catalog`, `verify_default_severity_matches_spec`, `verify_codes_sit_in_reserved_range`
- [x] 7.5 CLI dispatch emits `VerifyStarted` before each run; `VerifySucceeded` on `Passed`/`NoSources`, `VerifyFailed` on `Failed`, `VerifierMissing` on `ToolMissing`

## 8. MCP tools

- [x] 8.1 `smctl_verify_policy` MCP tool returns the Cedar VerifyReport JSON
- [x] 8.2 `smctl_verify_model`, `smctl_verify_proof`, `smctl_verify_protocol`, `smctl_verify_discover` mirror the CLI surface
- [x] 8.3 Five `smctl_verify_*` tools added to the `EXPECTED_TOOLS` list in stdio_handshake; new roundtrip assertions exercise `verify_discover` (asserts 4 entries, Cedar shows kind=found) and `verify_policy` (asserts outcome=no_sources against the fixture manifest).

## 9. Docs

- [x] 9.1 README "Verification" section + Subcommands table rows + Architecture entry for the smctl-verify crate
- [x] 9.2 CLAUDE.md gains a "Formal verification" subsection that points at the capability spec, MSGID range, and the v1 scope (Cedar e2e + shell-outs)
- [ ] 9.3 `openspec/specs/smctl-verify/spec.md` lifted from this change at archive time — this happens via `openspec archive` after the PR merges, not in this commit

## 10. Verify

- [x] 10.1 `cargo build --workspace` clean
- [x] 10.2 `cargo test --workspace` — 232 passed, 0 failed
- [x] 10.3 `cargo clippy --workspace -- -D warnings` clean
- [x] 10.4 `cargo fmt --check` clean
- [x] 10.5 `openspec validate formal-methods-v1 --strict` passes; `openspec validate --all --strict` reports 8 / 8
- [x] 10.6 Cedar end-to-end is unit-tested in §3 (well-formed, malformed-syntax, empty-policy, complex-policy-with-conditions, missing-files-glob); the manual smoke is redundant
- [x] 10.7 Shell-out tool-missing path covered by `shell::tests::run_returns_tool_missing_when_binary_absent` (spawns a deliberately-bogus binary); the manual `tlc`-missing smoke would just exercise the same code path
