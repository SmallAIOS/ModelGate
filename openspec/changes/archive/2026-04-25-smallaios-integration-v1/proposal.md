# SmallAIOS integration shakedown — Proposal

## Why

We've shipped a lot of `smctl` surface — workspace, flow, spec, build, quality, mcp, gate, web — entirely against test fixtures and ModelGate's own crates. The other half of the ecosystem is `SmallAIOS` itself: a 22-crate workspace, `edition = "2021"`, kernel built `#![no_std]`, with its own OpenSpec workflow, its own `develop` branch, and its own CI. **None of `smctl` has ever been run against SmallAIOS.** Before we cut `v0.1.0` we need evidence that the canonical use case — managing both ModelGate and SmallAIOS from one CLI — actually works.

Problems this surfaces:

- **Unknown integration bugs.** `smctl-quality dsm` runs against ModelGate fine; will it choke on SmallAIOS's `cfg(not(test))` + `#![no_std]` graph? We don't know.
- **Workspace.toml ergonomics never tested at real scale.** The manifest schema works for two-repo fixtures; SmallAIOS has 22 crate members and platform-specific targets.
- **Cross-repo flow never exercised.** `smctl flow feature start foo` should create a `feature/foo` branch on every repo in the workspace. With ModelGate + SmallAIOS that's two repos, and SmallAIOS's `develop` is real and busy.
- **`smctl spec`'s relationship to a repo that already uses OpenSpec is undefined.** SmallAIOS has its own `openspec/` tree; should `smctl spec list` aggregate them, ignore SmallAIOS, or error?

If any of these are broken, shipping `v0.1.0` first and discovering it after is much more expensive than catching it now.

## What Changes

This is a **shakedown, not a feature**. The deliverable is a populated checklist of integration findings plus whatever fixes turn out to be small enough to land inline.

For each `smctl` subcommand, drive it against a workspace at `/Users/e/Development/GitHub/integration-workspace/` (or equivalent) that registers both `/Users/e/Development/GitHub/ModelGate` and `/Users/e/Development/GitHub/SmallAIOS`. Record the outcome:

- **PASS** — works as documented
- **GAP** — works partially or with rough edges; flag for follow-up
- **FAIL** — broken; fix inline if small, or file a follow-up spec

The checklist lives in `specs/integration-checklist.md`. Findings live in the same file alongside each item.

## Capabilities

### New Capabilities

- (none — this is a verification pass)

### Modified Capabilities

- Whatever surfaces as broken during the shakedown. Fixes land as discrete commits on this branch with the failing scenario referenced in the message.

## Impact

### New Files

```
openspec/changes/smallaios-integration-v1/
├── .openspec.yaml
├── proposal.md
├── design.md
├── specs/integration-checklist.md
└── tasks.md
```

### Modified Files

Indeterminate — depends on what breaks. Each fix is its own commit and shows up in the diff.

## Non-Goals

1. **Not building SmallAIOS itself.** SmallAIOS has its own CI and its own toolchain configuration; this shakedown verifies that `smctl build` *invokes* SmallAIOS's build correctly, not that the SmallAIOS build succeeds.
2. **Not modifying SmallAIOS.** Anything we discover that needs SmallAIOS-side changes gets filed as an issue against `SmallAIOS/SmallAIOS`, not fixed here.
3. **Not running `smctl gate` against a real ModelGate.** That requires a running ModelGate server — out of scope until ModelGate ships its server side. The existing wiremock-backed integration tests in `smctl-gate` cover the protocol surface.
4. **Not reaching 100% subcommand coverage in this PR.** If we hit a high-impact bug early, we fix it and stop; remaining checklist items become a follow-up.

## References

- SmallAIOS workspace: `/Users/e/Development/GitHub/SmallAIOS` (cloned locally)
- ModelGate workspace: `/Users/e/Development/GitHub/ModelGate` (this repo)
- Original smctl spec: `openspec/changes/archive/2026-04-24-smctl-tool-v1/specs/cli-interface.md`
