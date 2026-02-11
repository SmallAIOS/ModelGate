# smctl — Design Document

## Context

SmallAIOS is a multi-repository Rust ecosystem building a unikernel for AI inference. Developers need a single CLI tool — `smctl` — to manage workspaces, branching, specs, builds, and releases across repos. The tool follows the Unix `*ctl` convention and integrates three key workflows: OpenSpec (spec-driven development), git flow (structured branching), and git worktree (parallel development).

Critically, `smctl` is designed as a **dual-interface tool**: a traditional CLI for terminal use, and an **MCP (Model Context Protocol) server** that exposes the same capabilities as tools to AI coding assistants (Claude Code, Cursor, Windsurf, Cline, etc.). This means every `smctl` operation is available both to humans typing commands and to AI agents orchestrating development workflows.

## Goals / Non-Goals

### Goals

1. Single binary CLI — one `smctl` command replaces per-repo scripts and manual git choreography
2. MCP server mode — expose all smctl capabilities as MCP tools for AI coding assistants
3. Git worktree-native — parallel features via linked worktrees, not multiple clones
4. Git flow enforcement — consistent branch naming and merge strategy across all repos
5. OpenSpec integration — first-class CLI surface for the spec-driven development lifecycle
6. Multi-repo awareness — workspace concept spanning SmallAIOS, ModelGate, and future repos
7. Cross-repo build orchestration — dependency-ordered builds with caching
8. ModelGate control plane — model and route management via `smctl gate`
9. Release automation — coordinated versioning, tagging, and changelog generation

### Non-Goals

1. Not a general-purpose git client — delegates to git/libgit2 for operations smctl doesn't wrap
2. Not a CI/CD system — orchestrates local builds, not pipeline definitions
3. Not a package manager — does not replace cargo, does not manage crate publishing
4. Not a deployment tool — builds artifacts, does not deploy to production clusters
5. Not an IDE plugin — MCP integration is the IDE bridge; no native editor plugins

## Decisions

### Decision 1: Rust with clap for CLI framework

**Choice:** Build smctl as a Rust binary using `clap` (derive API) for subcommand dispatch.

**Rationale:** The SmallAIOS ecosystem is Rust-native. Using Rust for the CLI keeps the toolchain uniform, enables static linking for single-binary distribution, and allows sharing types with kernel/runtime crates. `clap` is the de facto standard for Rust CLIs, supports hierarchical subcommands (smctl → workspace → init), shell completions, and man-page generation.

**Alternatives considered:**
- *Go (cobra)* — Would introduce a second language into the ecosystem
- *Python (click/typer)* — Runtime dependency; distribution complexity
- *Shell scripts* — Fragile for multi-repo coordination and worktree management

### Decision 2: Workspace manifest as TOML

**Choice:** `smctl` workspaces are defined by a `.smctl/workspace.toml` file at the workspace root.

```toml
[workspace]
name = "smallaios-dev"
root = "."

[[repos]]
name = "SmallAIOS"
url = "https://github.com/SmallAIOS/SmallAIOS"
path = "smallaios"
default_branch = "main"

[[repos]]
name = "ModelGate"
url = "https://github.com/SmallAIOS/ModelGate"
path = "modelgate"
default_branch = "main"
smctl_home = true                # smctl binary lives in this repo

[flow]
main_branch = "main"
develop_branch = "develop"
feature_prefix = "feature/"
release_prefix = "release/"
hotfix_prefix = "hotfix/"

[worktree]
base_dir = ".worktrees"

[spec]
openspec_dir = "openspec"
```

**Rationale:** TOML is already the Rust ecosystem standard (Cargo.toml). A declarative manifest makes workspace configuration reproducible and version-controllable. The `[[repos]]` array allows adding new SmallAIOS sub-projects without code changes.

### Decision 3: Git worktree as the parallelism primitive

**Choice:** Use `git worktree` (via libgit2 bindings) as the primary mechanism for parallel feature development instead of multiple clones or branch switching.

**How it works:**
```
workspace/
├── smallaios/                  # Main worktree (develop branch)
├── modelgate/                  # Main worktree (develop branch)
└── .worktrees/
    ├── feature-gpu-accel/
    │   ├── smallaios/          # Linked worktree on feature/gpu-accel
    │   └── modelgate/          # Linked worktree on feature/gpu-accel
    └── hotfix-boot-panic/
        └── smallaios/          # Linked worktree on hotfix/boot-panic
```

**Rationale:** Worktrees share the object store, so creating a parallel feature workspace is near-instantaneous and disk-efficient. Each worktree has its own working directory and index, so `cargo build` caches are independent. This eliminates the "stash-switch-unstash" dance when context-switching between features.

