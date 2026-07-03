# Changelog

All notable changes to ModelGate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-07-03

Formal verification and supply-chain security. Adds the `smctl verify` command tree with Cedar policy checking end-to-end and deep TLA+ model checking, per-repo OpenSpec aggregation, a RustSec advisory gate in CI, and a repo-wide security-hygiene pass that closed every open dependency advisory.

### Added

- **smctl-verify** — formal-verification surface under `smctl verify {policy,model,proof,protocol,discover}`. Cedar runs in-process via the `cedar-policy` SDK with per-file diagnostics; TLA+ (`tlc`), Lean 4 (`lake`), and SPIN (`spin`) ship as shell-out runners. Source roots declared in `[verify.<domain>]` workspace sections (`deny_unknown_fields`). MSGID range `SMCTL-0500..0599`.
- **Deep TLC model checking** (`tla-plus-runner-v1`) — `smctl verify model` resolves TLC via PATH binary or `java -jar tla2tools.jar` (`[verify.model] jar` / `TLA2TOOLS_JAR`), honors sibling `.cfg` configs, runs `-workers auto` with `-metadir` in a temp dir, and parses state-space statistics, invariant/deadlock/liveness/assumption violations, and bounded counter-example traces into a structured `detail` object on each source row. New MSGIDs `SMCTL-0505 VerifyCounterExample`, `SMCTL-0506 VerifyOutputUnparsed`. Hermetic test harness via `SMCTL_VERIFY_TLC_BIN` fixture scripts.
- **Per-repo OpenSpec aggregation** (`openspec-aggregate-v1`) — `smctl spec list/validate/apply/archive/status/ff` walk every registered repo's `openspec/` tree and aggregate; single-spec verbs accept `repo:name` qualification. Closes the 0.2.0 known limitation.
- **Security Audit CI job** — `cargo audit` (RustSec advisory database) on every PR and push, wired into the CI Gate, with a documented `--ignore RUSTSEC-<id>` escape hatch. New `repo-security` capability spec records the baseline: Dependabot alerts + security updates, secret scanning, and push protection all enabled.
- **Canonical OpenSpec format** — capability specs under `openspec/specs/<capability>/spec.md` with requirement/scenario structure; OpenSpec 1.5.0 CLI scaffolding (config, skills, opsx commands) committed. `openspec validate --all --strict` gates the tree.
- **rust-toolchain.toml** — pins the workspace to the stable channel so machine-wide nightly defaults (kept for SmallAIOS kernel targets) never silently build ModelGate; matches CI's `dtolnay/rust-toolchain@stable`.

### Changed

- Shell-out verifiers capture child stdout/stderr instead of inheriting smctl's stdio — raw tool output can no longer corrupt the piped-JSON contract; failure notes fold in the captured output head.
- Every verify subcommand emits the structured `tool_missing` envelope (`error`/`tool`/`install_hint`) in JSON mode, mirroring the `smctl quality` command family.
- Frontend toolchain: `vitest` 2.1.9 → 3.2.6 (critical GHSA-5xrq-8626-4rwp), `vite` 5.4.21 → 6.4.3 (high GHSA-fx2h-pf6j-xcff), `happy-dom` 15.11.7 → 20.10.6 (critical GHSA-37j7-fg3j-429f), transitive `@babel/core` → 7.29.7. `npm audit` reports zero vulnerabilities.
- Rust dependencies: `quinn-proto` → 0.11.15 (high RUSTSEC-2026-0185), `anyhow` → 1.0.103 (unsound RUSTSEC-2026-0190).
- `AGENTS.md` is now generated from `CLAUDE.md` and addressed to AI coding agents generally.

### Fixed

- All five `smctl quality` verbs fell back to the bare current directory when no `.smctl/workspace.toml` exists, so their cargo-ecosystem tools failed from any workspace member directory. New `smctl::find_cargo_root` walks up to the nearest `Cargo.lock`.
- `.gitignore` now covers `*.log` and `.claude/worktrees/`, keeping local tool logs and managed worktrees out of the public repo.

### Known limitations

- Lean 4 and SPIN runners remain exit-code wrappers; deep integration tracked as `lean-proof-runner-v1` and `spin-protocol-runner-v1`. Notably `spin -a` only generates the pan.c verifier — protocol "passed" means Promela parsed, not verified.
- `smctl verify discover` cannot see `[verify.model] jar` (the discover trait takes no context), so jar-only hosts list the model verifier as not installed even though `verify model` runs.
- `smctl quality` still requires the relevant cargo plugin on `PATH`; bundling tracked as `quality-tools-bundle-v1`.

### Specs landed in this release

