# smctl — Tasks

## Project Bootstrap

- [ ] Initialize Cargo workspace with `smctl` binary crate
- [ ] Set up clap (derive API) with top-level subcommand enum
- [ ] Configure CI (GitHub Actions): build, test, clippy, fmt
- [ ] Add `--json` global output flag with formatter trait
- [ ] Add `--dry-run` global flag with execution trait
- [ ] Set up integration test harness (temp git repos)
- [ ] Add shell completion generation (bash, zsh, fish)
- [ ] Create `.smctl/` config directory structure

## Workspace Management (`smctl-workspace`)

- [ ] Define `workspace.toml` schema and parser (serde + toml)
- [ ] Implement `smctl workspace init` — clone repos, create `.smctl/`
- [ ] Implement `smctl workspace add` — add a repo to workspace manifest
- [ ] Implement `smctl workspace remove` — remove a repo from manifest
- [ ] Implement `smctl workspace status` — show all repo branch + dirty state
- [ ] Implement `smctl workspace sync` — fetch/pull all repos
- [ ] Write unit tests for workspace config parsing
- [ ] Write integration tests for workspace init with real git repos

## Git Worktree Management (`smctl-worktree`)

- [ ] Implement git worktree operations via git2/libgit2 (or git CLI fallback)
- [ ] Implement `smctl worktree add` — create linked worktrees across repos
- [ ] Implement `smctl worktree list` — enumerate worktree sets with status
- [ ] Implement `smctl worktree remove` — clean up worktrees and optionally branches
- [ ] Implement `smctl worktree cd` — print worktree path for shell integration
- [ ] Handle branch-already-checked-out errors gracefully
- [ ] Write unit tests for worktree path resolution
- [ ] Write integration tests for worktree add/remove lifecycle

## Git Flow (`smctl-flow`)

- [ ] Implement `smctl flow init` — create develop branch in all repos
- [ ] Implement `smctl flow feature start` — create feature branches across repos
- [ ] Implement `smctl flow feature finish` — merge feature → develop, delete branch
- [ ] Implement `smctl flow feature list` — show active feature branches
- [ ] Implement `smctl flow release start` — create release branch from develop
- [ ] Implement `smctl flow release finish` — merge → main + develop, tag, changelog
- [ ] Implement `smctl flow hotfix start` — create hotfix from main
- [ ] Implement `smctl flow hotfix finish` — merge → main + develop, patch tag
- [ ] Implement cross-repo two-phase validate-then-execute pattern
- [ ] Implement merge conflict detection and reporting
- [ ] Add `--repos` filter for scoping operations to specific repos
- [ ] Write integration tests for full feature start → finish lifecycle
- [ ] Write integration tests for release lifecycle with tagging

## OpenSpec Integration (`smctl-spec`)

- [ ] Implement `smctl spec new` — scaffold openspec feature folder + git branch
- [ ] Create document templates (proposal.md, design.md, tasks.md scaffolds)
- [ ] Implement `smctl spec ff` — check document completeness, report gaps
- [ ] Implement `smctl spec apply` — parse tasks.md checkboxes, show progress
- [ ] Implement `smctl spec archive` — move to archive, update metadata, trigger merge
- [ ] Implement `smctl spec validate` — quality checks against spec criteria
- [ ] Implement `smctl spec status` — summary view (single spec or all)
- [ ] Implement `smctl spec list` — list active and archived specs
- [ ] Bind spec new → flow feature start (auto-create branch)
- [ ] Bind spec archive → flow feature finish (auto-merge)
- [ ] Write unit tests for tasks.md checkbox parser
- [ ] Write integration tests for spec new → validate → archive lifecycle

## Build Orchestration (`smctl-build`)

- [ ] Define per-repo build/test commands in workspace.toml schema
- [ ] Implement dependency graph resolution from `depends_on` fields
- [ ] Implement `smctl build` — sequential dependency-ordered build
- [ ] Implement `smctl build --parallel` — concurrent independent builds
- [ ] Implement `smctl build <repo>` — build specific repo + dependencies
- [ ] Implement `smctl build --test` — run tests after build
- [ ] Implement `smctl build --clean` — clean before building
- [ ] Capture and report build output (pass-through + summary)
- [ ] Write unit tests for dependency graph resolution
- [ ] Write integration tests for build ordering

