# Safety-Critical Engineering Quality Toolchain — Tasks

## Crate Setup

- [x] Create `smctl-quality` crate with Cargo.toml
- [x] Add `smctl-quality` as workspace member in root Cargo.toml
- [x] Add `smctl-quality` dependency to `smctl` binary crate
- [x] Add `quality` subcommand to smctl CLI command tree
- [ ] Add `[quality]` section to workspace.toml schema and parser
- [x] Verify workspace builds with `cargo build --workspace`

## Logging Catalog (prerequisite for MSGID emission)

- [x] Scaffold `smctl-log` crate with `MsgId` and `Severity` types
- [x] Allocate SMCTL-0400..=SMCTL-0499 variants for smctl-quality
- [x] Lock wire codes via `quality_msgid_codes_are_locked` test
- [x] Assert reserved-range invariant via `quality_msgids_fall_in_reserved_range` test
- [x] Assert per-variant default severity via `quality_default_severities_match_spec` test

## Tool Installation & Management

- [ ] Implement `smctl setup quality` — install all quality tools via `cargo install`
- [ ] Detect which quality tools are already installed and their versions
- [ ] Provide clear error messages when a required tool is missing
- [ ] Document minimum versions for each tool

## Design Structure Matrix (DSM)

- [ ] Implement `cargo-modules` wrapper for module-level dependency analysis
- [ ] Implement `cargo-depgraph` wrapper for crate-level dependency graph
- [ ] Implement cycle detection in module dependency graph
- [ ] Implement `smctl quality dsm` — generate DSM report
- [ ] Implement `smctl quality dsm --check` — fail if cycles detected (CI mode)
- [ ] Implement `smctl quality dsm --svg` — generate visual DSM as SVG
- [ ] Implement `smctl quality dsm --json` — machine-readable output
- [ ] Support `enforce_no_cycles` config in workspace.toml
- [ ] Support `max_coupling_depth` config for transitive dependency limits
- [ ] Run DSM across all workspace repos (multi-repo aware)
- [ ] Write tests for cycle detection logic
- [ ] Write tests for DSM output formatting

## Cyclomatic & Cognitive Complexity

- [ ] Integrate `rust-code-analysis` for cyclomatic complexity measurement
- [ ] Integrate clippy `cognitive_complexity` lint for cognitive complexity
- [ ] Implement `smctl quality complexity` — report per-function complexity
- [ ] Implement `smctl quality complexity --check` — fail if thresholds exceeded (CI mode)
- [ ] Implement `smctl quality complexity --json` — machine-readable output
- [ ] Support `max_cyclomatic_complexity` config (default: 15)
- [ ] Support `max_cognitive_complexity` config (default: 25)
- [ ] Support `max_function_lines` config (default: 100)
- [ ] Support per-crate threshold overrides for legacy code
- [ ] Identify and report top-N most complex functions
- [ ] Run complexity analysis across all workspace repos
- [ ] Write tests for threshold checking logic

## Dependency Security Audit

- [x] Implement `cargo-audit` wrapper for RUSTSEC advisory checking
- [ ] Implement `cargo-deny` wrapper for comprehensive dependency linting
- [ ] Create default `deny.toml` configuration for SmallAIOS ecosystem
- [x] Implement `smctl quality audit` — run full security audit
- [x] Implement `smctl quality audit --json` — machine-readable output
- [x] Implement `--fail-on <severity>` threshold gating
- [x] Emit SMCTL-0400 / 0420 / 0401 / 0402 MSGIDs through run
- [x] Graceful three-part error when cargo-audit is not installed
- [ ] Support `deny_advisories`, `deny_unmaintained`, `deny_yanked` config
- [ ] Support license allowlist configuration
- [ ] Support source registry restrictions
- [ ] Run audit across all workspace repos
- [x] Write tests for audit result parsing (empty, single-advisory, missing-severity, unknown-severity, round-trip, threshold gate both directions)
- [x] Integration test that invokes `smctl quality audit --json` and asserts structural JSON shape (detection-and-skip when cargo-audit absent)

## Unused Dependency Detection

- [ ] Implement `cargo-machete` wrapper for fast unused dependency detection
- [ ] Implement `cargo-udeps` wrapper for thorough analysis (nightly CI only)
- [ ] Implement `smctl quality deps` — report dependency health
- [ ] Implement `smctl quality deps --check` — fail if unused deps found (CI mode)
- [ ] Implement `smctl quality deps --json` — machine-readable output
- [ ] Support per-crate exception list for false positives
- [ ] Run dependency analysis across all workspace repos
- [ ] Write tests for dependency report parsing

## Unsafe Code Tracking

- [ ] Implement `cargo-geiger` wrapper for unsafe code usage reporting
- [ ] Implement `smctl quality unsafe` — report all unsafe blocks
- [ ] Implement `smctl quality unsafe --report` — detailed report with locations
- [ ] Implement `smctl quality unsafe --json` — machine-readable output
- [ ] Track unsafe justification comments (SAFETY: format)
- [ ] Report unjustified unsafe blocks as warnings
- [ ] Support `deny_new_unsafe` config to require justification
- [ ] Differentiate smctl crates (target: zero unsafe) from kernel crates
- [ ] Run unsafe analysis across all workspace repos
- [ ] Write tests for unsafe report parsing

