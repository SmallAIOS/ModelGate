# Safety-Critical Engineering Quality Toolchain — Proposal

## Why

SmallAIOS is a safety-critical unikernel for AI inference. The formal verification strategy (Cedar, TLA+, Lean 4, P, SPIN) validates correctness of protocols and policies, but it does not address the **engineering quality** of the Rust codebase itself. The kernel is ~120K lines of `#![no_std]` Rust and growing. Without structural quality gates, the codebase risks accumulating:

- **Cyclomatic/cognitive complexity** — Functions that are too complex to reason about, review, or maintain safely. In safety-critical systems, complexity directly correlates with defect density.
- **Architectural coupling cycles** — Circular dependencies between modules that make changes unpredictable and testing incomplete. A Design Structure Matrix (DSM) would expose these immediately.
- **Unused or phantom dependencies** — Cargo.toml dependencies that inflate build time and attack surface without contributing functionality.
- **Known vulnerability exposure** — Dependencies with published CVEs or RUSTSEC advisories that haven't been audited.
- **Unsafe code sprawl** — `unsafe` blocks that expand beyond what's strictly necessary for a `#![no_std]` kernel.
- **Compiler qualification gap** — Standard `rustc` is not qualified for safety-critical use under IEC 61508, ISO 26262, or DO-178C. Ferrocene (by Ferrous Systems) is the only qualified Rust compiler, and preparation for it should begin now.

The existing formal methods stack proves *what the system does*. This change ensures the codebase is *structurally sound enough to trust those proofs*.

## What Changes

Introduce a **safety-quality toolchain** integrated into `smctl` that provides:

### 1. Ferrocene Compiler Readiness
Prepare the codebase for compilation with Ferrocene — the IEC 61508 / ISO 26262 qualified Rust compiler by Ferrous Systems. This includes identifying and resolving incompatibilities, documenting the qualification strategy, and adding a `--ferrocene` build flag for when the license is acquired.

### 2. Design Structure Matrix (DSM) via Cargo Tools
Use `cargo-modules`, `cargo-depgraph`, and related tools to generate a DSM showing module-level and crate-level dependencies. Detect and prevent coupling cycles, measure modularity, and enforce architectural boundaries.

### 3. Cyclomatic & Cognitive Complexity Gates
Integrate complexity measurement tools that fail CI when functions exceed safety-critical thresholds. MISRA and safety standards mandate maximum complexity per function.

### 4. Rust-Native Security & Quality Tooling
A unified suite of Cargo ecosystem tools for dependency auditing, unused dependency detection, unsafe code tracking, and supply chain security.

## Capabilities

### New Capabilities

- `smctl quality dsm` — Generate Design Structure Matrix from crate/module dependency graph
- `smctl quality complexity` — Measure and report cyclomatic/cognitive complexity per function
- `smctl quality audit` — Run security audit across all workspace repos
- `smctl quality deps` — Analyze dependency health (unused, outdated, duplicate, unsafe)
- `smctl quality unsafe` — Report all `unsafe` blocks with justification tracking
- `smctl build --ferrocene` — Build with Ferrocene compiler (when available)
- `smctl build --quality` — Run full quality suite as part of build
- CI gates on complexity thresholds, dependency health, and vulnerability advisories

### Modified Capabilities

- `smctl build --verify` — Extended to include quality checks alongside formal verification
- `smctl spec validate` — Extended to check that quality gates pass before spec archive

## Impact

### New Files

```
smctl-quality/
├── Cargo.toml
└── src/
    ├── lib.rs              # Public API
    ├── dsm.rs              # DSM generation via cargo-modules/cargo-depgraph
    ├── complexity.rs       # Cyclomatic/cognitive complexity analysis
    ├── audit.rs            # Security audit (cargo-audit, cargo-deny)
    ├── deps.rs             # Dependency analysis (cargo-machete, cargo-udeps)
    ├── unsafe_report.rs    # Unsafe code tracking (cargo-geiger)
    └── ferrocene.rs        # Ferrocene compatibility checks
```

### Configuration

```toml
# workspace.toml additions
[quality]
max_cyclomatic_complexity = 15      # MISRA-aligned threshold
max_cognitive_complexity = 25       # SonarSource cognitive complexity
max_function_lines = 100            # Maximum lines per function
deny_new_unsafe = false             # Require justification for new unsafe blocks
ferrocene_target = "aarch64-unknown-none"  # Target for Ferrocene builds

[quality.dsm]
enforce_no_cycles = true            # Fail if module dependency cycles detected
max_coupling_depth = 4              # Maximum transitive dependency depth

[quality.audit]
deny_advisories = true              # Fail on RUSTSEC advisories
deny_unmaintained = true            # Fail on unmaintained dependencies
deny_yanked = true                  # Fail on yanked crate versions
```

### Modified Files

- `Cargo.toml` — Add `smctl-quality` workspace member
- `smctl/Cargo.toml` — Add `smctl-quality` dependency
- `smctl/src/main.rs` — Add `quality` subcommand
- `.github/workflows/ci.yml` — Add quality gate jobs

