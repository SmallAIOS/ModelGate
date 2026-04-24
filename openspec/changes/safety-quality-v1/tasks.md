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

- [x] Implement `cargo-modules` wrapper for module-level dependency analysis
- [ ] Implement `cargo-depgraph` wrapper for crate-level dependency graph
- [x] Implement cycle detection in module dependency graph (DFS back-edge detection over parsed adjacency map)
- [x] Implement `smctl quality dsm` — generate DSM report
- [x] Implement `smctl quality dsm --enforce-no-cycles` — fail if cycles detected (default on; pass `false` for report-only)
- [ ] Implement `smctl quality dsm --svg` — generate visual DSM as SVG
- [x] Implement `smctl quality dsm --json` — machine-readable output
- [x] Graceful three-part error when cargo-modules is not installed
- [x] Emit SMCTL-0400 / 0410 / 0401 / 0402 MSGIDs through run
- [x] Run DSM across all workspace members (enumerated via cargo metadata --no-deps)
- [ ] Support `max_coupling_depth` config for transitive dependency limits
- [x] Write tests for cycle detection logic (empty, tree without cycles, back-edge detection, round-trip, threshold gate both directions)
- [x] Integration test that invokes `smctl quality dsm --json` and asserts structural JSON shape (detection-and-skip when cargo-modules absent)

## Cyclomatic & Cognitive Complexity

- [x] Integrate `rust-code-analysis` for cyclomatic complexity measurement
- [ ] Integrate clippy `cognitive_complexity` lint for cognitive complexity
- [x] Implement `smctl quality complexity` — report per-function complexity
- [x] Implement `smctl quality complexity --cyclomatic-threshold / --cognitive-threshold` — fail if thresholds exceeded (CI mode)
- [x] Implement `smctl quality complexity --json` — machine-readable output
- [x] Support `--cyclomatic-threshold` flag (default: 15)
- [x] Support `--cognitive-threshold` flag (default: 25)
- [ ] Support `max_function_lines` config (default: 100)
- [ ] Support per-crate threshold overrides for legacy code
- [ ] Identify and report top-N most complex functions
- [x] Run complexity analysis across all workspace members (enumerated via cargo metadata --no-deps)
- [x] Write tests for threshold checking logic (empty, single-function, two-threshold interaction, round-trip, tool-missing detection, pass/fail gate)
- [x] Graceful three-part error when rust-code-analysis-cli is not installed
- [x] Emit SMCTL-0400 / 0411 / 0401 / 0402 MSGIDs through run
- [x] Integration test that invokes `smctl quality complexity --json` and asserts structural JSON shape (detection-and-skip when rust-code-analysis-cli absent)

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

- [x] Implement `cargo-machete` wrapper for fast unused dependency detection
- [ ] Implement `cargo-udeps` wrapper for thorough analysis (nightly CI only)
- [x] Implement `smctl quality deps` — report dependency health
- [x] Implement `smctl quality deps --fail-on-count <n>` — fail when the unused-dependency count meets `<n>` (CI mode)
- [x] Implement `smctl quality deps --json` — machine-readable output
- [x] Graceful three-part error when cargo-machete is not installed
- [x] Emit SMCTL-0400 / 0421 / 0401 / 0402 MSGIDs through run
- [ ] Support per-crate exception list for false positives
- [ ] Run dependency analysis across all workspace repos
- [x] Write tests for dependency report parsing (empty, single block, multiple blocks, round-trip, threshold gate both directions)
- [x] Integration test that invokes `smctl quality deps --json` and asserts structural JSON shape (detection-and-skip when cargo-machete absent)

## Unsafe Code Tracking

- [x] Implement `cargo-geiger` wrapper for unsafe code usage reporting
- [x] Implement `smctl quality unsafe` — report all unsafe blocks (per-crate counts)
- [ ] Implement `smctl quality unsafe --report` — detailed per-site report with locations
- [x] Implement `smctl quality unsafe --json` — machine-readable output
- [x] Implement `--fail-on-count <n>` threshold gating (default 0 — report-only by default; set to 1 in smctl CI to enforce zero unsafe)
- [x] Graceful three-part error when cargo-geiger is not installed
- [x] Emit SMCTL-0400 / 0430 / 0401 / 0402 MSGIDs through run
- [ ] Track unsafe justification comments (SAFETY: format)
- [ ] Report unjustified unsafe blocks as warnings
- [ ] Support `deny_new_unsafe` config to require justification
- [ ] Differentiate smctl crates (target: zero unsafe) from kernel crates
- [ ] Run unsafe analysis across all workspace repos
- [x] Write tests for unsafe report parsing (empty, single-package, zero-unsafe filter, round-trip, threshold gate both directions)
- [x] Integration test that invokes `smctl quality unsafe --json` and asserts structural JSON shape (detection-and-skip when cargo-geiger absent)

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

- [ ] `smctl quality ferrocene` — compatibility-pattern probe, target alignment audit
- [ ] `smctl quality` (no verb) — compose all verbs into a single run
- [ ] `smctl quality dsm --svg` — visual DSM rendering; crate-level graph via cargo-depgraph
- [ ] `deny.toml` + `cargo-deny` integration — belongs with the `deps` surface eventually but adds its own config surface

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
