# smctl — SmallAIOS Control Tool — Proposal

## Why

SmallAIOS is a multi-repo ecosystem: the kernel lives in `SmallAIOS/SmallAIOS`, model gateway logic in `SmallAIOS/ModelGate`, and future components (scheduler, networking, tooling) each occupy their own repositories. Developers working across these repos face several compounding problems:

- **No unified CLI.** Each repo has its own build scripts, Makefiles, and ad-hoc commands. There is no single entry point to build, test, deploy, or inspect a SmallAIOS system.
- **Multi-repo coordination is manual.** Cutting a release that spans kernel + ModelGate + other crates requires hand-stitched git tags, changelog assembly, and cross-repo dependency bumps.
- **Parallel development is friction-heavy.** Working on a kernel bugfix while simultaneously prototyping a ModelGate feature means juggling branches, stashes, or multiple clones — none of which compose cleanly.
- **Spec-driven development has no CLI surface.** OpenSpec documents live as markdown files, but creating, validating, and progressing through the spec workflow requires manual file creation and discipline.

Additionally:

- **AI coding assistants are disconnected from the workflow.** Tools like Claude Code, Cursor, and Windsurf can write code but have no structured way to interact with the SmallAIOS workspace, branching model, or spec lifecycle. Developers must manually relay context between their AI assistant and the ecosystem tooling.

These problems grow super-linearly with contributor count and repo count.

## What Changes

Introduce **`smctl`** ("SmallAIOS control"), a unified CLI tool following the Unix `*ctl` convention (kubectl, systemctl, journalctl). `smctl` becomes the single developer-facing entry point for the SmallAIOS ecosystem — usable both as a traditional CLI and as an **MCP (Model Context Protocol) server** that integrates with AI coding assistants.