### Dependencies (Cargo tools — installed, not library deps)

- `cargo-modules` — Module dependency visualization and cycle detection
- `cargo-depgraph` — Crate dependency graph generation (DOT/SVG)
- `cargo-machete` — Unused dependency detection
- `cargo-udeps` — Unused dependency detection (nightly-based, more thorough)
- `cargo-audit` — RUSTSEC advisory database checking
- `cargo-deny` — Comprehensive dependency linting (advisories, licenses, bans, sources)
- `cargo-geiger` — Unsafe code usage reporting
- `cargo-bloat` — Binary size analysis
- Ferrocene toolchain (future, licensed) — Qualified Rust compiler

### Shared Tooling with SmallAIOS Kernel

These tools apply identically to both ModelGate and the SmallAIOS kernel. The `smctl quality` commands are workspace-aware and run across all repos, ensuring a **common quality approach** across the ecosystem:

| Tool | ModelGate Use | SmallAIOS Kernel Use |
|---|---|---|
| `cargo-modules` | DSM for smctl crate structure | DSM for kernel module architecture |
| `cargo-depgraph` | Crate dependency visualization | Crate dependency visualization |
| `cargo-machete` | Unused deps in smctl-* crates | Unused deps in kernel crates |
| `cargo-audit` | Supply chain security | Supply chain security |
| `cargo-deny` | License + advisory enforcement | License + advisory enforcement |
| `cargo-geiger` | Track unsafe in smctl (should be zero) | Track unsafe in kernel (minimize) |
| Ferrocene | Build smctl with qualified compiler | Build kernel with qualified compiler |
| Complexity gates | smctl code quality | Kernel safety-critical complexity limits |

## Cross-Cutting Contracts

This change inherits three contracts from prior foundation specs. Adherence is **MUST**, not **SHOULD**.

- **Voice and lexicon** (`design-system-v1/specs/design-system.md`). Every string the `smctl quality` surface emits — command help, report summaries, CI failure text, HTML / SVG DSM legends — conforms to sentence case, imperative verbs, the canonical status vocabulary (`passed` / `failed` / `present` / `absent`), and the three-part error-message rubric. No emoji, no forbidden Unicode pictographs. Report thresholds use the canonical numeric conventions (space before unit, comma thousands).
- **Logging** (`smctl-logging-v1/specs/logging.md`). Every tracing event from the `smctl-quality` crate uses a MSGID from the reserved `SMCTL-0400`–`SMCTL-0499` range allocated in `design.md` and catalogued in `specs/quality-toolchain.md`. The subscriber is `smctl-log`'s — this crate is a log producer, never a consumer.
- **Error handling** (`smctl-errors-v1/design.md`). Quality-check failures that surface as CI output or exit-nonzero errors carry three-part messages: what failed, what it means for the safety-critical posture, what to do next (an executable `smctl quality <verb> --fix` invocation where one exists, or a specific remediation command otherwise).

MSGID range reservation: `SMCTL-0400`–`SMCTL-0499` for `smctl-quality`. Initial allocation is declared in `design.md` Decision 10; the full catalog lives in `specs/quality-toolchain.md`.

## References

- [`design-system-v1` — voice and lexicon contract](../design-system-v1/specs/design-system.md)
- [`smctl-logging-v1` — MSGID catalog and severity mapping](../smctl-logging-v1/specs/logging.md)
- [`smctl-errors-v1` — three-part error rubric](../smctl-errors-v1/design.md)
- [Ferrocene — Qualified Rust Compiler](https://ferrocene.dev/) (IEC 61508, ISO 26262)
- [Ferrous Systems](https://ferrous-systems.com/) — Ferrocene maintainers
- [cargo-modules](https://crates.io/crates/cargo-modules) — Module structure and dependency analysis
- [cargo-depgraph](https://crates.io/crates/cargo-depgraph) — Dependency graph visualization
- [cargo-machete](https://crates.io/crates/cargo-machete) — Unused dependency detection
- [cargo-audit](https://crates.io/crates/cargo-audit) — RUSTSEC vulnerability checking
- [cargo-deny](https://crates.io/crates/cargo-deny) — Dependency linting
- [cargo-geiger](https://crates.io/crates/cargo-geiger) — Unsafe code usage tracking
- [Design Structure Matrix (DSM)](https://dsmweb.org/) — Architectural coupling analysis
- [MISRA C:2012 / MISRA-Rust](https://www.misra.org.uk/) — Safety-critical coding standards
- [IEC 61508](https://en.wikipedia.org/wiki/IEC_61508) — Functional safety standard
- [ISO 26262](https://en.wikipedia.org/wiki/ISO_26262) — Automotive functional safety
- [Formal Methods spec (smctl-tool-v1)](../smctl-tool-v1/specs/formal-methods.md) — Complementary formal verification strategy
