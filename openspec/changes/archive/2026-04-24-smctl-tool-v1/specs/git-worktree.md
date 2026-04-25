# smctl Git Worktree Specification

## Overview

`smctl worktree` manages git worktrees across multiple repos simultaneously. Worktrees allow developers to have multiple working directories backed by the same repository, enabling parallel work on different features without branch switching, stashing, or multiple clones.

## Concepts

### Worktree Set

A **worktree set** is a named collection of linked worktrees, one per repo, all on the same-named branch. When you run `smctl worktree add my-feature`, smctl creates a linked worktree in each relevant repo, all checked out to the `my-feature` branch.

### Directory Layout

```
workspace/
├── .smctl/
│   └── workspace.toml
├── smallaios/                      # Main worktree (develop)
│   ├── .git
│   ├── kernel/
│   └── ...
├── modelgate/                      # Main worktree (develop)
│   ├── .git
│   └── ...
└── .worktrees/                     # All linked worktrees live here
    ├── feature-gpu-accel/
    │   ├── smallaios/              # Linked worktree → feature/gpu-accel
    │   │   ├── .git               # (file pointing to main .git)
    │   │   ├── kernel/
    │   │   └── ...
    │   └── modelgate/              # Linked worktree → feature/gpu-accel
    │       ├── .git
    │       └── ...
    └── hotfix-boot-panic/
        └── smallaios/              # Only SmallAIOS needed for this fix
            ├── .git
            └── ...
```

### Key Properties

- **Shared object store:** Linked worktrees share git objects with the main worktree. No data duplication.
- **Independent index:** Each worktree has its own staging area and working directory.
- **Independent build cache:** Each worktree directory has its own `target/` directory for Cargo builds.
- **Branch lock:** Git prevents two worktrees from checking out the same branch simultaneously.

## Configuration

```toml
[worktree]
base_dir = ".worktrees"            # Relative to workspace root
auto_create_branch = true          # Create branch if it doesn't exist
cleanup_on_remove = true           # Delete branch when removing worktree
```

## Commands

### `smctl worktree add <name> [--repos <r1,r2>] [--branch <branch>]`

Create a worktree set.

**Parameters:**
- `<name>` — Worktree set name (used as directory name under `.worktrees/`)
- `--repos` — Comma-separated list of repos to include (default: all)
- `--branch` — Branch to check out (default: `feature/<name>`)

**Behavior:**
1. Resolve target branch:
   - If `--branch` specified, use that
   - Else use `feature/<name>` (creating it from `develop` if `auto_create_branch` is true)
2. Create directory `.worktrees/<name>/`
3. For each target repo:
   - Run `git worktree add .worktrees/<name>/<repo> <branch>`
   - Verify worktree is functional
4. Print summary with paths

**Example:**
```bash
$ smctl worktree add gpu-accel
Created worktree set 'gpu-accel':
  smallaios  → .worktrees/gpu-accel/smallaios/  (feature/gpu-accel)
  modelgate  → .worktrees/gpu-accel/modelgate/  (feature/gpu-accel)
```

**Error cases:**
- Branch already checked out in another worktree → error with guidance
- Branch doesn't exist and `auto_create_branch` is false → error

### `smctl worktree list`

List all worktree sets in the workspace.

**Output:**
```
$ smctl worktree list
NAME              BRANCH                  REPOS          STATUS
gpu-accel         feature/gpu-accel       smallaios,     ✓ clean
                                          modelgate      ✗ 2 modified
hotfix-boot       hotfix/boot-panic       smallaios      ✓ clean
```

**JSON output (--json):**
```json
{
  "worktrees": [
    {
      "name": "gpu-accel",
      "branch": "feature/gpu-accel",
      "repos": [
        {"name": "smallaios", "path": ".worktrees/gpu-accel/smallaios", "clean": true},
        {"name": "modelgate", "path": ".worktrees/gpu-accel/modelgate", "clean": false, "modified": 2}
      ]
    }
  ]
}
```

### `smctl worktree remove <name> [--force]`

Remove a worktree set.

**Behavior:**
1. Check for uncommitted changes in each worktree
   - If dirty and no `--force`: abort with warning
   - If `--force`: proceed despite dirty state
2. For each repo in the worktree set:
   - Run `git worktree remove <path>`
   - If `cleanup_on_remove` is true: delete the branch (local only; remote deletion requires `--delete-remote`)
3. Remove `.worktrees/<name>/` directory

### `smctl worktree cd <name> [<repo>]`

Print the path to a worktree (designed for shell integration).

```bash
# Navigate to the gpu-accel worktree for smallaios
cd $(smctl worktree cd gpu-accel smallaios)

# Or set up a shell alias
alias wtcd='cd $(smctl worktree cd "$@")'
```

## Integration with Git Flow

Worktrees integrate tightly with `smctl flow`:

- `smctl flow feature start <name> --worktree` automatically calls `smctl worktree add <name>`
- `smctl flow feature finish <name>` automatically removes the worktree if one exists
- The convenience alias `smctl feat <name>` combines both: creates branch + worktree
- The convenience alias `smctl done <name>` combines both: removes worktree + merges branch

## Integration with OpenSpec

When creating a spec with `smctl spec new <name>`, if a worktree exists for that name, spec files are created in the worktree's copy of the repo (not the main worktree). This keeps spec work isolated to the feature branch.

## Disk Usage Considerations

- Worktrees share git objects → negligible git overhead per worktree
- Each worktree gets its own Cargo `target/` directory → significant disk usage for Rust projects
- Use `smctl worktree remove` promptly for completed features to reclaim build cache space
- Future consideration: shared `target/` directory via `CARGO_TARGET_DIR` (see design.md open questions)
