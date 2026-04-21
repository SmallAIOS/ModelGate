# Safety-Critical Engineering Quality Toolchain — Design Document

## Context

SmallAIOS has a formal verification stack (Cedar, TLA+, Lean 4, P, SPIN, MISRA-Rust) that proves protocol and policy correctness. This change adds the complementary layer: **engineering quality tools** that ensure the Rust codebase is structurally sound, free of unnecessary complexity, secure in its supply chain, and prepared for safety-critical compiler qualification. These tools produce measurable, enforceable quality metrics that gate CI and spec validation.

## Goals / Non-Goals

### Goals

1. Detect and prevent module-level dependency cycles across the workspace (DSM)
2. Enforce maximum cyclomatic and cognitive complexity per function
3. Audit all dependencies for known vulnerabilities and license compliance
4. Track and minimize unsafe code usage with justification requirements
5. Prepare the codebase for Ferrocene (qualified Rust compiler) compilation
6. Provide a single `smctl quality` command surface for all quality checks
7. Share the exact same toolchain across ModelGate and SmallAIOS kernel repos

### Non-Goals

1. Not replacing formal verification — quality tools complement, don't substitute for, Cedar/TLA+/Lean 4
2. Not implementing MISRA-Rust checking — that's a separate coding standard (already in use)
3. Not building custom static analysis — leveraging existing Cargo ecosystem tools
4. Not purchasing Ferrocene yet — preparing for it so the switch is seamless
5. Not enforcing code coverage thresholds — coverage is a weak quality signal for safety-critical code

## Decisions

### Decision 1: DSM via cargo-modules + cargo-depgraph

**Choice:** Use `cargo-modules` for module-level dependency analysis (within crates) and `cargo-depgraph` for crate-level dependency visualization (between crates). Together they produce a Design Structure Matrix.

**How it works:**

```bash
# Module-level structure (within a crate)
cargo modules structure --lib smctl-flow

# Module-level dependencies with cycle detection
cargo modules dependencies --lib smctl-flow --no-externs

# Crate-level dependency graph (DOT format → SVG)
cargo depgraph --workspace-only | dot -Tsvg > dsm-crates.svg

# Full dependency graph including external crates
cargo depgraph | dot -Tsvg > dsm-full.svg
```

**smctl integration:**
- `smctl quality dsm` generates the DSM and checks for cycles
- `smctl quality dsm --svg` produces visual output
- `smctl quality dsm --json` produces machine-readable output
- CI fails if `enforce_no_cycles = true` in workspace.toml and cycles are detected

**Rationale:** A DSM reveals architectural problems that are invisible in code review: hidden coupling, unexpected transitive dependencies, and modules that should be independent but aren't. For a safety-critical system, architectural clarity is non-negotiable. `cargo-modules` and `cargo-depgraph` are the standard Rust ecosystem tools for this — no need to build custom analysis.

**DSM interpretation for SmallAIOS:**
- **Diagonal blocks** = crate self-dependencies (expected)
- **Below diagonal** = dependencies following the intended architecture (expected)
- **Above diagonal** = reverse dependencies / coupling (problematic — potential cycles)
- **Dense off-diagonal clusters** = high coupling zones that need refactoring

### Decision 2: Cyclomatic + cognitive complexity via rust-code-analysis + clippy

**Choice:** Use `rust-code-analysis` (Mozilla) for cyclomatic complexity measurement and clippy's `cognitive_complexity` lint for cognitive complexity. Both produce per-function metrics.

**Tool selection:**

| Tool | Metric | Pros | Cons |
|---|---|---|---|
| `rust-code-analysis` | Cyclomatic (McCabe) | Mozilla-maintained, JSON output, supports Rust natively | Separate binary to install |
| clippy `cognitive_complexity` | Cognitive (SonarSource) | Already in toolchain, no install | Only warns, doesn't produce structured report |
| `tokei` | Lines of code | Fast, well-known | Not a complexity metric |

**Thresholds (configurable in workspace.toml):**

| Metric | Default | MISRA Equivalent | Rationale |
|---|---|---|---|
| Cyclomatic complexity | ≤ 15 | MISRA C Rule 1.1 (≤ 20) | Tighter for new code; kernel legacy may need exceptions |
| Cognitive complexity | ≤ 25 | No direct equivalent | SonarSource recommendation for maintainable code |
| Function length | ≤ 100 lines | MISRA C Dir 4.1 | Encourages decomposition |

**Rationale:** Cyclomatic complexity is the standard metric for safety-critical code (IEC 61508, DO-178C reference it). Cognitive complexity adds a human-readability dimension that cyclomatic misses (nested conditionals score higher). Using both gives a complete picture. Configurable thresholds allow the kernel (legacy, lower thresholds over time) and smctl (new, strict from day one) to have appropriate limits.

