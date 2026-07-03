# Formal methods — Proposal

## Why

SmallAIOS already runs three formal-verification stacks: TLA+ for behavioural properties, Lean 4 for theorem-proving over the Biba integrity lattice, SPIN/Promela for protocol verification. None of that work is reachable from `smctl`. Operators run each tool by hand against ad-hoc paths; CI pipelines stitch them together with shell. The result is a verification investment with no operator-facing surface — and no way for the smctl-mcp tool catalog to expose any of it to AI agents working in the workspace.

This change introduces `smctl verify`: a single command tree that discovers installed verifiers, dispatches per-domain runs, and aggregates results into the smctl-log MSGID stream. It also adds **Cedar** as a Rust-native policy verifier so smctl can verify authorization rules end-to-end without reaching for a JVM or a separate runtime.

## What Changes

- New `smctl-verify` crate that owns the verify command surface and the per-tool runner trait.
- New `smctl verify` CLI verb with subcommands `policy` (Cedar), `model` (TLA+), `proof` (Lean 4), `protocol` (SPIN/Promela), and `discover` (enumerate which verifiers are on PATH).
- New `[verify]` section in `workspace.toml` that lets each repo declare per-tool source roots and gating thresholds.
- Cedar end-to-end: parse policy files, run analysis (forbidden-by-default checks, policy-set well-formedness, schema conformance), surface diagnostics with three-part remediation per the design system.
- TLA+ / Lean 4 / SPIN runners ship as **shell-out wrappers** in this change — the deep integration (parsing each tool's output, mapping to MSGIDs) lands as named follow-up changes (`tla-plus-runner-v1`, `lean-proof-runner-v1`, `spin-protocol-runner-v1`).
- Reserve MSGID range `SMCTL-0500..0599` for verify in `smctl-log`. First MSGIDs land in this change: `SMCTL-0501` verify started, `SMCTL-0502` verify succeeded, `SMCTL-0503` verify failed, `SMCTL-0504` verifier missing on PATH.
- New MCP tool surface: `verify_policy`, `verify_model`, `verify_proof`, `verify_protocol`, `verify_discover` — exposed automatically once `smctl-mcp` re-scans the verb tree.

## Capabilities

### New Capabilities

- `smctl-verify`: `smctl verify` command tree, per-tool runner trait, Cedar end-to-end policy verification, shell-out wrappers for TLA+/Lean/SPIN, `[verify]` workspace section, MSGID range `SMCTL-0500..0599`.

### Modified Capabilities

- `smctl-cli`: gains the `verify` subcommand and three new global behaviours (verify-discover-on-startup, `--verifier <name>` filter, `--strict` gate).
- `smctl-log`: catalog grows the `SMCTL-0500..0599` range and four MSGIDs (`VerifyStarted`, `VerifySucceeded`, `VerifyFailed`, `VerifierMissing`).
- `smctl-mcp`: gains five MCP tools wrapping the new verb surface.

## Impact

### New Files

```
smctl-verify/
├── Cargo.toml
└── src/
    ├── lib.rs           # Verifier trait, registry, dispatch
    ├── cedar.rs         # Cedar policy verifier (Rust SDK)
    ├── tla.rs           # TLA+ shell-out wrapper
    ├── lean.rs          # Lean 4 shell-out wrapper
    ├── spin.rs          # SPIN/Promela shell-out wrapper
    └── discover.rs      # Per-tool PATH and version detection
```

### Modified Files

- `Cargo.toml` — add `smctl-verify` workspace member.
- `smctl/Cargo.toml` — add `smctl-verify` dep.
- `smctl/src/main.rs` — add `Verify { command }` variant + dispatch.
- `smctl-log/src/msgid.rs` — add `Verify*` MSGIDs in the 0500 range.
- `smctl-mcp/src/lib.rs` — add five `verify_*` tools.
- `smctl-workspace/src/lib.rs` — add `VerifyManifestSection`.
- `openspec/specs/smctl-cli/spec.md` — MODIFIED requirement for the subcommand surface.
- `openspec/specs/smctl-log/spec.md` — MODIFIED requirement for MSGID range allocation.
- `openspec/specs/smctl-mcp/spec.md` — MODIFIED requirement for the tool surface.
- `README.md` — new "Verification" section.

### Dependencies

- `cedar-policy` (4.x) — Rust-native authorization-policy engine, BSL-licensed. The other tools shell out and don't carry a Rust crate dep.

### Out of Scope

- Deep TLA+ output parsing, Lean diagnostic mapping, SPIN counter-example rendering. Each tool's runner ships as a shell-out wrapper that reports `passed` / `failed` / `tool_missing`. Per-tool deep integration lands in `tla-plus-runner-v1`, `lean-proof-runner-v1`, `spin-protocol-runner-v1` follow-up changes.
- Alloy, P, Rego/OPA, MISRA-Rust integration. Discussed in `design.md`; deferred.
- Auto-discovery of verification artefacts (`.tla`, `.lean`, `.cedar` files). The `[verify]` workspace section drives source-root selection in v1; a discovery pass is a v2 concern.

## References

- Salvaged design analysis: this change's `design.md` (originated as the dropped `formal-methods.md` from the stale `claude/design-smctl-tool-auqmR` branch — preserved here in canonical format).
- SmallAIOS formal-verification capability: `/Users/e/Development/GitHub/SmallAIOS/openspec/specs/formal-verification/spec.md`.
- Cedar policy language: <https://www.cedarpolicy.com>.
