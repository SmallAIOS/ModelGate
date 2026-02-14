# ModelGate

**smctl** (SmallAIOS Control) is a unified CLI for managing multi-repo workspaces, git flow branching, spec-driven development, and dependency-ordered builds.

## Installation

```bash
# Clone and build from source
git clone https://github.com/SmallAIOS/ModelGate.git
cd ModelGate
cargo install --path smctl
```

Requires Rust 2024 edition (1.85+).

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
```

## Architecture

5-crate Cargo workspace:

- **smctl** — CLI binary (clap derive, subcommand dispatch)
- **smctl-workspace** — workspace manifest, repo status, worktree management
- **smctl-flow** — git flow branching (feature, release, hotfix lifecycle)
- **smctl-spec** — OpenSpec workflow (scaffold, validate, archive)
- **smctl-build** — dependency-ordered build orchestration

## License

MIT OR Apache-2.0