**Key behaviors:**
- `smctl worktree add <name>` creates linked worktrees across all repos that need the feature branch
- `smctl worktree list` shows active worktrees with branch and status info
- `smctl worktree remove <name>` cleans up worktrees and optionally deletes the branch
- Worktree directories are co-located under `.worktrees/` to avoid polluting the workspace root

### Decision 4: Git flow model with cross-repo coordination

**Choice:** Adopt git flow branching (main, develop, feature/*, release/*, hotfix/*) with smctl enforcing consistency across repos.

**Branch lifecycle:**
```
main ─────────────────────────────────────────── (tagged releases)
  │                                        ▲
  └─ develop ──────────────────────────────┤
       │          ▲          │        ▲    │
       └─ feature/foo ───────┘        │    │
       └─ feature/bar ────────────────┘    │
                                           │
  └─ release/1.0 ─────────────────────────►┘
  └─ hotfix/cve-fix ──────────────────────►
```

**Cross-repo commands:**
- `smctl flow feature start <name>` — Creates `feature/<name>` branch in relevant repos, optionally creates worktree
- `smctl flow feature finish <name>` — Merges feature into develop across repos, deletes branches
- `smctl flow release start <version>` — Creates `release/<version>` from develop in all repos
- `smctl flow release finish <version>` — Merges to main + develop, tags, generates changelogs
- `smctl flow hotfix start <name>` — Creates `hotfix/<name>` from main
- `smctl flow hotfix finish <name>` — Merges to main + develop, tags

**Rationale:** Git flow provides a well-understood branching discipline. The cross-repo coordination is the key value — smctl ensures that `feature/gpu-accel` exists in both SmallAIOS and ModelGate, that merges happen atomically, and that release tags are consistent.

### Decision 5: OpenSpec commands map directly to lifecycle phases

**Choice:** `smctl spec` subcommands mirror the OpenSpec lifecycle but add validation and git integration.

| Command | OpenSpec Phase | What smctl adds |
|---|---|---|
| `smctl spec new <name>` | Create feature folder | Auto-creates `openspec/changes/<name>/`, scaffolds files, creates git branch |
| `smctl spec ff` | Fast-forward | Validates existing docs, generates missing ones, links to git branch |
| `smctl spec apply` | Implement | Runs tasks.md checklist, tracks completion, updates task status |
| `smctl spec archive` | Archive | Moves to `openspec/changes/archive/YYYY-MM-DD-<name>/`, commits |
| `smctl spec validate` | (new) | Checks spec completeness: proposal exists, design has decisions, tasks have owners |
| `smctl spec status` | (new) | Shows spec progress: tasks done/total, open questions, linked branches |

**Rationale:** OpenSpec provides the document structure; smctl provides the automation. Creating a new spec also creates the git flow feature branch. Archiving a spec also triggers the feature finish merge. This binds the specification lifecycle to the git lifecycle.

### Decision 6: Build orchestration via dependency graph

**Choice:** `smctl build` reads the workspace manifest and builds repos in dependency order.

**Dependency graph:**
```
SmallAIOS (kernel) ──► ModelGate (gateway) ──► OCI Image
```

**Behavior:**
- `smctl build` — Build all repos in dependency order
- `smctl build <repo>` — Build a specific repo and its dependencies
- `smctl build --parallel` — Build independent repos concurrently
- Build commands are defined per-repo in workspace.toml:

```toml
[[repos]]
name = "SmallAIOS"
build_cmd = "cargo build --release"
test_cmd = "cargo test"
depends_on = []

[[repos]]
name = "ModelGate"
build_cmd = "cargo build --release"
test_cmd = "cargo test"
depends_on = ["SmallAIOS"]
```

### Decision 7: ModelGate subcommands for gateway operations

**Choice:** `smctl gate` provides a control-plane interface to ModelGate instances.

**Commands:**
- `smctl gate status` — Show running ModelGate instances and health
- `smctl gate models list` — List registered ONNX models
- `smctl gate models add <path>` — Register a new model
- `smctl gate routes list` — Show inference routing table
- `smctl gate routes set <model> <endpoint>` — Configure routing
- `smctl gate test <model> --input <file>` — Run test inference
- `smctl gate logs [--follow]` — Stream ModelGate logs

**Rationale:** As ModelGate matures, developers need a CLI to interact with running instances during development. This mirrors how `kubectl` interacts with running clusters.

### Decision 8: Configuration layering

**Choice:** Three-tier configuration with override precedence: CLI flags > workspace config > user config.

```
~/.config/smctl/config.toml      # User defaults (git identity, preferred editor)
.smctl/workspace.toml             # Workspace-level settings
CLI flags                         # Per-invocation overrides
```

### Decision 9: Subcommand aliasing for common workflows

**Choice:** Provide short aliases for frequent compound operations.

| Alias | Expands to |
|---|---|
| `smctl feat <name>` | `smctl flow feature start <name> && smctl worktree add <name>` |
| `smctl done <name>` | `smctl worktree remove <name> && smctl flow feature finish <name>` |
| `smctl ss <name>` | `smctl spec new <name>` |
| `smctl sb` | `smctl build` |

### Decision 10: MCP server for AI coding assistant integration

**Choice:** `smctl` embeds an MCP (Model Context Protocol) server that exposes workspace, git flow, worktree, spec, build, and gate operations as MCP tools. AI coding assistants connect to smctl via stdio or SSE transport.

**Architecture:**
```
┌─────────────────────────────────────────────────┐
│  AI Coding Assistant (Claude Code / Cursor / …) │
│                                                 │
│  MCP Client ◄──── stdio/SSE ────► smctl serve   │
└─────────────────────────────────────────────────┘
                                        │
                          ┌─────────────┼─────────────┐
                          ▼             ▼             ▼
                    smctl-workspace  smctl-flow   smctl-spec
                    smctl-worktree   smctl-build  smctl-gate
```

**MCP tool mapping (each CLI subcommand becomes an MCP tool):**

| MCP Tool Name | Parameters | Description |
|---|---|---|
| `smctl_workspace_init` | `{name, repos[]}` | Initialize a multi-repo workspace |
| `smctl_workspace_status` | `{}` | Show workspace repo status |
| `smctl_worktree_add` | `{name, repos[]}` | Create linked worktrees for a feature |
| `smctl_worktree_list` | `{}` | List active worktrees with branches |
| `smctl_worktree_remove` | `{name}` | Clean up a worktree |
| `smctl_flow_feature_start` | `{name, repos[]}` | Start a feature branch across repos |
| `smctl_flow_feature_finish` | `{name}` | Merge feature into develop |
| `smctl_flow_release_start` | `{version}` | Create release branch |
| `smctl_flow_release_finish` | `{version}` | Finalize release (tag, merge, changelog) |
| `smctl_flow_hotfix_start` | `{name}` | Start hotfix from main |
| `smctl_flow_hotfix_finish` | `{name}` | Merge hotfix to main + develop |
| `smctl_spec_new` | `{name}` | Create OpenSpec feature folder + branch |
| `smctl_spec_ff` | `{name}` | Fast-forward: generate spec documents |
| `smctl_spec_apply` | `{name}` | Execute tasks from tasks.md |
| `smctl_spec_archive` | `{name}` | Archive completed spec |
| `smctl_spec_status` | `{name?}` | Show spec progress |
| `smctl_spec_validate` | `{name}` | Validate spec completeness |
| `smctl_build` | `{repo?, parallel?}` | Build repos in dependency order |
| `smctl_gate_status` | `{}` | ModelGate instance health |
| `smctl_gate_models_list` | `{}` | List registered models |
| `smctl_gate_models_add` | `{path}` | Register a model |
| `smctl_gate_test` | `{model, input}` | Run test inference |

**MCP resources (read-only context for AI assistants):**

| Resource URI | Description |
|---|---|
| `smctl://workspace/config` | Current workspace.toml contents |
| `smctl://workspace/status` | Repo statuses, branches, dirty state |
| `smctl://worktree/list` | Active worktrees and their branches |
| `smctl://spec/{name}/proposal` | Proposal document for a spec |
| `smctl://spec/{name}/design` | Design document for a spec |
| `smctl://spec/{name}/tasks` | Tasks and completion status |
| `smctl://flow/branches` | All flow branches across repos |
| `smctl://gate/models` | Registered models and metadata |

**Integration configuration:**

For Claude Code (`~/.claude/claude_desktop_config.json` or project `.mcp.json`):
```json
{
  "mcpServers": {
    "smctl": {
      "command": "smctl",
      "args": ["serve", "--mcp", "--stdio"],
      "env": {
        "SMCTL_WORKSPACE": "/path/to/workspace"
      }
    }
  }
}
```

For Cursor (`.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "smctl": {
      "command": "smctl",
      "args": ["serve", "--mcp", "--stdio"]
    }
  }
}
```

**Rationale:** MCP is the emerging standard for tool integration with AI coding assistants. By exposing smctl as an MCP server, AI agents can autonomously manage workspaces, create feature branches, scaffold specs, and trigger builds — without the user manually typing CLI commands. This is especially powerful for OpenSpec workflows: an AI assistant can call `smctl_spec_new` to create a feature folder, populate the proposal/design/tasks via file writes, then call `smctl_spec_apply` to begin implementation.

**Key design principles for MCP mode:**
- Every CLI command has a 1:1 MCP tool equivalent — no capabilities are CLI-only or MCP-only
- MCP tools return structured JSON; CLI commands return human-readable output (same core logic, different formatters)
- The `smctl serve` command starts the MCP server; all other commands run as normal CLI
- Supports both `stdio` transport (for local tools like Claude Code) and `SSE` transport (for remote/web-based assistants)

### Decision 11: smctl lives inside ModelGate

**Choice:** `smctl` and all `smctl-*` library crates live as Cargo workspace members inside `SmallAIOS/ModelGate`, not in a separate repository.

**Rationale:** ModelGate is the developer-facing control plane of the SmallAIOS ecosystem — it already handles model routing and gateway logic. Adding `smctl` here makes ModelGate the unified "tooling + gateway" repo. This avoids creating yet another repo (which would itself need smctl to manage), keeps the Cargo workspace cohesive, and lets `smctl-gate` directly share types with the ModelGate core library.

**Trade-offs accepted:**
- ModelGate repo grows in scope (gateway + CLI tooling) — mitigated by clear crate boundaries
- `cargo install` from the repo installs both ModelGate and smctl — can be scoped with `--bin smctl`
- If smctl outgrows ModelGate, it can be extracted to its own repo later; the Cargo workspace structure makes this a clean split

**Alternatives considered:**
- *Separate `SmallAIOS/smctl` repo* — Adds a repo that smctl itself would need to manage; circular bootstrapping problem
- *Monorepo (all SmallAIOS in one repo)* — Too large a change to the existing multi-repo structure

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| **libgit2 worktree support gaps** | Fall back to shelling out to `git worktree` for operations libgit2 doesn't cover; wrap in a `GitBackend` trait for swappability |
| **Cross-repo atomicity** | Multi-repo operations (flow finish, release) are not truly atomic; implement two-phase approach: validate all repos first, then execute; provide `--dry-run` flag |
| **Git flow rigidity** | Some teams prefer trunk-based development; make branch model configurable in workspace.toml with git-flow as default |
| **OpenSpec coupling** | smctl should work without OpenSpec; spec commands are optional and gracefully degrade if no openspec/ directory exists |
| **Scope creep into CI/CD** | Build orchestration stops at local builds; explicitly do not add pipeline generation, deployment, or cloud integration |
| **MCP protocol evolution** | MCP spec is still maturing; abstract transport and tool registration behind traits so protocol changes don't require rewriting core logic |
| **AI assistant security** | MCP tools can execute destructive operations (branch delete, force merge); implement confirmation prompts and `--dry-run` support; respect MCP's built-in approval mechanisms |

### Decision 12: Formal methods integration

**Choice:** smctl integrates with the SmallAIOS formal methods toolchain, and key state machine logic within smctl itself is formally specified.

**SmallAIOS already uses:**
- **TLA+** — Specifications for memory allocator, scheduler, and syscall dispatch
- **Lean 4** — Proofs for cryptographic correctness
- **MISRA-Rust** — Coding standards for safety-critical kernel code

**smctl's role in formal methods:**
- `smctl spec validate` checks that formal verification artifacts (TLA+ specs, Lean proofs) are present when required by a spec's safety classification
- `smctl build --verify` can invoke TLA+ model checking and Lean proof checking as part of the build pipeline
- The git flow state machine (branch transitions: develop → feature → develop, develop → release → main) is itself specifiable in TLA+ to verify no illegal branch states are reachable
- MCP tools expose verification status so AI assistants can check proof state before proposing merges

**Within smctl itself:**
- The workspace state machine (init → configured → synced) and flow state machine (branch lifecycle) are candidates for TLA+ specification
- Cross-repo merge ordering (validate-then-execute) can be formally verified for deadlock-freedom
- This is not required for v0.1 but establishes the pattern

## Open Questions

1. **Should smctl manage the smctl binary itself?** (`smctl self-update` via GitHub releases)
2. **Should worktrees share a Cargo target directory?** (saves disk but complicates parallel builds)
3. **Should `smctl spec` invoke AI assistants directly?** (e.g., calling OpenSpec's `/opsx:ff` via subprocess or API)
4. **What is the minimum viable subcommand set for v0.1?** (workspace + worktree + flow, deferring build and gate?)
5. **Which formal methods tool for smctl's own state machines?** (TLA+ for consistency with kernel, or Alloy/P for lighter-weight modeling?)