- **Workspace management** — `smctl workspace init` clones and configures all SmallAIOS repos into a single workspace, using `git worktree` to enable parallel branch work without multiple clones.
- **Git flow integration** — `smctl` enforces a consistent branching model (main, develop, feature/*, release/*, hotfix/*) across all repos, with commands to create, finish, and synchronize flow branches.
- **Git worktree-native parallelism** — `smctl worktree add <feature>` creates linked worktrees so developers can work on multiple features simultaneously with isolated working directories but shared git history.
- **OpenSpec workflow** — `smctl spec new`, `smctl spec ff`, `smctl spec apply`, `smctl spec archive` provide CLI access to the OpenSpec spec-driven development lifecycle.
- **Build orchestration** — `smctl build` drives cross-repo builds with dependency ordering (kernel → runtime → ModelGate → images).
- **Model gateway control** — `smctl gate` subcommands manage ModelGate-specific operations: model registration, routing configuration, health checks, and inference testing.
- **Release management** — `smctl release` automates cross-repo version bumps, changelog generation, tag signing, and artifact publishing.
- **MCP server mode** — `smctl serve --mcp` exposes all smctl capabilities as MCP tools over stdio or SSE transport, enabling AI coding assistants (Claude Code, Cursor, Windsurf, Cline) to autonomously manage workspaces, branches, specs, and builds.

## Capabilities

### New Capabilities

- `smctl-cli` — Top-level CLI binary with subcommand dispatch
- `smctl-workspace` — Multi-repo workspace init, sync, and status
- `smctl-worktree` — Git worktree lifecycle management (add, list, remove, switch)
- `smctl-flow` — Git flow branching model (feature, release, hotfix start/finish)
- `smctl-spec` — OpenSpec workflow commands (new, ff, apply, archive, validate)
- `smctl-build` — Cross-repo build orchestration with dependency graph
- `smctl-gate` — ModelGate management (models, routes, health, test)
- `smctl-release` — Versioning, changelogs, tagging, publishing
- `smctl-config` — Workspace and user-level configuration management
- `smctl-mcp` — MCP server exposing all tools/resources over stdio and SSE transport

### Modified Capabilities

- (None — this is a greenfield tool)

## Impact

### Repository Home

`smctl` lives inside `SmallAIOS/ModelGate` as a Cargo workspace member. ModelGate is both the model gateway and the developer tooling hub for the SmallAIOS ecosystem.

### New Files

```
ModelGate/
├── Cargo.toml                      # Workspace root (adds smctl members)
├── smctl/
│   ├── Cargo.toml                  # smctl binary crate
│   └── src/
│       ├── main.rs                 # Entry point, clap dispatch
│       └── lib.rs                  # Shared types and utilities
├── smctl-workspace/                # Workspace management crate
│   ├── Cargo.toml
│   └── src/lib.rs
├── smctl-worktree/                 # Git worktree operations crate
│   ├── Cargo.toml
│   └── src/lib.rs
├── smctl-flow/                     # Git flow branching crate
│   ├── Cargo.toml
│   └── src/lib.rs
├── smctl-spec/                     # OpenSpec workflow crate
│   ├── Cargo.toml
│   └── src/lib.rs
├── smctl-build/                    # Build orchestration crate
│   ├── Cargo.toml
│   └── src/lib.rs
├── smctl-gate/                     # ModelGate operations crate
│   ├── Cargo.toml
│   └── src/lib.rs
├── smctl-release/                  # Release management crate
│   ├── Cargo.toml
│   └── src/lib.rs
├── smctl-config/                   # Configuration crate
│   ├── Cargo.toml
│   └── src/lib.rs
├── smctl-mcp/                      # MCP server crate (stdio + SSE transport)
│   ├── Cargo.toml
│   └── src/lib.rs
└── openspec/                       # OpenSpec documents (already present)
```

### Affected Repos

| Repository | Impact |
|---|---|
| `SmallAIOS/ModelGate` | **Home repo** — smctl binary + all smctl-* library crates live here |
| `SmallAIOS/SmallAIOS` | Build targets consumed by `smctl build`; git flow conventions adopted |
| Future SmallAIOS repos | Will integrate via `smctl workspace` registry |

### Dependencies

- Rust toolchain (edition 2024)
- `clap` — CLI argument parsing and subcommand dispatch
- `git2` / `libgit2` — Programmatic git operations (worktree, branch, tag)
- `toml` — Configuration file parsing
- `serde` / `serde_json` — Serialization for config and spec files
- `reqwest` — HTTP client for ModelGate health/inference operations
- `dialoguer` — Interactive prompts for flow and release workflows
- `rmcp` — Official Rust MCP SDK (modelcontextprotocol/rust-sdk), JSON-RPC over stdio/SSE
- `cedar-policy` — Cedar authorization policy engine (CNCF Sandbox, Lean 4-verified, native Rust)
- `tokio` — Async runtime for MCP server and SSE transport
- `axum` — HTTP server for SSE-based MCP transport

## References

- [kubectl CLI conventions](https://kubernetes.io/docs/reference/kubectl/)
- [git-flow (AVH edition)](https://github.com/petervanderdoes/gitflow-avh)
- [git worktree documentation](https://git-scm.com/docs/git-worktree)
- [OpenSpec — Fission AI](https://github.com/Fission-AI/OpenSpec)
- [SmallAIOS kernel](https://github.com/SmallAIOS/SmallAIOS)
- [Model Context Protocol (MCP)](https://modelcontextprotocol.io)
- [MCP specification](https://spec.modelcontextprotocol.io)
- [Cedar policy language](https://github.com/cedar-policy) — CNCF Sandbox, Lean 4-verified authorization
- [Cedar formal spec in Lean 4](https://github.com/cedar-policy/cedar-spec)
- [P language](https://github.com/p-org/P) — Microsoft Research, async state machine testing
- [SmallAIOS formal-type-gate](https://github.com/SmallAIOS/SmallAIOS/tree/claude/plan-mac-strategy-uoqzp/openspec/changes/formal-type-gate-v1)
