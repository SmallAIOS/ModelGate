# smctl OpenSpec Workflow Specification

## Overview

`smctl spec` commands integrate the OpenSpec spec-driven development (SDD) lifecycle into the SmallAIOS workflow. Each spec maps to a git flow feature branch, binding the specification lifecycle to the code lifecycle.

## Lifecycle

```
┌─────────┐     ┌────────────┐     ┌───────────┐     ┌──────────┐     ┌──────────┐
│   new    │────►│ fast-fwd   │────►│  apply    │────►│ validate │────►│ archive  │
│         │     │            │     │           │     │          │     │          │
│ scaffold │     │ generate   │     │ implement │     │ check    │     │ complete │
│ files    │     │ docs       │     │ tasks     │     │ coverage │     │ + merge  │
└─────────┘     └────────────┘     └───────────┘     └──────────┘     └──────────┘
     │                                                                      │
     └── creates feature/ branch ──────────────────── merges to develop ────┘
```

## Directory Structure

Active specs live in `openspec/changes/`:
```
openspec/
├── changes/
│   ├── gpu-acceleration-v1/
│   │   ├── .openspec.yaml       # Metadata
│   │   ├── proposal.md          # Why + what changes
│   │   ├── design.md            # Technical decisions
│   │   ├── tasks.md             # Implementation checklist
│   │   └── specs/               # Detailed requirements
│   │       ├── gpu-memory.md
│   │       └── cuda-kernels.md
│   └── archive/
│       └── 2026-02-01-boot-optimization/
│           ├── .openspec.yaml
│           ├── proposal.md
│           ├── design.md
│           └── tasks.md
└── <project-level-specs>/       # Standing project specs (not changes)
```

## Commands

### `smctl spec new <name>`

Create a new spec feature folder with scaffolded documents.

**Behavior:**
1. Create directory `openspec/changes/<name>/`
2. Generate `.openspec.yaml`:
   ```yaml
   schema: spec-driven
   created: 2026-02-11
   branch: feature/<name>
   status: draft
   ```
3. Generate scaffolded `proposal.md`:
   ```markdown
   # <Name> — Proposal

   ## Why
   <!-- Why is this change needed? What problem does it solve? -->

   ## What Changes
   <!-- Bullet list of what will be added/modified -->

   ## Capabilities
   ### New Capabilities
   <!-- List new modules/features -->

   ### Modified Capabilities
   <!-- List changes to existing modules -->

   ## Impact
   <!-- File structure, affected repos, dependencies -->
   ```
4. Generate scaffolded `design.md`:
   ```markdown
   # <Name> — Design Document

   ## Context
   <!-- Brief background -->

   ## Goals / Non-Goals
   ### Goals
   ### Non-Goals

   ## Decisions
   ### Decision 1: <title>
   **Choice:**
   **Rationale:**
   **Alternatives considered:**

   ## Risks / Trade-offs

   ## Open Questions
   ```
5. Generate scaffolded `tasks.md`:
   ```markdown
   # <Name> — Tasks

   ## Implementation
   - [ ] Task 1
   - [ ] Task 2

   ## Testing
   - [ ] Write unit tests
   - [ ] Write integration tests

   ## Verify
   - [ ] All tests pass
   - [ ] Spec review completed
   ```
6. Create `specs/` subdirectory
7. Create git flow feature branch: `feature/<name>`
8. Commit scaffolded files on the feature branch
9. Optionally create worktree (if `--worktree` flag)

**Example:**
```bash
$ smctl spec new gpu-acceleration
Created spec: openspec/changes/gpu-acceleration/
  ✓ .openspec.yaml
  ✓ proposal.md (scaffold)
  ✓ design.md (scaffold)
  ✓ tasks.md (scaffold)
  ✓ specs/ directory
Created branch: feature/gpu-acceleration
```

### `smctl spec ff [<name>]`

Fast-forward: validate and fill in spec documents. This command checks what exists and reports what's missing or incomplete.

**Behavior:**
1. If `<name>` omitted, detect from current branch (`feature/<name>` → `<name>`)
2. Check spec completeness:
   - proposal.md exists and has non-scaffold content
   - design.md exists and has at least one Decision section
   - tasks.md exists and has at least one task
3. Report status for each document
4. If integrated with an AI assistant (via MCP), the assistant can populate the scaffolds

