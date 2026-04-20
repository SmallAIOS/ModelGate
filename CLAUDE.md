# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ModelGate is the developer tooling and model gateway hub for the SmallAIOS ecosystem. Its primary deliverable is **`smctl`** (SmallAIOS Control), a unified CLI tool for managing the SmallAIOS multi-repo workspace.

**Current state:** Alpha (v0.1.0) — initial `smctl` CLI with workspace, git flow, OpenSpec, and build orchestration. 5 crates, 21 tests.

## What smctl Does

- **Workspace management** — `smctl workspace init` configures all SmallAIOS repos into a single workspace using git worktrees for parallel branch work
- **Git flow** — Enforces consistent branching model (main, develop, feature/*, release/*, hotfix/*) across repos
- **OpenSpec workflow** — `smctl spec new/ff/apply/archive/validate` provides CLI access to spec-driven development
- **Build orchestration** — `smctl build` drives cross-repo builds with dependency ordering

## Related Repositories

- **SmallAIOS-Design** (`/home/e/Development/SmallAIOS-Design`) — The OS kernel itself. ~120K lines of Rust, `#![no_std]`, edition 2021.

## Build Commands

```bash
cargo build --workspace        # Build all crates
cargo test --workspace         # Run all tests
cargo clippy --workspace -- -D warnings
cargo fmt -- --check
```

## Branching Model (Git Flow)

- `main` — Production-ready releases
- `develop` — Integration branch for next release
- `feature/*` — New features (branch from develop, merge to develop)
- `release/*` — Release prep (branch from develop, merge to main + develop)
- `hotfix/*` — Emergency fixes (branch from main, merge to main + develop)
- `change/*` — OpenSpec change proposals (equivalent to feature branches)

## OpenSpec Workflow

Changes follow the OpenSpec spec-driven development lifecycle:
1. `spec new` — Scaffold proposal/design/tasks + create feature branch
2. `spec ff` — Fill in spec documents
3. `spec apply` — Track implementation progress
4. `spec validate` — Check completeness
5. `spec archive` — Complete and merge to develop

Specs live in `openspec/changes/<name>/` with: `.openspec.yaml`, `proposal.md`, `design.md`, `tasks.md`, `specs/`.

## Conventions

- `.local/` — AI-generated scratch, temp files, things not for git. Listed in .gitignore.
- All Rust code follows `cargo fmt` and `cargo clippy -D warnings`.
- Feature branches map 1:1 with OpenSpec changes.

## Design system

User-facing copy (CLI output, error messages, docs, any future web UI) follows the SmallAIOS design system declared in `openspec/changes/design-system-v1/specs/design-system.md`. The reference artifacts (tokens, voice rules, iconography, logo proposals) live in `ui/`.

Key rules, applied reflexively:

- **Voice:** address the operator as `you`, never `we`. Sentence case for labels. Imperative verbs on buttons (`Start build`, not `Building…`). No emoji. No exclamation points.
- **Status vocabulary:** reuse canonical terms — `clean`/`dirty`, `ahead N`/`behind N`, `pending`/`running`/`passed`/`failed`, `active`/`archived`, `verified`/`unverified`, `present`/`absent`. New terms require updating the spec first.
- **Error messages:** three parts — what happened, what it means, what to do next (an executable command).
- **Product names:** `SmallAIOS`, `ModelGate`, `smctl` cased exactly. `smctl` stays lowercase; reword rather than capitalize at sentence start.

Claude Code users: the `smallaios-design` skill at `.claude/skills/smallaios-design/` loads these rules automatically. Invoke it when designing or styling any surface.

## Logging

All log output conforms to RFC 5424 via the `smctl-log` crate. Callers use the `tracing` macros; the subscriber emits the wire format. The canonical MSGID catalog and severity-mapping table live in `openspec/changes/smctl-logging-v1/specs/logging.md` — that document is authoritative for any new MSGID allocation, facility choice, or transport change.
