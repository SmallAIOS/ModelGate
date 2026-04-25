# smctl Git Flow Specification

## Overview

`smctl flow` implements the git flow branching model with cross-repository coordination. It ensures that branch naming, merge strategy, and release tagging are consistent across all repos in a SmallAIOS workspace.

## Branch Model

```
main ─────────────────────────────────────────────── stable releases
  │                                            ▲  ▲
  └── develop ─────────────────────────────────┤  │
       │          ▲          │          ▲       │  │
       └─ feature/foo ───────┘          │       │  │
       └─ feature/bar ──────────────────┘       │  │
                                                │  │
  └── release/1.0.0 ───────────────────────────►┘  │
  └── hotfix/cve-2026-1234 ────────────────────────►┘
```

### Branch Types

| Branch | Source | Merges Into | Naming | Lifetime |
|---|---|---|---|---|
| `main` | — | — | `main` | Permanent |
| `develop` | `main` | `main` (via release) | `develop` | Permanent |
| `feature/*` | `develop` | `develop` | `feature/<name>` | Temporary |
| `release/*` | `develop` | `main` + `develop` | `release/<semver>` | Temporary |
| `hotfix/*` | `main` | `main` + `develop` | `hotfix/<name>` | Temporary |

### Configuration

Branch naming is configurable in `.smctl/workspace.toml`:

```toml
[flow]
main_branch = "main"
develop_branch = "develop"
feature_prefix = "feature/"
release_prefix = "release/"
hotfix_prefix = "hotfix/"
tag_prefix = "v"                # Tags become v1.0.0, v1.1.0, etc.
```

## Commands

### `smctl flow init`

Initialize git flow in all workspace repos. Creates `develop` branch from `main` if it doesn't exist.

**Behavior:**
1. For each repo in workspace:
   - Verify `main` branch exists
   - Create `develop` from `main` if missing
   - Push `develop` to origin
2. Write flow config to `.smctl/workspace.toml` if not present

**Preconditions:**
- Workspace must be initialized (`smctl workspace init`)
- All repos must have a `main` branch

### `smctl flow feature start <name> [--repos <r1,r2>] [--worktree]`

Create a feature branch across repos.

**Behavior:**
1. Validate `<name>` doesn't conflict with existing branches
2. For each target repo (default: all, or `--repos` subset):
   - Ensure `develop` is up to date with origin
   - Create `feature/<name>` from `develop`
   - Push branch to origin
3. If `--worktree`: create linked worktrees via `smctl worktree add <name>`

**Naming rules:**
- `<name>` must match `[a-z0-9][a-z0-9-]*` (lowercase, hyphens, no slashes)
- `feature/` prefix is added automatically

### `smctl flow feature finish <name> [--no-delete] [--squash]`

Merge feature into develop across repos.

**Behavior:**
1. For each repo where `feature/<name>` exists:
   - Switch to `develop`
   - Merge `feature/<name>` into `develop` (merge commit by default, `--squash` optional)
   - Delete local and remote `feature/<name>` branch (unless `--no-delete`)
   - Push `develop` to origin
2. If worktree exists for `<name>`: remove it

**Conflict handling:**
- If merge conflicts occur in any repo, abort the merge in that repo and report
- Successfully merged repos are not rolled back (partial completion is acceptable)
- User resolves conflicts manually, then re-runs `smctl flow feature finish`

### `smctl flow release start <version>`

Create a release branch from develop across all repos.

**Behavior:**
1. Validate `<version>` is valid semver
2. For each repo:
   - Create `release/<version>` from `develop`
   - Push to origin

### `smctl flow release finish <version>`

Finalize a release: merge to main + develop, tag, generate changelogs.

**Behavior:**
1. For each repo:
   - Merge `release/<version>` into `main`
   - Tag `main` with `v<version>` (signed if GPG key configured)
   - Merge `release/<version>` into `develop`
   - Delete `release/<version>` branch
   - Push `main`, `develop`, and tags to origin
2. Generate cross-repo changelog (commits since last tag)

### `smctl flow hotfix start <name>`

Create a hotfix branch from main.

**Behavior:**
1. For each repo:
   - Create `hotfix/<name>` from `main`
   - Push to origin

### `smctl flow hotfix finish <name>`

Merge hotfix to main + develop.

**Behavior:**
1. For each repo where `hotfix/<name>` exists:
   - Merge `hotfix/<name>` into `main`
   - Auto-increment patch version, tag
   - Merge `hotfix/<name>` into `develop`
   - Delete branch
   - Push everything

## Cross-Repo Coordination

### Repo Selection

By default, flow commands operate on **all repos** in the workspace. The `--repos` flag limits scope:

```bash
# Feature only in SmallAIOS repo
smctl flow feature start gpu-accel --repos SmallAIOS

# Feature spanning SmallAIOS and ModelGate
smctl flow feature start api-v2 --repos SmallAIOS,ModelGate
```

### Atomicity

Multi-repo operations are **not** atomically transactional. smctl uses a two-phase approach:

1. **Validate phase:** Check preconditions in all repos (branch exists, no conflicts, working tree clean)
2. **Execute phase:** Perform operations repo-by-repo

If execution fails partway through, smctl reports which repos succeeded and which failed. The user can re-run to complete the remaining repos.

### Dry Run

All flow commands support `--dry-run` to preview changes without executing:

```bash
$ smctl flow feature finish gpu-accel --dry-run
Would merge feature/gpu-accel → develop in SmallAIOS
Would merge feature/gpu-accel → develop in ModelGate
Would delete feature/gpu-accel in SmallAIOS (local + remote)
Would delete feature/gpu-accel in ModelGate (local + remote)
```
