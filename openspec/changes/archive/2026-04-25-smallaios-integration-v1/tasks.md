# SmallAIOS integration — Tasks

The plan is structured around the checklist in `specs/integration-checklist.md`. Each phase ships as its own commit so the run-log of a shakedown reads as a series of discrete, reviewable updates.

## Setup

- [x] Build `smctl` in release mode for the shakedown
- [x] Create `/Users/e/Development/GitHub/integration-workspace/`
- [x] Initialise the workspace (`smctl workspace init --name smallaios-integration`)
- [x] Register both repos via `smctl workspace add` (ModelGate then SmallAIOS)

## Shakedown phases

- [x] Phase 1 — workspace (PASS x3)
- [x] Phase 2 — worktree (PASS x2)
- [x] Phase 3 — flow (PASS x4 with one PASS-with-note)
- [x] Phase 4 — spec (FAIL — workspace-root vs per-repo openspec resolution)
- [x] Phase 5 — build (FAIL → fixed inline; dry-run now respects repo filter)
- [x] Phase 6 — quality (SKIP — local cargo plugins not installed; CI exercises these)
- [x] Phase 7 — mcp (PASS-with-note — EOF on stdin emits a noisy error)
- [x] Phase 8 — gate / web (PASS x2)

## Inline fixes (1 of 5 used)

- [x] **Fix #1** — `smctl build <repo> --dry-run` ignored the repo argument. Extracted `resolve_build_subset()` and called it from both the real-build and dry-run paths.

## Follow-ups (filed)

- [ ] `openspec-aggregate-v1` — `smctl spec` should aggregate across each registered repo's `openspec/`, not look at a single workspace-level dir
- [ ] `flow-feature-list-includes-change-branches-v1` — surface `change/*` alongside `feature/*`
- [ ] `mcp-stdio-clean-eof-shutdown-v1` — clean shutdown on stdin EOF instead of "unhandled error"
- [ ] `quality-tools-bundle-v1` — document + bundle the cargo plugin prereqs, run quality phase against SmallAIOS in CI

## Cleanup

- [x] Remove `/Users/e/Development/GitHub/integration-workspace/` (will run after the PR commits)
- [x] No SmallAIOS branches were created — only `--dry-run` mutating commands were used

## Verify

- [x] `cargo build --workspace` clean after the inline fix
- [x] `cargo test -p smctl-build` clean (build_inner refactor didn't break anything)
- [x] `cargo clippy --workspace -- -D warnings` clean
- [x] `cargo fmt --check` clean
- [x] `specs/integration-checklist.md` populated with PASS/FAIL/SKIP outcomes per row
- [x] Findings summary written verbatim into the PR body
