# Safety-Critical Quality Toolchain — Specification

## Overview

This spec defines the toolchain, integration patterns, and CLI interface for `smctl quality` — the engineering quality layer that complements SmallAIOS's formal verification stack. The tools fall into five categories: structural analysis (DSM), complexity measurement, dependency security, unsafe tracking, and compiler qualification.

## Inherited contracts

This surface MUST satisfy three contracts declared in prior changes:

- [`design-system-v1/specs/design-system.md`](../../design-system-v1/specs/design-system.md) — voice, lexicon, error-message rubric, forbidden Unicode pictographs. Applies to command help, report summaries, CI output, DSM diagram legends.
- [`smctl-logging-v1/specs/logging.md`](../../smctl-logging-v1/specs/logging.md) — RFC 5424 wire format. `smctl-quality` is a log producer only; it never installs its own subscriber.
- [`smctl-errors-v1/design.md`](../../smctl-errors-v1/design.md) — three-part error rubric for every CI failure or exit-nonzero path.

## MSGID catalog for smctl-quality

Allocated from the `SMCTL-0400`–`SMCTL-0499` range reserved in `smctl-logging-v1`. Catalog extensions live here alongside the tool-specific structure.

| MSGID | Enum variant | Severity | STRUCTURED-DATA keys | Emitted when |
|---|---|---|---|---|
| `SMCTL-0400` | `QualityCheckStarted` | Informational | `verb`, `repo`, `scope` | `smctl quality <verb>` begins |
| `SMCTL-0401` | `QualityCheckCompleted` | Informational | `verb`, `duration_ms`, `violation_count` | verb completes, `violation_count` is 0 |
| `SMCTL-0402` | `QualityCheckFailed` | Error | `verb`, `duration_ms`, `violation_count`, `remediation` | verb completes with at least one violation |
| `SMCTL-0410` | `DsmCycleDetected` | Error | `crate_a`, `crate_b`, `via` | `cargo-modules` / `cargo-depgraph` reports a module or crate cycle |
| `SMCTL-0411` | `ComplexityThresholdExceeded` | Warning | `function`, `file`, `cyclomatic`, `cognitive`, `threshold` | a function exceeds the configured threshold |
| `SMCTL-0420` | `DependencyVulnerability` | Error | `crate`, `advisory_id`, `severity`, `installed`, `patched` | `cargo-audit` or `cargo-deny` flags an advisory |
| `SMCTL-0421` | `DependencyUnused` | Warning | `crate`, `manifest` | `cargo-machete` flags an unused Cargo.toml entry |
| `SMCTL-0430` | `UnsafeBlockFound` | Notice | `crate`, `file`, `line_count` | `cargo-geiger` reports an unsafe block |
| `SMCTL-0440` | `FerroceneIncompatibility` | Warning | `crate`, `pattern`, `file` | Ferrocene-readiness probe surfaces an incompatible pattern |

All STRUCTURED-DATA keys are snake_case ASCII. Numeric values render as decimal strings. `verb` is one of `dsm`, `complexity`, `audit`, `deps`, `unsafe`, `ferrocene`.

Adding a new MSGID in this range follows the rules in `smctl-logging-v1/specs/logging.md`: immutable once published, allocate the next unused number, update the enum variant and this table in a single commit.

## Tool Stack

### Tier 1 — Core (required, run on every PR)

| Tool | Version | Purpose | Install |
|---|---|---|---|
| `cargo-modules` | 0.17+ | Module dependency structure and cycle detection | `cargo install cargo-modules` |
| `cargo-depgraph` | 4.0+ | Crate dependency graph (DOT output) | `cargo install cargo-depgraph` |
| `cargo-machete` | 0.7+ | Unused dependency detection (stable Rust) | `cargo install cargo-machete` |
| `cargo-audit` | 0.21+ | RUSTSEC advisory checking | `cargo install cargo-audit` |
| `cargo-deny` | 0.16+ | Advisory + license + ban + source linting | `cargo install cargo-deny` |
| `rust-code-analysis` | 0.0.25+ | Cyclomatic complexity measurement | `cargo install rust-code-analysis-cli` |

### Tier 2 — Enhanced (run in CI, not required locally)

| Tool | Version | Purpose | Install |
|---|---|---|---|
| `cargo-udeps` | 0.1+ | Thorough unused dep detection (nightly) | `cargo install cargo-udeps` |
| `cargo-geiger` | 0.11+ | Unsafe code usage reporting | `cargo install cargo-geiger` |
| `cargo-bloat` | 0.12+ | Binary size analysis | `cargo install cargo-bloat` |

### Tier 3 — Future (Ferrocene)

| Tool | Version | Purpose | Acquire |
|---|---|---|---|
| Ferrocene | Latest | IEC 61508 / ISO 26262 qualified rustc | License from Ferrous Systems |

## CLI Interface

### Command Tree