**Output:**
```
$ smctl spec ff gpu-acceleration
Spec: gpu-acceleration

  proposal.md    ✓ complete (3 sections filled)
  design.md      ⚠ partial  (context filled, 0 decisions)
  tasks.md       ✗ scaffold (no tasks defined)
  specs/         ✗ empty    (no spec files)

Action needed: Fill in design decisions and task list
```

### `smctl spec apply [<name>]`

Track implementation progress against tasks.md.

**Behavior:**
1. Parse tasks.md for checkbox items (`- [ ]` and `- [x]`)
2. Display progress:
   ```
   $ smctl spec apply gpu-acceleration
   Spec: gpu-acceleration — 7/15 tasks complete (47%)

   ## Implementation
   ✓ Set up GPU memory allocator
   ✓ Implement PCIe enumeration
   ✓ Create compute engine abstraction
   ○ Implement CUDA kernel launcher
   ○ Add DMA transfer support
   ...
   ```
3. With `--interactive`: walk through each unchecked task, mark as done

### `smctl spec archive [<name>]`

Archive a completed spec.

**Behavior:**
1. Validate all tasks are complete (warn if not)
2. Move `openspec/changes/<name>/` to `openspec/changes/archive/YYYY-MM-DD-<name>/`
3. Update `.openspec.yaml` with `status: archived` and `archived: YYYY-MM-DD`
4. Commit the move
5. Trigger `smctl flow feature finish <name>` to merge the branch

### `smctl spec validate [<name>]`

Check spec completeness against quality criteria.

**Checks:**
- [ ] `proposal.md` exists and has Why + What Changes sections
- [ ] `design.md` exists and has at least one Decision with rationale
- [ ] `tasks.md` exists and has at least one task
- [ ] `.openspec.yaml` has valid schema and created date
- [ ] Feature branch exists and matches spec name
- [ ] No unresolved TODO/FIXME markers in spec docs (warning)
- [ ] Open Questions section is empty or acknowledged (warning)

**Output:**
```
$ smctl spec validate gpu-acceleration
Validating spec: gpu-acceleration

  ✓ proposal.md — complete
  ✓ design.md — 4 decisions documented
  ✓ tasks.md — 15 tasks defined
  ✓ .openspec.yaml — valid
  ✓ branch — feature/gpu-acceleration exists
  ⚠ design.md — 2 open questions remain
  ⚠ tasks.md — 8/15 tasks incomplete

Result: PASS (with warnings)
```

### `smctl spec status [<name>]`

Show spec progress summary.

**Without name (all specs):**
```
$ smctl spec status
SPEC                        STATUS    TASKS     BRANCH
gpu-acceleration            active    7/15      feature/gpu-acceleration
api-v2-design               draft     0/0       feature/api-v2-design
boot-optimization           archived  12/12     (merged)
```

**With name (single spec detail):**
```
$ smctl spec status gpu-acceleration
Spec: gpu-acceleration
  Status:   active
  Created:  2026-02-01
  Branch:   feature/gpu-acceleration
  Tasks:    7/15 complete (47%)
  Docs:     proposal ✓ | design ✓ | tasks ✓ | specs: 2 files
  Open Qs:  2 remaining
```

### `smctl spec list`

List all specs (active and archived).

```
$ smctl spec list
Active:
  gpu-acceleration (7/15 tasks)
  api-v2-design (draft)

Archived:
  2026-02-01-boot-optimization (12/12 tasks)
  2026-01-15-ipc-redesign (8/8 tasks)
```

## Integration Points

### Git Flow Binding

| Spec Event | Git Flow Action |
|---|---|
| `spec new` | `flow feature start` |
| `spec archive` | `flow feature finish` |
| Feature branch deleted | Spec marked as orphaned (warning) |

### MCP Integration

AI assistants can use the full spec workflow via MCP tools:

1. `smctl_spec_new` → Create spec folder + branch
2. Write proposal/design/tasks via file operations
3. `smctl_spec_validate` → Check completeness
4. `smctl_spec_apply` → Track progress
5. `smctl_spec_archive` → Complete and merge

The `smctl://spec/{name}/*` resources let AI assistants read spec documents as context before making code changes.

### Versioning Convention

Spec names can include version suffixes following the SmallAIOS convention:
- `gpu-acceleration-v1` (initial)
- `gpu-acceleration-v2` (revision/iteration)

This matches the pattern seen in `SmallAIOS/SmallAIOS` openspec changes (e.g., `smallaios-kernel-v1`, `platform-expansion-v2`).
