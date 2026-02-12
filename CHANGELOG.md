# Changelog

All notable changes to ModelGate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-02-12

Initial release of `smctl` (SmallAIOS Control) CLI tool.

### Added

- **smctl CLI** with subcommand hierarchy: `workspace`, `flow`, `spec`, `build`, `config`
- **Workspace management** (`smctl workspace init/add/remove/status`) — multi-repo workspace configuration with manifest tracking
- **Git worktree support** (`smctl workspace worktree add/remove/list`) — parallel branch development using git worktrees
- **Git flow enforcement** (`smctl flow start/finish/status`) — consistent branching model (main, develop, feature/\*, release/\*, hotfix/\*) with two-phase validate-then-execute
- **OpenSpec workflow** (`smctl spec new/ff/apply/validate/archive`) — spec-driven development lifecycle management
- **Build orchestration** (`smctl build`) — dependency-ordered cross-repo builds with topological sort
- **Configuration system** (`smctl config get/set/show`) — workspace-level and user-level config with JSON/YAML output
- **OpenSpec design documents** — proposal, design, CLI interface spec, git flow spec, worktree spec, OpenSpec workflow spec, MCP server spec (deferred)

### Architecture

- 5-crate Cargo workspace: `smctl`, `smctl-workspace`, `smctl-flow`, `smctl-spec`, `smctl-build`
- Rust edition 2024, resolver v3
- 21 unit tests covering all crates
- Compatible with SmallAIOS-Design v0.1.0

[0.1.0]: https://github.com/SmallAIOS/ModelGate/releases/tag/v0.1.0