### Decision 3: Dependency security via cargo-audit + cargo-deny

**Choice:** Use `cargo-audit` for RUSTSEC advisory checking and `cargo-deny` as the comprehensive dependency linter covering advisories, licenses, bans, and sources.

**cargo-deny configuration (deny.toml):**

```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-3.0"]
default = "deny"

[bans]
multiple-versions = "warn"
wildcards = "deny"
highlight = "all"

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

**Rationale:** For a safety-critical kernel, every dependency is part of the trusted computing base. `cargo-deny` is the most comprehensive tool — it checks not just vulnerabilities but also license compatibility, source authenticity, and duplicate versions. `cargo-audit` provides the fastest RUSTSEC checking for rapid feedback. Using both is standard practice in security-sensitive Rust projects.

### Decision 4: Unused dependency detection via cargo-machete

**Choice:** Use `cargo-machete` as the primary unused dependency detector. Optionally use `cargo-udeps` (requires nightly) for deeper analysis.

**Why cargo-machete over cargo-udeps alone:**
- `cargo-machete` works on stable Rust — no nightly required
- Fast (regex-based scan, no compilation needed)
- Good enough for 95% of cases
- `cargo-udeps` is more thorough but requires nightly and full compilation — use in CI only

**Rationale:** Unused dependencies inflate build time, increase binary size, and expand the attack surface. In a safety-critical context, every dependency must justify its presence. `cargo-machete` provides fast local feedback; `cargo-udeps` provides thorough CI verification.

### Decision 5: Unsafe code tracking via cargo-geiger

**Choice:** Use `cargo-geiger` to report all `unsafe` code usage across the workspace, including transitive dependencies.

**smctl integration:**
- `smctl quality unsafe` runs `cargo-geiger` and produces a report
- `smctl quality unsafe --json` for machine-readable output
- For smctl crates: target is **zero unsafe** (pure safe Rust)
- For kernel crates: track unsafe blocks with mandatory justification comments

**Unsafe justification format (convention):**
```rust
// SAFETY: Direct hardware register access required for MMIO.
// Justified: No safe abstraction exists for bare-metal register writes.
// Review: 2026-02-15 by @author
unsafe { mmio_write(addr, value) }
```

**Rationale:** `cargo-geiger` is the standard tool for auditing unsafe usage. In a safety-critical system, every `unsafe` block is a potential soundness hole. Tracking them centrally — with justification and review dates — ensures they're deliberate, minimal, and reviewed.

### Decision 6: Ferrocene compiler readiness (pre-purchase)

**Choice:** Prepare the codebase for Ferrocene compilation *before* purchasing the license. This means:

1. **Identify Ferrocene-incompatible features** — Ferrocene supports a subset of Rust (qualified features only). Audit the codebase for features outside this subset.
2. **Add `--ferrocene` build flag** — `smctl build --ferrocene` will invoke the Ferrocene toolchain when available, falling back to rustc with a compatibility warning.
3. **Track Ferrocene qualification status** — Document which crates are Ferrocene-ready.
4. **Target architecture alignment** — Ferrocene currently supports `aarch64-unknown-none` and `x86_64-unknown-none` (among others). Ensure SmallAIOS targets align.

**Ferrocene-incompatible patterns to audit:**
- Nightly-only features
- `#![feature(...)]` gates
- Inline assembly (`asm!`) — supported but with qualification caveats
- Compiler intrinsics not in Ferrocene's qualified set
- Build scripts that assume standard rustc behavior

**Rationale:** Ferrocene is the only IEC 61508 SIL 4 / ISO 26262 ASIL D qualified Rust compiler. For SmallAIOS to claim safety-critical compliance, it must compile with a qualified compiler. Starting preparation now — auditing compatibility, documenting gaps, structuring builds — means the switch to Ferrocene is a configuration change, not a rewrite.

### Decision 7: Quality gates in CI pipeline

**Choice:** Quality checks run as a separate CI job that gates merges, alongside the existing format/clippy/test/build jobs.

```yaml
quality:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install quality tools
      run: |
        cargo install cargo-modules cargo-depgraph cargo-machete cargo-audit cargo-deny cargo-geiger
    - name: DSM cycle detection
      run: smctl quality dsm --check
    - name: Complexity gates
      run: smctl quality complexity --check
    - name: Dependency audit
      run: smctl quality audit
    - name: Unused dependency check
      run: smctl quality deps --check
    - name: Unsafe code report
      run: smctl quality unsafe --report
```

