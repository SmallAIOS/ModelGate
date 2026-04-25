# Formal methods — Tasks

## 1. Crate Setup

- [ ] 1.1 Create `smctl-verify/` crate with `Cargo.toml` (cedar-policy, smctl-log, smctl-workspace deps)
- [ ] 1.2 Add `smctl-verify` as a workspace member in root `Cargo.toml`
- [ ] 1.3 Add `smctl-verify` dep to `smctl/Cargo.toml`
- [ ] 1.4 `cargo build --workspace` passes with the new empty crate

## 2. Verifier Trait + Registry

- [ ] 2.1 Define `Verifier` trait: `fn name(&self) -> &str`, `fn discover(&self) -> DiscoveryResult`, `fn run(&self, ctx: &VerifyContext) -> VerifyReport`
- [ ] 2.2 Define `VerifyContext` (workspace root, repo paths, manifest section, `--strict` flag, `--verifier` filter)
- [ ] 2.3 Define `VerifyReport` (verifier name, per-source pass/fail rows, overall outcome, diagnostics)
- [ ] 2.4 Define `DiscoveryResult { Found { path, version }, NotInstalled }`
- [ ] 2.5 Build a registry that owns one boxed `Verifier` per supported tool

## 3. Cedar end-to-end (the headline integration)

- [ ] 3.1 Implement `cedar::CedarVerifier` using the `cedar-policy` Rust SDK
- [ ] 3.2 Discovery: always `Found` (Rust dep, no PATH lookup)
- [ ] 3.3 Glob `[verify.policy] sources` against each registered repo
- [ ] 3.4 Parse each policy file, run `Validator::validate` and `PolicySet::is_valid`
- [ ] 3.5 Map each diagnostic to a three-part remediation message
- [ ] 3.6 Unit tests with fixture policies (well-formed, syntax-error, schema-mismatch)

## 4. Shell-out runners (TLA+, Lean, SPIN)

- [ ] 4.1 `tla::TlaVerifier` — discover `tlc`/`tla2tools.jar`, shell out, parse exit code
- [ ] 4.2 `lean::LeanVerifier` — discover `lake`, shell out at each `[verify.proof] roots`
- [ ] 4.3 `spin::SpinVerifier` — discover `spin`, shell out at each `[verify.protocol] specs`
- [ ] 4.4 Unit tests stub the binaries via `which`-mock; deep output parsing is out of scope (per proposal Out-of-Scope)

## 5. CLI Integration

- [ ] 5.1 Add `Commands::Verify { command: VerifyCommands }` to `smctl/src/main.rs`
- [ ] 5.2 Add `VerifyCommands::{Policy, Model, Proof, Protocol, Discover}` enum
- [ ] 5.3 Wire `--verifier <name>` and `--strict` global-to-verify flags
- [ ] 5.4 Dispatch each subcommand: build `VerifyContext`, call into `smctl_verify::run`, render report
- [ ] 5.5 TTY-aware JSON fallback per safety-quality-v1 Decision 9
- [ ] 5.6 Three-part remediation on every error path
- [ ] 5.7 `--dry-run` previews planned source roots without running any verifier

## 6. Workspace.toml [verify] section

- [ ] 6.1 Add `VerifyManifestSection` to `smctl-workspace/src/lib.rs`
- [ ] 6.2 `[verify.policy] / [verify.model] / [verify.proof] / [verify.protocol]` subsections (all optional, `deny_unknown_fields`)
- [ ] 6.3 Tests: parses, absent-is-None, rejects unknown subsection
- [ ] 6.4 Update README with the section schema

## 7. Logging MSGIDs

- [ ] 7.1 Reserve range `SMCTL-0500..0599` in `smctl-log` MsgId enum + range docstring
- [ ] 7.2 Allocate `VerifyStarted (501)`, `VerifySucceeded (502)`, `VerifyFailed (503)`, `VerifierMissing (504)`
- [ ] 7.3 Default severities: Info / Info / Error / Warning
- [ ] 7.4 Tests: codes_match_spec_catalog, codes_sit_in_reserved_range, default_severity_matches_spec
- [ ] 7.5 Emit `VerifyStarted` at the start of each verifier run, `VerifySucceeded` / `VerifyFailed` at the end, `VerifierMissing` on discovery failure

## 8. MCP tools

- [ ] 8.1 Add `verify_policy` MCP tool wrapping `smctl verify policy --json`
- [ ] 8.2 Add `verify_model`, `verify_proof`, `verify_protocol`, `verify_discover`
- [ ] 8.3 Update `smctl-mcp` integration test to assert each new tool round-trips

## 9. Docs

- [ ] 9.1 README "Verification" section with the five subcommands and `[verify]` example
- [ ] 9.2 CLAUDE.md design-system pointer notes the new `smctl verify` surface
- [ ] 9.3 `openspec/specs/smctl-verify/spec.md` lifted from this change at archive time

## 10. Verify

- [ ] 10.1 `cargo build --workspace` clean
- [ ] 10.2 `cargo test --workspace` clean
- [ ] 10.3 `cargo clippy --workspace -- -D warnings` clean
- [ ] 10.4 `cargo fmt --check` clean
- [ ] 10.5 `openspec validate formal-methods-v1 --strict` passes
- [ ] 10.6 Manual smoke against a Cedar policy file: well-formed passes, malformed fails with remediation
- [ ] 10.7 Manual smoke against a host without `tlc` installed: `verify model --json` returns `tool_missing` cleanly
