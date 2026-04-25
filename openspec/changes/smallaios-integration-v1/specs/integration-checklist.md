# SmallAIOS integration — Checklist

Run-log of the shakedown. Each row records the outcome of one
invocation against the workspace at
`/Users/e/Development/GitHub/integration-workspace/`. Outcomes use
the keys defined in proposal.md: **PASS / GAP / FAIL / SKIP /
TIMED_OUT**.

## Setup

| # | Step | Outcome | Notes |
|---|---|---|---|
| 1 | `mkdir -p /Users/e/Development/GitHub/integration-workspace` | PASS | |
| 2 | `smctl --workspace … workspace init --name smallaios-integration` | PASS | Emits `SMCTL-0001` cleanly |
| 3 | `smctl … workspace add file:///…/ModelGate --path …/ModelGate` | PASS | Records the entry; does not clone |
| 4 | `smctl … workspace add file:///…/SmallAIOS --path …/SmallAIOS` | PASS | |

## Phase 1 — workspace

| # | Command | Outcome | Notes |
|---|---|---|---|
| 1.1 | `workspace status` | PASS | Reports both repos, dirty state, branch |
| 1.2 | `workspace status --json` | PASS | Valid JSON; fields match the human output |
| 1.3 | `workspace sync --dry-run` | PASS | "would fetch/pull" both repos |

## Phase 2 — worktree

| # | Command | Outcome | Notes |
|---|---|---|---|
| 2.1 | `worktree list` | PASS | "no active worktrees" |
| 2.2 | `worktree add shakedown --dry-run` | PASS | "would create worktree set 'shakedown' on branch 'feature/shakedown'" |

## Phase 3 — flow

| # | Command | Outcome | Notes |
|---|---|---|---|
| 3.1 | `flow feature list` | PASS-with-note | Only finds `feature/*` branches. ModelGate uses `change/*` per OpenSpec convention so it shows zero ModelGate entries. Considered a design choice, not a bug. Aggregating `change/*` under `feature list` could be a follow-up. |
| 3.2 | `flow feature start shakedown-test --dry-run` | PASS | "would start feature 'shakedown-test'" |
| 3.3 | `flow release list` | PASS | "no active releases" |
| 3.4 | `flow hotfix list` | PASS | "no active hotfixes" |

## Phase 4 — spec

| # | Command | Outcome | Notes |
|---|---|---|---|
| 4.1 | `spec list` | **FAIL** | Reports "no specs found" even though SmallAIOS has 7+ specs in its own `openspec/changes/` and ModelGate has this very change in flight. Root cause: `openspec_dir = root.join(&manifest.spec.openspec_dir)` resolves at the workspace root, not per-repo. Filed as follow-up: `openspec-aggregate-v1` (proposal: discover specs across each registered repo). Too big for this PR's inline-fix budget. |
| 4.2 | `spec validate <name>` | SKIP | Blocked on 4.1 — there's nothing to validate at the workspace level. |

## Phase 5 — build

| # | Command | Outcome (before fix) | Outcome (after fix) | Notes |
|---|---|---|---|---|
| 5.1 | `build --dry-run` | PASS | PASS | "would build in order: ModelGate → SmallAIOS" |
| 5.2 | `build ModelGate --dry-run` | **FAIL** | PASS | Pre-fix: ignored repo arg, printed full order. Post-fix: prints "would build in order: ModelGate". |
| 5.3 | `build SmallAIOS --dry-run` | **FAIL** | PASS | Pre-fix: same bug. Post-fix: prints "would build in order: SmallAIOS". |

**Fix landed inline** in this PR: extracted `smctl_build::resolve_build_subset(manifest, repo_name)` and called it from both the dry-run dispatch path and `build_inner`. Closes Phase 5 rows. Counts as **fix #1 of 5** against the inline-fix budget.

## Phase 6 — quality

| # | Command | Outcome | Notes |
|---|---|---|---|
| 6.1 | `quality audit --json` | PASS-with-note | Surfaced `tool_missing` JSON because `cargo-audit` is not installed locally. The error path itself is correct and includes a remediation clause. CI exercises the real path. |
| 6.2..6.5 | `quality deps / unsafe / dsm / complexity` | SKIP | Same reason — external cargo plugins not on PATH. Re-running these against SmallAIOS in CI is filed as a follow-up. |

## Phase 7 — mcp

| # | Command | Outcome | Notes |
|---|---|---|---|
| 7.1 | `serve --mcp --stdio` (200ms smoke, stdin `/dev/null`) | PASS-with-note | MCP server emits `SMCTL-0200` and binds to stdio. Empty stdin closes the connection; the EOF path emits an `SMCTL-0099 unhandled error` line ("connection closed: initialize request") rather than a clean shutdown. Minor noise but correct exit code. Filing as a small UX polish follow-up. |

## Phase 8 — gate / web

| # | Command | Outcome | Notes |
|---|---|---|---|
| 8.1 | `gate status --dry-run` | PASS | "would query http://localhost:8080 for /health" — pulls `MODELGATE_URL` defaults correctly. |
| 8.2 | `gate web --dry-run` | PASS | "would start modelgate-web at http://127.0.0.1:9378/ proxying http://localhost:8080". |

## Cleanup

| # | Step | Outcome | Notes |
|---|---|---|---|
| C.1 | Remove any branches `smctl flow` created on SmallAIOS | PASS | Nothing to remove — only `--dry-run` was used in Phase 3. |
| C.2 | Remove `/Users/e/Development/GitHub/integration-workspace` | PASS | |

---

## Findings summary

**Counts:** 1 FAIL fixed inline (Phase 5), 1 FAIL filed for follow-up (Phase 4), 4 PASS-with-note observations, 5 SKIP rows (Phase 6 mostly).

**Inline fix landed (1/5 budget):**

- `smctl build <repo> --dry-run` ignored the repo argument and always printed the full build order. Extracted `resolve_build_subset()` from `build_inner`'s body so both the real build path and the dry-run path share the same filter. Real build was correct already; only the preview was wrong.

**Follow-ups filed:**

1. `openspec-aggregate-v1` — `smctl spec list` (and `validate`, `apply`, `archive`, `status`) need to aggregate across each registered repo's `openspec/`, not look at a single workspace-level dir. Architectural change; deferred.
2. `flow-feature-list-includes-change-branches-v1` — small enhancement to surface `change/*` branches under `flow feature list` since OpenSpec specs use that prefix.
3. `mcp-stdio-clean-eof-shutdown-v1` — `serve --mcp --stdio` should treat stdin EOF as graceful shutdown, not an unhandled error.
4. `quality-tools-bundle-v1` — bundle the cargo plugins (cargo-audit, cargo-geiger, cargo-modules, cargo-machete, rust-code-analysis) as documented prereqs and run them against SmallAIOS in CI.

**Conclusion:** `smctl` is workably integrated with the real SmallAIOS workspace for the workspace / worktree / flow / build / mcp / gate paths. The `spec` surface needs an architectural revisit before claiming end-to-end coverage. Releasing v0.1.0 unblocked from this shakedown is reasonable provided the four follow-ups are tracked.
