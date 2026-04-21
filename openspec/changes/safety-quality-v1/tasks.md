# Safety-Critical Engineering Quality Toolchain — Tasks

## Crate Setup

- [ ] Create `smctl-quality` crate with Cargo.toml
- [ ] Add `smctl-quality` as workspace member in root Cargo.toml
- [ ] Add `smctl-quality` dependency to `smctl` binary crate
- [ ] Add `quality` subcommand to smctl CLI command tree
- [ ] Add `[quality]` section to workspace.toml schema and parser
- [ ] Verify workspace builds with `cargo build --workspace`

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

- [ ] Implement `cargo-audit` wrapper for RUSTSEC advisory checking
- [ ] Implement `cargo-deny` wrapper for comprehensive dependency linting
- [ ] Create default `deny.toml` configuration for SmallAIOS ecosystem
- [ ] Implement `smctl quality audit` — run full security audit
- [ ] Implement `smctl quality audit --json` — machine-readable output
- [ ] Support `deny_advisories`, `deny_unmaintained`, `deny_yanked` config
- [ ] Support license allowlist configuration
- [ ] Support source registry restrictions
- [ ] Run audit across all workspace repos
- [ ] Write tests for audit result parsing

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
- [ ] `--json` output is valid JSON for all quality commands
- [ ] Quality gates work across multiple workspace repos
- [ ] CI quality job passes on current codebase
- [ ] `cargo test --workspace` passes with new crate
- [ ] `cargo clippy --workspace -- -D warnings` passes