```
smctl quality
├── dsm [--check] [--svg] [--json] [--repos <r1,r2>]
├── complexity [--check] [--json] [--top <n>] [--repos <r1,r2>]
├── audit [--json] [--repos <r1,r2>]
├── deps [--check] [--json] [--repos <r1,r2>]
├── unsafe [--report] [--json] [--repos <r1,r2>]
└── (no subcommand) [--check] [--json] [--repos <r1,r2>]
```

### `smctl quality dsm`

Generates a Design Structure Matrix from the workspace dependency graph.

```
$ smctl quality dsm
Design Structure Matrix — SmallAIOS Workspace

Crate-level dependencies (workspace members only):
  smctl-workspace  →  (none)
  smctl-flow       →  (none)
  smctl-spec       →  (none)
  smctl-build      →  smctl-workspace
  smctl-quality    →  (none)
  smctl            →  smctl-workspace, smctl-flow, smctl-spec, smctl-build, smctl-quality

Module-level analysis:
  smctl-flow/src/lib.rs
    ├── flow::feature  →  (no external module deps)
    ├── flow::release  →  (no external module deps)
    └── flow::hotfix   →  (no external module deps)

Cycles: none detected ✓
Max coupling depth: 2 (limit: 4) ✓
```

```
$ smctl quality dsm --check
DSM check passed: no cycles, coupling depth 2 ≤ 4
```

```
$ smctl quality dsm --svg
DSM visualization written to: .smctl/reports/dsm-2026-03-08.svg
```

### `smctl quality complexity`

Measures cyclomatic and cognitive complexity per function.

```
$ smctl quality complexity
Complexity Report — SmallAIOS Workspace

smctl-flow/src/lib.rs:
  feature_start()      cyclomatic: 8   cognitive: 12   lines: 45   ✓
  feature_finish()     cyclomatic: 11  cognitive: 18   lines: 62   ✓
  release_finish()     cyclomatic: 14  cognitive: 22   lines: 78   ✓

smctl-build/src/lib.rs:
  resolve_build_levels()  cyclomatic: 6   cognitive: 10   lines: 38   ✓
  build_parallel()        cyclomatic: 9   cognitive: 15   lines: 52   ✓

Summary: 47 functions analyzed, 0 violations
  Max cyclomatic: 14 (limit: 15) ✓
  Max cognitive:  22 (limit: 25) ✓
  Max lines:      78 (limit: 100) ✓
```

```
$ smctl quality complexity --check
Complexity check passed: all functions within thresholds

$ smctl quality complexity --top 5
Top 5 most complex functions:
  1. smctl-flow::release_finish()     cyclomatic: 14  cognitive: 22
  2. smctl-flow::feature_finish()     cyclomatic: 11  cognitive: 18
  3. smctl-build::build_parallel()    cyclomatic: 9   cognitive: 15
  4. smctl-flow::feature_start()      cyclomatic: 8   cognitive: 12
  5. smctl-workspace::init()          cyclomatic: 7   cognitive: 11
```

### `smctl quality audit`

Runs security audit against RUSTSEC advisories and cargo-deny rules.

```
$ smctl quality audit
Security Audit — SmallAIOS Workspace

RUSTSEC advisories:
  No vulnerabilities found ✓

License compliance:
  All dependencies use approved licenses ✓
  Approved: MIT, Apache-2.0, BSD-2-Clause, BSD-3-Clause, ISC, Unicode-3.0

Source verification:
  All dependencies from crates.io ✓

Yanked crates: none ✓
Unmaintained crates: none ✓
```

### `smctl quality deps`

Checks dependency health across the workspace.

```
$ smctl quality deps
Dependency Health — SmallAIOS Workspace

Unused dependencies (cargo-machete):
  smctl-build/Cargo.toml: 'regex' appears unused

Duplicate versions:
  syn: v1.0.109, v2.0.48 (both needed by proc-macro deps)

Summary:
  Workspace crates: 6
  Direct dependencies: 24
  Transitive dependencies: 147
  Potentially unused: 1
  Duplicate versions: 1

$ smctl quality deps --check
Dependency check FAILED: 1 potentially unused dependency
  smctl-build/Cargo.toml: 'regex'
```

### `smctl quality unsafe`

Reports unsafe code usage across the workspace.

```
$ smctl quality unsafe --report
Unsafe Code Report — SmallAIOS Workspace

smctl crates (target: zero unsafe):
  smctl/             0 unsafe blocks ✓
  smctl-workspace/   0 unsafe blocks ✓
  smctl-flow/        0 unsafe blocks ✓
  smctl-spec/        0 unsafe blocks ✓
  smctl-build/       0 unsafe blocks ✓
  smctl-quality/     0 unsafe blocks ✓

Dependencies with unsafe:
  tokio (34 unsafe blocks)    — well-audited async runtime
  reqwest (8 unsafe blocks)   — well-audited HTTP client
  clap (0 unsafe blocks)      — safe

Summary:
  Workspace unsafe: 0 ✓
  Dependency unsafe: 42 (in 2 crates)
```

## Ferrocene Compatibility Specification

### Ferrocene Overview