| Spec | PR |
|---|---|
| formal-methods-v1 | [#23](https://github.com/SmallAIOS/ModelGate/pull/23) |
| openspec-aggregate-v1 | [#25](https://github.com/SmallAIOS/ModelGate/pull/25) |
| security-hygiene-v1 | [#30](https://github.com/SmallAIOS/ModelGate/pull/30) |
| tla-plus-runner-v1 | [#31](https://github.com/SmallAIOS/ModelGate/pull/31) |

## [0.2.0] - 2026-04-25

Major capability expansion. Adds the ModelGate control-plane (CLI + web dashboard), an MCP server, a Rust-native engineering-quality suite, an RFC 5424 logging stack, and a design-system-driven copy contract. End-to-end shakedown against the real SmallAIOS workspace surfaced and fixed one regression in `smctl build`.

### Added

- **smctl-gate** — ModelGate control-plane client. `smctl gate {status,models list/add/remove,routes list/set,test,logs}` with reqwest-backed JSON + SSE, multipart streaming uploads, and tokio-signal Ctrl+C on `gate logs --follow`. wiremock-backed integration tests cover every endpoint.
- **modelgate-web** — Axum server that serves an embedded React SPA dashboard plus a JSON / SSE proxy at `/api/*`. Launched via `smctl gate web [--host] [--port] [--open]`. Default bind 127.0.0.1:9378.
- **ui/modelgate-web** — Vite + React 18 + TypeScript dashboard. Five screens (Overview / Models / Routes / Inference / Terminal). React Query for server state. Toaster + ConfirmDialog primitives. XHR upload with real progress for `Register model`. JSON-validated `<JsonEditor>` for `Run inference`.
- **smctl-mcp** — MCP server exposing 20 tools and 5 resources to AI coding assistants. Stdio and SSE transports.
- **smctl-quality** — engineering-quality wrappers under `smctl quality`: `audit` (cargo-audit), `deps` (cargo-machete), `unsafe` (cargo-geiger), `dsm` (cargo-modules), `complexity` (rust-code-analysis).
- **smctl-log** — RFC 5424 tracing subscriber. Multi-transport (stderr / file / Unix syslog socket), MSGID catalog with reserved ranges (`SMCTL-0001..0099` core, `SMCTL-0200..0299` MCP, `SMCTL-0300..0399` web, `SMCTL-0400..0499` quality). `[logging]` and `[gate]` sections in `workspace.toml`.
- **Design system** — voice rules, tokens, iconography under `ui/`. Reflexively applied to every CLI output, error message, and SPA label. Three-part error remediation pattern (`smctl-errors-v1`).
- **Quality gates** — Codecov + SonarCloud reporting on every PR via the SmallAIOS org's `CODECOV_TOKEN` / `SONAR_TOKEN` secrets. Per-flag thresholds (rust / frontend) in `codecov.yml`. Patch coverage gate set at 70%.
- **Pre-commit hooks** — fast checks (cargo fmt --check, whitespace, YAML) on `git commit`; heavy checks (clippy, workspace tests, frontend typecheck and vitest) on `git push`. Exact mirror of CI.
- **CI** — frontend build job uploads `dist/` as an artifact; Rust jobs download it before invoking cargo. `actions/checkout@v5`, `setup-node@v5`, artifact actions @v5. `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` set workflow-wide.

### Fixed

- `smctl build <repo> --dry-run` ignored the repo argument and printed the full workspace order. Surfaced by [#18](https://github.com/SmallAIOS/ModelGate/pull/18) (smallaios-integration-v1 shakedown). Extracted `smctl_build::resolve_build_subset()` so the dry-run path and the real build path share the same filter. Four new unit tests in `smctl-build`.

### Known limitations

- `smctl spec list` (and validate / archive / status) operate on a workspace-level `openspec/`. Does not yet aggregate across each registered repo's `openspec/`. Tracked as `openspec-aggregate-v1`.
- `smctl flow feature list` only matches the `feature/` prefix; OpenSpec changes use `change/` and don't appear there. Tracked as `flow-feature-list-includes-change-branches-v1`.
- `smctl serve --mcp --stdio` emits a noisy "unhandled error" line when stdin closes. Tracked as `mcp-stdio-clean-eof-shutdown-v1`.
- `smctl quality` requires the relevant cargo plugin to be on `PATH`. Bundling these as a documented prereq + CI step is tracked as `quality-tools-bundle-v1`.

### Specs landed in this release

| Spec | PR |
|---|---|
| design-system-v1 | [#1](https://github.com/SmallAIOS/ModelGate/pull/1) |
| smctl-copy-v1 | [#2](https://github.com/SmallAIOS/ModelGate/pull/2) |
| smctl-errors-v1 | [#3](https://github.com/SmallAIOS/ModelGate/pull/3) |
| smctl-logging-v1 | [#4](https://github.com/SmallAIOS/ModelGate/pull/4) |
| smctl-mcp-v1 | [#8](https://github.com/SmallAIOS/ModelGate/pull/8) |
| safety-quality-v1 | [#9](https://github.com/SmallAIOS/ModelGate/pull/9) |
| smctl-gate-v1 | [#10](https://github.com/SmallAIOS/ModelGate/pull/10) |
| modelgate-web-v1 | [#12](https://github.com/SmallAIOS/ModelGate/pull/12) |
| modelgate-web-actions-v1 | [#14](https://github.com/SmallAIOS/ModelGate/pull/14) |
| smallaios-integration-v1 | [#18](https://github.com/SmallAIOS/ModelGate/pull/18) |

[0.2.0]: https://github.com/SmallAIOS/ModelGate/compare/v0.1.3...v0.2.0

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
