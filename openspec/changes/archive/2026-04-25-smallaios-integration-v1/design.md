# SmallAIOS integration shakedown — Design Document

## Context

`smctl v0.1.0` is days from cutting. Every subcommand has unit + integration test coverage against fixtures, but the canonical end-to-end scenario — manage `ModelGate` and `SmallAIOS` together from one workspace — has never been driven. This document scopes the shakedown so it produces useful evidence without sprawling.

## Goals / Non-Goals

### Goals

1. Run **every** `smctl` subcommand at least once against a real workspace containing both repos.
2. Capture each outcome (PASS / GAP / FAIL) with a one-line note in `specs/integration-checklist.md`.
3. Land obviously-small fixes inline so develop ends the day in better shape than it started.
4. Surface the bigger issues as follow-up spec proposals or issues — without expanding scope of this change.

### Non-Goals

1. Achieving 100% feature parity between `smctl` and the SmallAIOS Justfile.
2. Modifying `SmallAIOS` to make `smctl` work better.
3. Running `smctl gate` against a real ModelGate server (no server exists yet).
4. Producing a benchmark report.

## Decisions

### Decision 1: Workspace lives at `/Users/e/Development/GitHub/integration-workspace/`

**Choice:** Create a fresh dir outside both repos, drop a `.smctl/workspace.toml` in it, and register the two existing clones by absolute path.

**Rationale:** Keeps the test isolated from either repo's working tree. Doesn't pollute the user's `Development/GitHub/` parent. Clones are referenced, not duplicated — so any commits made through `smctl flow` show up in the actual repos and can be undone with a single `git reset` afterwards.

**Alternative rejected:** Placing the workspace.toml inside `ModelGate` or `SmallAIOS`. Both repos are git repos themselves; nesting a workspace inside one of them confuses `smctl-workspace`'s repo-discovery logic.

### Decision 2: Use `--workspace <path>` rather than `cd` into the workspace

**Choice:** Every shakedown invocation uses an explicit `--workspace /Users/e/Development/GitHub/integration-workspace`.

**Rationale:** Lets us run from this branch's worktree without needing to `cd`. Also exercises the `--workspace` flag path through every dispatch (which we've never explicitly verified works for every subcommand).

### Decision 3: One commit per shakedown phase, plus one commit per fix

**Choice:** Phases (workspace, flow, spec, build, quality, mcp, gate, web) each get a single results commit that updates `specs/integration-checklist.md`. Inline fixes land as separate, narrowly-scoped commits with a `Closes integration-checklist row N` reference.

**Rationale:** Reviewers can read this PR as either "what worked?" (the checklist commits) or "what needed fixing?" (the fix commits). Mixing them would obscure both stories.

### Decision 4: Use the release binary at `target/release/smctl`

**Choice:** Build once with `cargo build --release --bin smctl`, then invoke that binary directly throughout the shakedown.

**Rationale:** ~3× faster startup than `cargo run`, especially relevant when invoking smctl 30+ times in sequence. Also matches how an installed user runs the tool, which is the surface we're validating.

### Decision 5: Bound the fix budget at 5 inline fixes

**Choice:** If we hit a 6th broken scenario, log it on the checklist and stop fixing. File the rest as follow-up specs.

**Rationale:** Avoids scope creep. The point is evidence, not heroics. Five fixes is roughly a day of focused work; beyond that this PR becomes a release-blocking refactor in disguise.

### Decision 6: Restore SmallAIOS to a clean state at the end

**Choice:** Any branches or worktrees `smctl flow` creates against SmallAIOS during the shakedown get cleaned up before the PR merges. The shakedown commit explicitly captures the cleanup as the last step.

**Rationale:** SmallAIOS is the user's working repo. Don't leave detritus behind.

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| `smctl flow` mutates SmallAIOS unexpectedly | Run with `--dry-run` first for any mutating command; only drop the flag once the dry-run output looks reasonable |
| Workspace.toml schema can't represent SmallAIOS's multi-target build | Capture the schema gap as a finding; don't widen the schema in this PR |
| `smctl quality` runs out of disk or time on SmallAIOS's 22-crate graph | Time-box each `quality <verb>` call at 5 minutes; record `TIMED_OUT` as the result if hit |
| Shakedown surfaces a deep architectural issue | Stop, write it up as a follow-up proposal, ship the partial checklist |

## Open Questions

1. Should the shakedown record findings in machine-readable JSON as well as Markdown? Markdown only for v1; JSON if the checklist grows past 30 rows.
2. Is the right level of "PASS" to mean "command exits 0" or "command produces correct output"? Going with "exits 0 and output is at least sensible" — anything more rigorous belongs in the per-command tests, not here.
3. Do we want CI to keep the integration-workspace alive for future runs? No — local-only for now; CI integration is a separate spec once we've verified the shape.