## Ferrocene Compiler Readiness

- [ ] Audit codebase for nightly-only features (`#![feature(...)]`)
- [ ] Audit codebase for Ferrocene-incompatible patterns
- [ ] Document Ferrocene qualification strategy (IEC 61508 / ISO 26262)
- [ ] Document Ferrocene target architecture alignment (aarch64, x86_64)
- [ ] Implement `smctl build --ferrocene` flag (toolchain selection)
- [ ] Implement Ferrocene compatibility check in `smctl quality`
- [ ] Create per-crate Ferrocene readiness tracking
- [ ] Add `ferrocene_target` config to workspace.toml
- [ ] Identify and abstract Ferrocene-incompatible patterns behind `cfg` gates
- [ ] Write Ferrocene preparation checklist for pre-purchase review

## Unified Quality Command

- [ ] Implement `smctl quality` (no subcommand) — run all quality checks
- [ ] Implement `smctl quality --check` — CI mode, fail on any violation
- [ ] Implement `smctl quality --json` — unified machine-readable report
- [ ] Implement `--repos` filter to scope to specific repos
- [ ] Add `--quality` flag to `smctl build` to include quality checks in build
- [ ] Wire quality results into `smctl spec validate` (quality gates for spec archive)

## CI Integration

- [ ] Add quality job to `.github/workflows/ci.yml`
- [ ] Cache quality tool installations in CI
- [ ] Configure quality gates as required checks for PR merge
- [ ] Add quality badge to README.md
- [ ] Document CI quality gate configuration

## Documentation

- [ ] Document `smctl quality` subcommand reference
- [ ] Document workspace.toml `[quality]` configuration reference
- [ ] Document Ferrocene preparation guide
- [ ] Document DSM interpretation guide (how to read the matrix)
- [ ] Document complexity threshold rationale (IEC 61508 / MISRA alignment)
- [ ] Add quality section to README.md

## Verify

- [ ] `smctl quality dsm --check` detects intentionally introduced cycle
- [ ] `smctl quality complexity --check` fails on function exceeding threshold
- [ ] `smctl quality audit` reports known advisory on test dependency
- [ ] `smctl quality deps --check` detects intentionally unused dependency
- [ ] `smctl quality unsafe --report` identifies all unsafe blocks
- [ ] `smctl quality` runs all checks and produces unified output
- [x] `--json` output is valid JSON for `smctl quality audit`
- [ ] Quality gates work across multiple workspace repos
- [ ] CI quality job passes on current codebase
- [x] `cargo test --workspace` passes with new crates (74 tests)
- [x] `cargo clippy --workspace -- -D warnings` passes

## Spec Drift and Follow-ups

This iteration landed a single vertical slice — the `audit` verb only. The
full five-verb surface is deferred to follow-up changes so each can go
through its own review cycle.

### Deferred verbs (each its own follow-up)

- [ ] `smctl quality dsm` — cargo-modules + cargo-depgraph wrapper, cycle detection, SVG/JSON output
- [ ] `smctl quality complexity` — rust-code-analysis wrapper, threshold gating, top-N reporting
- [ ] `smctl quality deps` — cargo-machete wrapper, unused-dep detection, per-crate exceptions
- [ ] `smctl quality unsafe` — cargo-geiger wrapper, justification-comment tracking
- [ ] `smctl quality ferrocene` — compatibility-pattern probe, target alignment audit
- [ ] `smctl quality` (no verb) — compose all verbs into a single run

### Deferred platform work

- [ ] `smctl setup quality` — auto-install tool dependencies via `cargo install`
- [ ] `smctl build --ferrocene` — toolchain selection flag
- [ ] `smctl build --quality` — run quality suite as part of build
- [ ] `smctl spec validate` — gate spec archive on quality checks
- [ ] `--repos` filter on every verb for multi-repo scoping
- [ ] `[quality]` section in workspace.toml schema and parser
- [ ] Per-crate complexity overrides for legacy code
- [ ] `.github/workflows/ci.yml` — quality job + tool caching + gate config
- [ ] `deny.toml` — license + ban + source policy for `cargo-deny`
- [ ] README.md quality section + badge
- [ ] DSM interpretation guide, complexity-threshold rationale doc
- [ ] MCP tool surface (`smctl_quality_audit` etc.) — owned by smctl-mcp-v1

### Inherited prerequisite work also landed here

The `smctl-log` crate and MSGID catalog were supposed to land in the
`smctl-logging-v1` change, which has not been written yet. A minimal
`smctl-log` crate (MsgId + Severity + tests) was scaffolded here as a
pragmatic unblock. A follow-up change should:

- [ ] Formalise `smctl-logging-v1` spec and retrofit this `smctl-log`
      into whatever shape that spec declares
- [ ] Add the RFC 5424 subscriber (the consumer side) — this slice ships
      the producer side only
- [ ] Add STRUCTURED-DATA wire-format verification tests
- [ ] Allocate MSGID ranges for non-quality producers (workspace, flow,
      spec, build, mcp) per their respective specs
