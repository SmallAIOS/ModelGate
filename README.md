# ModelGate

[![CI](https://github.com/SmallAIOS/ModelGate/actions/workflows/ci.yml/badge.svg?branch=develop)](https://github.com/SmallAIOS/ModelGate/actions/workflows/ci.yml)
[![codecov](https://codecov.io/gh/SmallAIOS/ModelGate/branch/develop/graph/badge.svg)](https://codecov.io/gh/SmallAIOS/ModelGate)
[![Quality Gate Status](https://sonarcloud.io/api/project_badges/measure?project=SmallAIOS_ModelGate&metric=alert_status)](https://sonarcloud.io/summary/new_code?id=SmallAIOS_ModelGate)

**smctl** (SmallAIOS Control) is a unified CLI for managing multi-repo workspaces, git flow branching, spec-driven development, and dependency-ordered builds.

## Installation

```bash
# Clone and build from source
git clone https://github.com/SmallAIOS/ModelGate.git
cd ModelGate
cargo install --path smctl
```

Requires Rust 2024 edition (1.85+).

### Pre-commit hooks (recommended)

```bash
brew install pre-commit          # if not already on PATH
pre-commit install --install-hooks
pre-commit install --hook-type pre-push
```

`git commit` then runs the fast set (formatter + whitespace + YAML) and `git push` runs the heavier set that mirrors CI (clippy, workspace tests, frontend typecheck and vitest). Bypass once with `--no-verify` if you genuinely know what you're doing.

## Quickstart

```bash
# Initialize a workspace
smctl workspace init --name my-project

# Add repositories
smctl workspace add https://github.com/org/repo-a.git --name repo-a
smctl workspace add https://github.com/org/repo-b.git --name repo-b --path b

# Check status across all repos
smctl workspace status

# Start a feature using git flow
smctl flow init                       # ensure develop branch exists
smctl flow feature start my-feature   # create feature/my-feature across repos

# Create a spec-driven feature
smctl spec new my-feature             # scaffold openspec documents + git branch
smctl spec ff my-feature              # fast-forward: check document completeness
smctl spec apply my-feature           # list pending/completed tasks
smctl spec validate my-feature        # validate required sections

# Build in dependency order
smctl build
smctl build --test                    # build + run tests
smctl build repo-a                    # build specific repo + dependencies

# Finish and archive
smctl spec archive my-feature         # archive spec + merge feature branch
```

## Subcommands

| Command | Description |
|---|---|
| `workspace init` | Initialize a new workspace with `.smctl/workspace.toml` |
| `workspace add` | Add a repository to the workspace manifest |
| `workspace remove` | Remove a repository from the manifest |
| `workspace status` | Show branch + dirty state for all repos |
| `workspace sync` | Fetch/pull all repositories |
| `worktree add` | Create linked worktrees across repos |
| `worktree list` | Enumerate active worktree sets |
| `worktree remove` | Remove a worktree set |
| `flow init` | Create develop branch in all repos |
| `flow feature start/finish/list` | Feature branch operations |
| `flow release start/finish/list` | Release branch operations |
| `flow hotfix start/finish/list` | Hotfix branch operations |
| `spec new` | Scaffold openspec feature folder + branch |
| `spec ff` | Fast-forward validation (document completeness + task progress) |
| `spec apply` | List pending and completed tasks |
| `spec validate` | Check required sections in spec documents |
| `spec list` | List all specs (active + archived) |
| `spec archive` | Move spec to archive + finish feature branch |
| `build` | Build repos in dependency order |
| `quality audit/deps/unsafe/dsm/complexity` | Engineering-quality checks |
| `gate status` | Show health and summary metadata of a running ModelGate instance |
| `gate models list/add/remove` | Inspect and manage registered models |
| `gate routes list/set` | Inspect and configure the inference routing table |
| `gate test <model> --input <file>` | Run a test inference against a model |
| `gate logs [--follow]` | Stream ModelGate logs (Ctrl+C to exit) |
| `gate web [--host] [--port] [--open]` | Start the ModelGate dashboard in a browser |
| `verify policy/model/proof/protocol` | Run formal-verification suites (Cedar / TLA+ / Lean 4 / SPIN/Promela) |
| `verify discover` | Enumerate which formal-verification tools are reachable on PATH |
| `config show/set/get` | Configuration management |
| `completions <shell>` | Generate shell completions (bash, zsh, fish, etc.) |

### Aliases

| Alias | Equivalent |
|---|---|
| `smctl feat <name>` | `flow feature start` + `worktree add` |
| `smctl done <name>` | `worktree remove` + `flow feature finish` |
| `smctl ss <name>` | `spec new` |
| `smctl sb` | `build` |

## Global Flags

| Flag | Description |
|---|---|
| `-w, --workspace <PATH>` | Override workspace root (default: auto-detect) |
| `--json` | Output in JSON format |
| `--dry-run` | Show what would be done without executing |
| `-v, --verbose` | Increase verbosity (repeatable: -v, -vv, -vvv) |
| `-q, --quiet` | Suppress non-error output |
| `--no-color` | Disable colored output |
| `--log-level <LEVEL>` | `error` / `warn` / `info` / `debug` / `trace` (also `SMCTL_LOG_LEVEL`) |
| `--log-file <PATH>` | Append RFC 5424 events to a file (also `SMCTL_LOG_FILE`) |
| `--log-syslog` | Emit to local syslog Unix socket, Unix only (also `SMCTL_LOG_SYSLOG`) |

## workspace.toml Reference

The workspace manifest lives at `.smctl/workspace.toml`:

```toml
[workspace]
name = "my-project"
root = "."                    # workspace root (default: ".")

[[repos]]
name = "SmallAIOS"
url = "https://github.com/SmallAIOS/SmallAIOS"
path = "smallaios"            # local path (default: repo name)
default_branch = "main"
smctl_home = false            # true if this repo contains smctl
build_cmd = "cargo build"     # custom build command
test_cmd = "cargo test"       # custom test command
clean_cmd = "cargo clean"     # custom clean command
depends_on = []               # build ordering dependencies

[[repos]]
name = "ModelGate"
url = "https://github.com/SmallAIOS/ModelGate"
default_branch = "main"
smctl_home = true
depends_on = ["SmallAIOS"]    # built after SmallAIOS

[flow]
main_branch = "main"          # default: "main"
develop_branch = "develop"    # default: "develop"
feature_prefix = "feature/"   # default: "feature/"
release_prefix = "release/"   # default: "release/"
hotfix_prefix = "hotfix/"     # default: "hotfix/"

[worktree]
base_dir = ".worktrees"       # default: ".worktrees"

[spec]
openspec_dir = "openspec"     # default: "openspec"

[logging]
transports = ["stderr"]       # any of "stderr", "file", "syslog"
file = "/var/log/smctl.log"   # only used when "file" is in transports
facility = "local0"           # local0..local7 or daemon
level = "info"                # error / warn / info / debug / trace

[verify.policy]
sources = ["security/policies/*.cedar"]   # globs relative to each repo
fail_on = "any"                            # "any" or "error"

[verify.model]
specs = ["formal/tla/*.tla"]               # TLA+ specs (`tlc` runner)
fail_on = "any"

[verify.proof]
roots = ["formal/lean"]                    # Lean 4 project roots (`lake build`)
fail_on = "any"

[verify.protocol]
specs = ["formal/spin/*.pml"]              # SPIN/Promela specs (`spin -a`)
fail_on = "any"
```

## Architecture

6-crate Cargo workspace:

- **smctl** — CLI binary (clap derive, subcommand dispatch)
- **smctl-workspace** — workspace manifest, repo status, worktree management
- **smctl-flow** — git flow branching (feature, release, hotfix lifecycle)
- **smctl-spec** — OpenSpec workflow (scaffold, validate, archive)
- **smctl-build** — dependency-ordered build orchestration
- **smctl-log** — RFC 5424 tracing subscriber and MSGID catalog
- **smctl-mcp** — MCP server exposing smctl tools and resources to AI agents
- **smctl-quality** — engineering-quality checks (audit, deps, unsafe, dsm, complexity)
- **smctl-gate** — ModelGate control-plane client (status, models, routes, inference, logs)
- **modelgate-web** — Axum server serving the React dashboard SPA plus a JSON/SSE proxy to ModelGate
- **smctl-verify** — formal-verification surface (Cedar policy, TLA+ model, Lean 4 proof, SPIN/Promela protocol)

## Logging

All `smctl` log events conform to [RFC 5424](https://datatracker.ietf.org/doc/html/rfc5424) (The Syslog Protocol). The `smctl-log` crate owns the subscriber, the wire format, and the MSGID catalog; every other crate uses the `tracing` macros and inherits the format.

Three transports are available and can run together:

- **stderr** (default) — RFC 5424 lines to `std::io::stderr`
- **file** — set via `--log-file <path>` or `SMCTL_LOG_FILE`; append-only, creates parent directories
- **syslog** — set via `--log-syslog` or `SMCTL_LOG_SYSLOG`; local Unix socket (`/dev/log` on Linux, `/var/run/syslog` on macOS). Unsupported on Windows — the flag warns and falls back to stderr. On open-failure the subscriber emits one `SMCTL-0099` warning and continues on stderr.

The level is set with `--log-level <error|warn|info|debug|trace>` or `SMCTL_LOG_LEVEL`; `-v` / `-vv` bump up, `-q` / `-qq` bump down. Defaults applied in precedence order: CLI flags, then env vars, then `[logging]` in `workspace.toml`, then built-in defaults (stderr only, info level, `local0` facility).

The canonical MSGID catalog (`SMCTL-0001` through `SMCTL-0099`) lives in [`openspec/changes/smctl-logging-v1/specs/logging.md`](openspec/changes/smctl-logging-v1/specs/logging.md) — that document is the authoritative wire-format contract.

## Web UI

`smctl gate web` starts a local dashboard served by the [`modelgate-web`](modelgate-web/) crate. The server embeds a React SPA (built from [`ui/modelgate-web/`](ui/modelgate-web/)) and exposes a JSON + SSE proxy at `/api/*` that fronts the same ModelGate instance `smctl gate` already talks to.

```bash
smctl gate web --open        # bind 127.0.0.1:9378 and launch the default browser
smctl gate web --port 9400   # use a different port
```

The dashboard mirrors the CLI surface: Overview reads `/api/health`, Models reads `/api/models`, Policy and Terminal render placeholder states until the upstream endpoints ship. For frontend development run `npm run dev` in `ui/modelgate-web/` side-by-side with `smctl gate web` — Vite proxies `/api/*` to `127.0.0.1:9378` on port `5173`.

The default bind is `127.0.0.1`. Non-loopback binds emit a warning because there is no authentication layer yet.

## Verification

`smctl verify` exposes four formal-verification domains plus a discovery sweep, backed by the [`smctl-verify`](smctl-verify/) crate:

| Subcommand | Tool | Domain |
|---|---|---|
| `verify policy` | [Cedar](https://www.cedarpolicy.com) (Rust SDK, end-to-end) | Authorization policies (RBAC / ABAC / MAC) |
| `verify model` | TLA+ (`tlc`, shell-out) | Behavioural / temporal properties |
| `verify proof` | Lean 4 (`lake build`, shell-out) | Theorem proving |
| `verify protocol` | SPIN/Promela (`spin -a`, shell-out) | Concurrent protocol verification |
| `verify discover` | — | Lists which of the above are reachable on PATH |

Each verifier reads its source roots from the matching `[verify.<domain>]` section in `workspace.toml` (see the reference above). With no section, the verifier exits 0 with `no sources configured`. Cedar runs end-to-end inside the `smctl` process; the other three are exit-code level shell-out wrappers in v1 — deep output parsing is queued for follow-up changes (`tla-plus-runner-v1`, `lean-proof-runner-v1`, `spin-protocol-runner-v1`).

```bash
smctl verify discover                 # which verifiers are installed?
smctl verify policy --json            # Cedar policy verification with JSON output
smctl verify model --strict           # treat warnings as errors when computing the gate
smctl --dry-run verify proof          # preview which Lean roots would run
```

Lifecycle events emit on `SMCTL-0501` (`VerifyStarted`), `SMCTL-0502` (`VerifySucceeded`), `SMCTL-0503` (`VerifyFailed`), `SMCTL-0504` (`VerifierMissing`).

## Design system

The canonical design source for SmallAIOS / ModelGate / `smctl` surfaces — CLI output, docs, slides, future web UI — lives in [`ui/`](ui/). Start with [`ui/README.md`](ui/README.md) for voice, tokens, iconography, and brand rules. The contract form is declared in [`openspec/changes/design-system-v1/specs/design-system.md`](openspec/changes/design-system-v1/specs/design-system.md); when the two disagree, the spec wins.

Contributors using Claude Code auto-load the rules via the `smallaios-design` skill at [`.claude/skills/smallaios-design/`](.claude/skills/smallaios-design/).

Fonts (IBM Plex Sans, JetBrains Mono), the logo marks in `ui/assets/`, and the Lucide icon set are all adopted as **placeholders** — superseded by future `design-system-v2`, `brand-v1`, and bespoke-icon changes respectively.

## License

MIT OR Apache-2.0
