# smctl — Tasks

## Project Bootstrap

- [x] Initialize Cargo workspace with `smctl` binary crate and clap derive API
- [x] Add `--json` and `--dry-run` global flags with formatter/execution traits
- [x] Configure CI (GitHub Actions): build, test, clippy, fmt
- [x] Set up integration test harness (temp git repos)
- [x] Create `.smctl/` config directory structure and shell completions

## Workspace Management (`smctl-workspace`)

- [x] Define `workspace.toml` schema and parser (serde + toml)
- [x] Implement `smctl workspace init` — clone repos, create `.smctl/`
- [x] Implement `smctl workspace add` / `remove` — manage repos in manifest
- [x] Implement `smctl workspace status` — show all repo branch + dirty state
- [x] Implement `smctl workspace sync` — fetch/pull all repos
- [x] Write unit + integration tests for workspace operations

## Git Worktree Management

- [x] Implement git worktree operations via git CLI
- [x] Implement `smctl worktree add` — create linked worktrees across repos
- [x] Implement `smctl worktree list` — enumerate worktree sets with status
- [x] Implement `smctl worktree remove` — clean up worktrees and optionally branches
- [x] Write unit tests for worktree lifecycle

## Git Flow (`smctl-flow`)

- [x] Implement `smctl flow init` — create develop branch in all repos
- [x] Implement `smctl flow feature start` / `finish` / `list` across repos
- [x] Implement `smctl flow release start` / `finish` with tagging
- [x] Implement `smctl flow hotfix start` / `finish`
- [x] Implement cross-repo two-phase validate-then-execute pattern
- [ ] Implement merge conflict detection, `--repos` filter
- [x] Write integration tests for feature and release lifecycles

## OpenSpec Integration (`smctl-spec`)

- [x] Implement `smctl spec new` — scaffold openspec feature folder + git branch
- [x] Create document templates (proposal.md, design.md, tasks.md scaffolds)
- [x] Implement `smctl spec ff` / `apply` / `validate` / `status` / `list`
- [x] Implement `smctl spec archive` — move to archive
- [x] Bind spec new → flow feature start; spec archive → flow feature finish
- [x] Write unit tests for spec lifecycle

## Build Orchestration (`smctl-build`)

- [x] Define per-repo build/test commands in workspace.toml schema
- [x] Implement dependency graph resolution from `depends_on` fields
- [x] Implement `smctl build` with `--test`, `--clean` flags
- [ ] Wire up `--parallel` flag for concurrent builds
- [x] Write unit tests for build ordering

## ModelGate Control (`smctl-gate`) — Deferred

- [ ] Define ModelGate API client (reqwest-based)
- [ ] Implement `smctl gate status` / `models list` / `models add`
- [ ] Implement `smctl gate routes list` / `routes set` / `test` / `logs`
- [ ] Write integration tests with mock ModelGate server

## MCP Server (`smctl-mcp`) — Deferred

- [ ] Integrate rmcp SDK and implement server handler with stdio transport
- [ ] Implement SSE transport (axum HTTP server)
- [ ] Register all workspace/worktree/flow/spec/build/gate tools
- [ ] Implement MCP resources and error code mapping
- [ ] Write integration tests: MCP tool call → JSON response

## Configuration

- [x] Implement three-tier config resolution (CLI > workspace > user)
- [x] Implement `smctl config show` / `set` / `get` / `edit`

## Convenience Aliases

- [x] Implement `smctl feat` / `done` / `ss` / `sb` shorthand aliases

## Documentation

- [x] Write README.md with installation, quickstart, and workspace.toml reference
- [ ] Document MCP integration guide for Claude Code, Cursor, Windsurf

## Verify

- [x] All unit tests pass (51 tests)
- [x] `smctl workspace init` → `status` works end-to-end (integration test)
- [x] `smctl flow feature start` → `finish` works across multiple repos (integration test)
- [ ] `smctl worktree add` → `list` → `remove` lifecycle works (needs integration test)
- [x] `smctl spec new` → `validate` → `archive` lifecycle works
- [x] `smctl build` correctly orders dependencies
- [ ] `smctl serve --mcp --stdio` responds to MCP initialize handshake (deferred)
- [x] `--dry-run` and `--json` flags work correctly
- [x] clippy passes with no warnings; `cargo fmt` reports no changes
