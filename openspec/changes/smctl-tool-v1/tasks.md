# smctl — Tasks

## Project Bootstrap

- [ ] Initialize Cargo workspace with `smctl` binary crate and clap derive API
- [ ] Add `--json` and `--dry-run` global flags with formatter/execution traits
- [ ] Configure CI (GitHub Actions): build, test, clippy, fmt
- [ ] Set up integration test harness (temp git repos)
- [ ] Create `.smctl/` config directory structure and shell completions

## Workspace Management (`smctl-workspace`)

- [ ] Define `workspace.toml` schema and parser (serde + toml)
- [ ] Implement `smctl workspace init` — clone repos, create `.smctl/`
- [ ] Implement `smctl workspace add` / `remove` — manage repos in manifest
- [ ] Implement `smctl workspace status` — show all repo branch + dirty state
- [ ] Implement `smctl workspace sync` — fetch/pull all repos
- [ ] Write unit + integration tests for workspace operations

## Git Worktree Management (`smctl-worktree`)

- [ ] Implement git worktree operations via git2/libgit2 (or git CLI fallback)
- [ ] Implement `smctl worktree add` — create linked worktrees across repos
- [ ] Implement `smctl worktree list` — enumerate worktree sets with status
- [ ] Implement `smctl worktree remove` — clean up worktrees and optionally branches
- [ ] Write unit + integration tests for worktree lifecycle

## Git Flow (`smctl-flow`)

- [ ] Implement `smctl flow init` — create develop branch in all repos
- [ ] Implement `smctl flow feature start` / `finish` / `list` across repos
- [ ] Implement `smctl flow release start` / `finish` with tagging and changelog
- [ ] Implement `smctl flow hotfix start` / `finish`
- [ ] Implement cross-repo two-phase validate-then-execute pattern
- [ ] Implement merge conflict detection, `--repos` filter
- [ ] Write integration tests for feature and release lifecycles

## OpenSpec Integration (`smctl-spec`)

- [ ] Implement `smctl spec new` — scaffold openspec feature folder + git branch
- [ ] Create document templates (proposal.md, design.md, tasks.md scaffolds)
- [ ] Implement `smctl spec ff` / `apply` / `validate` / `status` / `list`
- [ ] Implement `smctl spec archive` — move to archive, trigger merge
- [ ] Bind spec new → flow feature start; spec archive → flow feature finish
- [ ] Write unit + integration tests for spec lifecycle

## Build Orchestration (`smctl-build`)

- [ ] Define per-repo build/test commands in workspace.toml schema
- [ ] Implement dependency graph resolution from `depends_on` fields
- [ ] Implement `smctl build` with `--parallel`, `--test`, `--clean` flags
- [ ] Write unit + integration tests for build ordering

## ModelGate Control (`smctl-gate`)

- [ ] Define ModelGate API client (reqwest-based)
- [ ] Implement `smctl gate status` / `models list` / `models add`
- [ ] Implement `smctl gate routes list` / `routes set` / `test` / `logs`
- [ ] Write integration tests with mock ModelGate server

## MCP Server (`smctl-mcp`)

- [ ] Integrate rmcp SDK and implement server handler with stdio transport
- [ ] Implement SSE transport (axum HTTP server)
- [ ] Register all workspace/worktree/flow/spec/build/gate tools
- [ ] Implement MCP resources and error code mapping
- [ ] Write integration tests: MCP tool call → JSON response

## Configuration (`smctl-config`)

- [ ] Implement three-tier config resolution (CLI > workspace > user)
- [ ] Implement `smctl config show` / `set` / `get` / `edit`

## Convenience Aliases

- [ ] Implement `smctl feat` / `done` / `ss` / `sb` shorthand aliases

## Documentation

- [ ] Write README.md with installation, quickstart, and workspace.toml reference
- [ ] Document MCP integration guide for Claude Code, Cursor, Windsurf

## Verify

- [ ] All unit and integration tests pass
- [ ] `smctl workspace init` → `status` works end-to-end
- [ ] `smctl flow feature start` → `finish` works across multiple repos
- [ ] `smctl worktree add` → `list` → `remove` lifecycle works
- [ ] `smctl spec new` → `validate` → `archive` lifecycle works
- [ ] `smctl build` correctly orders dependencies
- [ ] `smctl serve --mcp --stdio` responds to MCP initialize handshake
- [ ] `--dry-run` and `--json` flags work correctly
- [ ] clippy passes with no warnings; `cargo fmt` reports no changes