Ferrocene is a downstream distribution of the Rust compiler qualified under:
- **IEC 61508** (SIL 1-4) — Industrial functional safety
- **ISO 26262** (ASIL A-D) — Automotive functional safety

It compiles standard Rust but qualifies only a subset of language features and standard library items. Code using unqualified features will compile but loses the safety qualification claim.

### Readiness Checklist

```
$ smctl quality ferrocene
Ferrocene Readiness — SmallAIOS Workspace

Nightly features:      0 found ✓
  No #![feature(...)] gates detected

Inline assembly:       3 occurrences (kernel only)
  SmallAIOS/src/arch/aarch64/boot.rs:12   asm!("msr ...")
  SmallAIOS/src/arch/aarch64/mmu.rs:45    asm!("tlbi ...")
  SmallAIOS/src/arch/x86_64/boot.rs:8     asm!("cli; hlt")
  Status: asm! is qualified in Ferrocene for aarch64/x86_64 ✓

Target alignment:
  aarch64-unknown-none  — Ferrocene supported ✓
  x86_64-unknown-none   — Ferrocene supported ✓

Compiler intrinsics:   0 unqualified intrinsics found ✓

Build scripts:         2 found
  SmallAIOS/build.rs    — linker script selection (Ferrocene-compatible) ✓
  ModelGate/build.rs    — not present ✓

Summary: Ferrocene-ready (pending license acquisition)
```

### Ferrocene Build Flag

```bash
# When Ferrocene is installed:
smctl build --ferrocene

# Equivalent to:
RUSTUP_TOOLCHAIN=ferrocene cargo build --workspace --target aarch64-unknown-none

# Without Ferrocene installed:
smctl build --ferrocene
# Warning: Ferrocene toolchain not found. Building with standard rustc.
# Note: This build is NOT safety-qualified. Install Ferrocene for qualified builds.
```

## workspace.toml Configuration Reference

```toml
[quality]
# Complexity thresholds
max_cyclomatic_complexity = 15      # Per-function cyclomatic complexity limit
max_cognitive_complexity = 25       # Per-function cognitive complexity limit
max_function_lines = 100            # Maximum lines per function

# Unsafe policy
deny_new_unsafe = false             # If true, new unsafe blocks require justification

# Ferrocene
ferrocene_target = "aarch64-unknown-none"
ferrocene_toolchain = "ferrocene"   # rustup toolchain name

[quality.dsm]
enforce_no_cycles = true            # Fail CI if module dependency cycles detected
max_coupling_depth = 4              # Maximum transitive dependency depth between modules

[quality.audit]
deny_advisories = true              # Fail on RUSTSEC advisories
deny_unmaintained = true            # Fail on unmaintained dependencies
deny_yanked = true                  # Fail on yanked crate versions
license_allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-3.0"]

[quality.deps]
# Per-crate exceptions for cargo-machete false positives
exceptions = [
    { crate = "smctl-build", dep = "regex", reason = "Used via re-export in test module" }
]

# Per-crate complexity overrides (for legacy/kernel code)
[[quality.overrides]]
crate = "SmallAIOS"
max_cyclomatic_complexity = 20      # Kernel has some inherently complex functions
max_function_lines = 150            # Hardware abstraction functions are longer
```

## Relationship to Formal Methods Stack

```
┌─────────────────────────────────────────────────────────────────┐
│                SmallAIOS Quality + Verification                  │
│                                                                  │
│  ┌──────────────────────────────┐  ┌──────────────────────────┐ │
│  │   Engineering Quality        │  │   Formal Verification     │ │
│  │   (this change)              │  │   (formal-methods spec)   │ │
│  │                              │  │                           │ │
│  │  • DSM / cycle detection     │  │  • Cedar (MAC policy)     │ │
│  │  • Complexity gates          │  │  • TLA+ (behavioral)      │ │
│  │  • Dependency audit          │  │  • Lean 4 (proofs)        │ │
│  │  • Unsafe tracking           │  │  • P (async testing)      │ │
│  │  • Ferrocene readiness       │  │  • SPIN (protocols)       │ │
│  │  • Supply chain security     │  │  • MISRA-Rust (standards) │ │
│  │                              │  │                           │ │
│  │  "Is the code well-built?"   │  │  "Is the code correct?"   │ │
│  └──────────────────────────────┘  └──────────────────────────┘ │
│                                                                  │
│  Unified via: smctl build --verify --quality                     │
│  CI gate:     Both must pass for merge to develop                │
└─────────────────────────────────────────────────────────────────┘
```

The quality toolchain answers structural questions that formal methods don't address:
- **DSM:** "Are our modules properly decoupled?" (Formal methods verify behavior, not architecture)
- **Complexity:** "Can a human understand this function?" (Formal proofs don't help if the code is unmaintainable)
- **Audit:** "Are our dependencies safe?" (Formal methods verify our code, not third-party code)
- **Ferrocene:** "Can we claim safety certification?" (Formal proofs are necessary but not sufficient — the compiler must also be qualified)
