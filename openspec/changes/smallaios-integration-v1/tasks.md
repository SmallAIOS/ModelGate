# SmallAIOS integration — Tasks

The plan is structured around the checklist in `specs/integration-checklist.md`. Each phase ships as its own commit so the run-log of a shakedown reads as a series of discrete, reviewable updates.

## Setup

- [ ] Build `smctl` in release mode for the shakedown
- [ ] Create `/Users/e/Development/GitHub/integration-workspace/`
- [ ] Initialise the workspace (`smctl workspace init --name smallaios-integration`)
- [ ] Register both repos via `smctl workspace add` (ModelGate then SmallAIOS)

## Shakedown phases (one commit each)

- [ ] Phase 1 — workspace
- [ ] Phase 2 — worktree
- [ ] Phase 3 — flow
- [ ] Phase 4 — spec
- [ ] Phase 5 — build
- [ ] Phase 6 — quality
- [ ] Phase 7 — mcp
- [ ] Phase 8 — gate / web

## Inline fixes (budgeted at 5 — see design.md Decision 5)

- [ ] (recorded as found, each fix lands as its own commit referencing the failing checklist row)

## Followups (created as we discover gaps too big for inline)

- [ ] (filed as new spec proposals or repo issues; this list links them)

## Cleanup

- [ ] Remove any feature / worktree branches created on SmallAIOS during the shakedown
- [ ] Remove `/Users/e/Development/GitHub/integration-workspace/`
- [ ] Update `specs/integration-checklist.md` "Findings summary" section

## Verify

- [ ] `cargo build --workspace` still clean after any inline fixes
- [ ] `cargo test --workspace` still green
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] PR description carries the findings summary verbatim