## ModelGate Control (`smctl-gate`)

- [ ] Define ModelGate API client (reqwest-based)
- [ ] Implement `smctl gate status` — health check against running instance
- [ ] Implement `smctl gate models list` — enumerate registered models
- [ ] Implement `smctl gate models add` — register ONNX model
- [ ] Implement `smctl gate models remove` — unregister model
- [ ] Implement `smctl gate routes list` — show routing table
- [ ] Implement `smctl gate routes set` — configure model → endpoint route
- [ ] Implement `smctl gate test` — run inference with test input
- [ ] Implement `smctl gate logs` — stream logs (with --follow)
- [ ] Write integration tests with mock ModelGate server

## MCP Server (`smctl-mcp`)

- [ ] Integrate MCP SDK / implement JSON-RPC 2.0 protocol handler
- [ ] Implement stdio transport (read stdin, write stdout)
- [ ] Implement SSE transport (axum HTTP server + SSE stream)
- [ ] Implement streamable HTTP transport
- [ ] Register all workspace tools (init, status, sync)
- [ ] Register all worktree tools (add, list, remove)
- [ ] Register all flow tools (feature/release/hotfix start/finish)
- [ ] Register all spec tools (new, ff, apply, archive, validate, status)
- [ ] Register build tools
- [ ] Register gate tools (status, models, routes, test)
- [ ] Implement MCP resources for workspace state
- [ ] Implement MCP resources for spec documents
- [ ] Implement MCP resources for gate models/routes
- [ ] Implement resource subscription + change notifications
- [ ] Implement MCP error codes mapping (smctl errors → JSON-RPC errors)
- [ ] Write integration tests: MCP tool call → smctl action → JSON response
- [ ] Test with Claude Code (stdio transport)
- [ ] Test with Cursor (stdio transport)
- [ ] Document MCP configuration for each supported AI assistant

## Configuration (`smctl-config`)

- [ ] Implement three-tier config resolution (CLI > workspace > user)
- [ ] Implement `smctl config show` — print effective config
- [ ] Implement `smctl config set` / `smctl config get`
- [ ] Implement `smctl config edit` — open in $EDITOR
- [ ] Write unit tests for config layering and override logic

## Convenience Aliases

- [ ] Implement `smctl feat <name>` → flow feature start + worktree add
- [ ] Implement `smctl done <name>` → worktree remove + flow feature finish
- [ ] Implement `smctl ss <name>` → spec new
- [ ] Implement `smctl sb` → build

## Formal Methods

- [ ] Write TLA+ spec for git flow state machine (branch lifecycle transitions)
- [ ] Write TLA+ spec for cross-repo merge ordering (validate-then-execute)
- [ ] Model check git flow spec for illegal state reachability
- [ ] Add `smctl build --verify` flag to invoke TLA+ and Lean checks
- [ ] Add formal artifact presence checks to `smctl spec validate`
- [ ] Document formal methods integration in design.md

## Documentation

- [ ] Write README.md with installation and quickstart
- [ ] Write man pages (generated from clap)
- [ ] Document all subcommands with examples
- [ ] Document MCP integration guide for each AI assistant
- [ ] Document workspace.toml reference
- [ ] Add CONTRIBUTING.md with development setup

## Verify

- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] `smctl workspace init` → `status` works end-to-end
- [ ] `smctl flow feature start` → `finish` works across multiple repos
- [ ] `smctl worktree add` → `list` → `remove` lifecycle works
- [ ] `smctl spec new` → `validate` → `archive` lifecycle works
- [ ] `smctl build` correctly orders dependencies
- [ ] `smctl serve --mcp --stdio` responds to MCP initialize handshake
- [ ] MCP tools return valid JSON matching tool schemas
- [ ] Claude Code can discover and invoke smctl MCP tools
- [ ] Cursor can discover and invoke smctl MCP tools
- [ ] `--dry-run` flag works for all destructive operations
- [ ] `--json` output is parseable for all commands
- [ ] Shell completions work for bash, zsh, fish
- [ ] clippy passes with no warnings
- [ ] `cargo fmt` reports no changes needed