**Rationale:** Quality gates must be automated and mandatory. Manual quality reviews don't scale and are skipped under time pressure. CI enforcement ensures every merge to develop meets the quality bar.

### Decision 8: `smctl-log` is the log producer; the quality surface is a consumer too

The `smctl-quality` crate emits tracing events via the `smctl_log::MsgId` catalog. It does not install a subscriber — that is `smctl-log`'s job, initialized by `smctl serve` or the normal CLI path. Inside a quality-check run, structured events are emitted at start / completion / violation-found sites with MSGIDs drawn from the reserved `SMCTL-0400`–`SMCTL-0499` range.

**Rationale:** Same reasoning as `smctl-mcp-v1` Decision 6. Consolidating subscriber ownership in `smctl-log` guarantees RFC 5424 conformance at the wire level; every crate in the ecosystem is a producer. A quality run that fires `cargo-audit` with seven advisories and `cargo-machete` with three unused deps emits a coherent stream a SIEM can ingest without per-tool quirks.

### Decision 9: Report output is machine-readable JSON by default, human-readable on a TTY

`smctl quality <verb>` writes structured JSON to stdout when called with `--json` or when stdout is not a TTY (e.g. in CI). Otherwise it renders a human-readable table or tree. Both forms contain the same data.

**Rationale:** Matches the existing `smctl` pattern (every other subcommand supports `--json`). Machine-parseable output is what CI ingests and what MCP clients see when `smctl_quality_*` tools land in a future change. Human forms are for operators running locally.

**Alternative considered:** Separate `smctl quality <verb> --report <file>` flag that writes a standalone artifact. Deferred — compose from JSON stdout with shell redirection for now.

### Decision 10: MSGID allocations in the SMCTL-0400 range

`smctl-logging-v1` reserved `SMCTL-0400`–`SMCTL-0499` for this change. Initial allocations:

| MSGID | Enum variant | Severity | Emitted when |
|---|---|---|---|
| `SMCTL-0400` | `QualityCheckStarted` | Informational | `smctl quality <verb>` begins |
| `SMCTL-0401` | `QualityCheckCompleted` | Informational | verb completes; `violation_count` is 0 |
| `SMCTL-0402` | `QualityCheckFailed` | Error | verb completes with at least one violation |
| `SMCTL-0410` | `DsmCycleDetected` | Error | `cargo-modules` / `cargo-depgraph` reports a cycle |
| `SMCTL-0411` | `ComplexityThresholdExceeded` | Warning | a function exceeds the configured cyclomatic or cognitive threshold |
| `SMCTL-0420` | `DependencyVulnerability` | Error | `cargo-audit` or `cargo-deny` flags a RUSTSEC advisory |
| `SMCTL-0421` | `DependencyUnused` | Warning | `cargo-machete` flags a Cargo.toml entry with no callers |
| `SMCTL-0430` | `UnsafeBlockFound` | Notice | `cargo-geiger` reports an unsafe block (may be expected in kernel crates) |
| `SMCTL-0440` | `FerroceneIncompatibility` | Warning | a Ferrocene-readiness probe surfaces an incompatible pattern |

STRUCTURED-DATA keys are declared per-MSGID in `specs/quality-toolchain.md`. Adding a new MSGID in this range follows the same rules as the other reserved ranges (see `smctl-logging-v1/specs/logging.md`): immutable once published, add a new number rather than repurpose, update the enum and the catalog table in one commit.

**Rationale:** Nine initial MSGIDs cover the common quality-check terminals (start, completion, failure, plus the specific violation classes). Severity is assigned per the semantic of the class: cycles and published-advisory hits are `Error` (code fails CI), complexity and unused deps are `Warning` (visible but not blocking by default), unsafe blocks are `Notice` (expected in kernel crates; annotations are how we know what's deliberate).

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| Tool installation overhead | `smctl setup quality` installs all tools; cache in CI |
| False positives in cargo-machete | Allow per-crate exceptions in workspace.toml |
| Complexity thresholds too strict for kernel | Per-crate overrides; gradual tightening over releases |
| Ferrocene subset too restrictive | Audit early; abstract incompatible patterns behind cfg gates |
| cargo-geiger slow on large dep trees | Cache results; run only in CI, not local dev loop |
| rust-code-analysis not in cargo ecosystem | Install as standalone binary; wrap in smctl |

## Open Questions

1. Should `smctl quality` produce a unified quality score (single number) or only pass/fail per check?
2. Should DSM visualization be interactive (HTML) or static (SVG)?
3. Should Ferrocene compatibility auditing be automated or a manual checklist?
4. Should we adopt `cargo-vet` for supply chain trust verification in addition to `cargo-deny`?
