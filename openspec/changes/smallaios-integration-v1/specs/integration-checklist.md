# SmallAIOS integration — Checklist

Run-log of the shakedown. Each row records the outcome of one
invocation against the workspace at
`/Users/e/Development/GitHub/integration-workspace/`. Outcomes use
the keys defined in proposal.md: **PASS / GAP / FAIL / SKIP /
TIMED_OUT**.

## Setup

| # | Step | Outcome | Notes |
|---|---|---|---|
| 1 | `mkdir -p /Users/e/Development/GitHub/integration-workspace` | | |
| 2 | `smctl --workspace … workspace init --name smallaios-integration` | | |
| 3 | `smctl --workspace … workspace add file:///Users/e/Development/GitHub/ModelGate --name ModelGate` | | |
| 4 | `smctl --workspace … workspace add file:///Users/e/Development/GitHub/SmallAIOS --name SmallAIOS` | | |

## Phase 1 — workspace

| # | Command | Outcome | Notes |
|---|---|---|---|
| 1.1 | `smctl --workspace … workspace status` | | |
| 1.2 | `smctl --workspace … workspace status --json` | | |
| 1.3 | `smctl --workspace … workspace sync --dry-run` | | |

## Phase 2 — worktree

| # | Command | Outcome | Notes |
|---|---|---|---|
| 2.1 | `smctl --workspace … worktree list` | | |
| 2.2 | `smctl --workspace … worktree add shakedown --dry-run` | | |

## Phase 3 — flow

| # | Command | Outcome | Notes |
|---|---|---|---|
| 3.1 | `smctl --workspace … flow feature list` | | |
| 3.2 | `smctl --workspace … flow feature start shakedown-test --dry-run` | | |
| 3.3 | `smctl --workspace … flow release list` | | |
| 3.4 | `smctl --workspace … flow hotfix list` | | |

## Phase 4 — spec

| # | Command | Outcome | Notes |
|---|---|---|---|
| 4.1 | `smctl --workspace … spec list` | | Aggregate, ModelGate-only, or error? |
| 4.2 | `smctl --workspace … spec validate <some name>` | | |

## Phase 5 — build

| # | Command | Outcome | Notes |
|---|---|---|---|
| 5.1 | `smctl --workspace … build --dry-run` | | |
| 5.2 | `smctl --workspace … build ModelGate --dry-run` | | |
| 5.3 | `smctl --workspace … build SmallAIOS --dry-run` | | |

## Phase 6 — quality

| # | Command | Outcome | Notes |
|---|---|---|---|
| 6.1 | `smctl --workspace … quality audit` | | (against ModelGate) |
| 6.2 | `smctl --workspace … quality deps` | | |
| 6.3 | `smctl --workspace … quality unsafe` | | (against SmallAIOS — `#![no_std]`) |
| 6.4 | `smctl --workspace … quality dsm` | | |
| 6.5 | `smctl --workspace … quality complexity --path /Users/e/Development/GitHub/SmallAIOS/kernel` | | |

## Phase 7 — mcp

| # | Command | Outcome | Notes |
|---|---|---|---|
| 7.1 | `smctl --workspace … serve --mcp --stdio` (smoke for 1s, then close) | | |

## Phase 8 — gate / web

| # | Command | Outcome | Notes |
|---|---|---|---|
| 8.1 | `smctl --workspace … gate status --dry-run` | | |
| 8.2 | `smctl --workspace … gate web --dry-run` | | |

## Cleanup

| # | Step | Outcome | Notes |
|---|---|---|---|
| C.1 | Remove any branches `smctl flow` created on SmallAIOS | | |
| C.2 | Remove `/Users/e/Development/GitHub/integration-workspace` | | |

---

## Findings summary

(populated as phases complete)
