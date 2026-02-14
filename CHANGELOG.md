# Changelog

All notable changes to ModelGate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2026-02-13

### Added

- **`--parallel` build flag** — concurrent builds using thread-scoped parallelism with dependency-level grouping
- **Merge conflict detection** — `feature_check_merge()` does dry-run merge to detect conflicts before finishing
- **Build levels** — `resolve_build_levels()` groups repos into concurrent execution tiers
- **Worktree integration tests** — add/list/remove lifecycle with real git repos
- 8 new tests: parallel build levels, merge conflict detection, worktree lifecycle (59 tests total)

### Changed

- Updated task tracking: 46 of 57 tasks complete (remaining 11 are deferred)
- Per-repo build timing now tracked in `BuildResult.duration_ms`

[0.1.3]: https://github.com/SmallAIOS/ModelGate/compare/v0.1.2...v0.1.3

## [0.1.2] - 2026-02-13

### Added

- **Integration tests** — 27 new tests using real git repos for workspace init/status, flow feature start/finish, and CLI end-to-end (51 tests total)
- **README.md** — installation, quickstart, subcommand reference, workspace.toml reference
- **CLI tests** — 16 assert_cmd tests covering workspace, spec, config, alias, and completions commands

### Fixed

- **`release` subcommand `version` arg** — renamed to avoid clap conflict with `--version` flag

### Changed

- Updated task tracking: 43 of 57 tasks now complete

[0.1.2]: https://github.com/SmallAIOS/ModelGate/compare/v0.1.1...v0.1.2

## [0.1.1] - 2026-02-13

### Added

- **GitHub Actions CI** — format check, clippy lint, test, build, and gate jobs
- **`smctl spec ff`** — fast-forward validation showing document completeness and task progress
- **`smctl spec apply`** — lists pending and completed tasks from tasks.md
- **Spec-flow binding** — `spec new` auto-creates feature branch, `spec archive` auto-finishes it
- 3 new tests for spec phase detection and validation edge cases (24 tests total)

### Changed

- Updated task tracking: 33 of 56 tasks now complete
- Marked ModelGate Control and MCP Server sections as deferred

[0.1.1]: https://github.com/SmallAIOS/ModelGate/compare/v0.1.0...v0.1.1

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
