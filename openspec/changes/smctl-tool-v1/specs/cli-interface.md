# smctl CLI Interface Specification

## Overview

`smctl` is a single binary with hierarchical subcommands. Every subcommand follows a consistent pattern: `smctl <domain> <action> [args] [flags]`.

## Global Flags

| Flag | Short | Description |
|---|---|---|
| `--workspace <path>` | `-w` | Override workspace root (default: auto-detect from cwd) |
| `--verbose` | `-v` | Increase output verbosity (repeatable: -vv, -vvv) |
| `--quiet` | `-q` | Suppress non-error output |
| `--dry-run` | | Show what would be done without executing |
| `--json` | | Output in JSON format (for scripting and MCP) |
| `--no-color` | | Disable colored output |
| `--config <path>` | `-c` | Override config file path |
| `--help` | `-h` | Show help for any command |
| `--version` | `-V` | Show version |

## Command Tree

```
smctl
├── workspace
│   ├── init [--name <name>]            # Initialize a new workspace
│   ├── add <url> [--path <path>]       # Add a repo to the workspace
│   ├── remove <repo>                   # Remove a repo from workspace
│   ├── status                          # Show status of all repos
│   └── sync                            # Fetch/pull all repos
│
├── worktree
│   ├── add <name> [--repos <r1,r2>]    # Create linked worktrees
│   ├── list                            # List active worktrees
│   ├── remove <name> [--force]         # Remove a worktree set
│   └── cd <name>                       # Print path to worktree (for shell eval)
│
├── flow
│   ├── init                            # Initialize git flow in all repos
│   ├── feature
│   │   ├── start <name> [--worktree]   # Create feature branch(es)
│   │   ├── finish <name>               # Merge feature → develop
│   │   └── list                        # List active features
│   ├── release
│   │   ├── start <version>             # Create release branch
│   │   ├── finish <version>            # Merge → main + develop, tag
│   │   └── list                        # List active releases
│   └── hotfix
│       ├── start <name>                # Create hotfix from main
│       ├── finish <name>               # Merge → main + develop, tag
│       └── list                        # List active hotfixes
│
├── spec
│   ├── new <name>                      # Create OpenSpec feature folder
│   ├── ff [<name>]                     # Fast-forward: generate docs
│   ├── apply [<name>]                  # Execute tasks
│   ├── archive [<name>]               # Archive completed spec
│   ├── validate [<name>]              # Check spec completeness
│   ├── status [<name>]                # Show spec progress
│   └── list                            # List all specs (active + archived)
│
├── build [<repo>]
│   ├── (default)                       # Build in dependency order
│   ├── --parallel                      # Build independent repos concurrently
│   ├── --test                          # Run tests after build
│   └── --clean                         # Clean before building
│
├── gate
│   ├── status                          # ModelGate health
│   ├── models
│   │   ├── list                        # List registered models
│   │   ├── add <path>                  # Register a model
│   │   └── remove <name>              # Unregister a model
│   ├── routes
│   │   ├── list                        # Show routing table
│   │   └── set <model> <endpoint>      # Configure route
│   ├── test <model> --input <file>     # Test inference
│   ├── logs [--follow]                 # Stream logs
│   ├── policy
│   │   ├── show                        # Display active SecurityPolicy
│   │   ├── load <blob>                 # Load signed policy blob (ML-DSA-65)
│   │   ├── diff <old> <new>            # Compare two policies
│   │   ├── verify                      # Run TLA+ model checker on policy
│   │   └── check <model>              # Run 5-layer verification on model
│   └── boundaries
│       ├── list                        # Show trust boundaries + SecurityLabels
│       └── check                      # Verify all crossings have formal proofs
│
├── serve
│   ├── --mcp                           # Start MCP server
│   ├── --stdio                         # Use stdio transport (default)
│   ├── --sse [--port <port>]           # Use SSE transport
│   └── --http [--port <port>]          # Use streamable HTTP transport
│
├── config
│   ├── show                            # Print effective configuration
│   ├── set <key> <value>               # Set a config value
│   ├── get <key>                       # Get a config value
│   └── edit                            # Open config in editor
│
└── (aliases)
    ├── feat <name>                     # → flow feature start + worktree add
    ├── done <name>                     # → worktree remove + flow feature finish
    ├── ss <name>                       # → spec new
    └── sb                              # → build

```

## Exit Codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | General error |
| 2 | Usage error (bad arguments) |
| 3 | Git operation failed |
| 4 | Workspace not found or invalid |
| 5 | Spec validation failed |
| 6 | Build failed |
| 7 | Network error (gate operations) |
| 10 | Dry-run completed (no changes made) |

## Output Formats

**Default (human-readable):**
```
$ smctl workspace status
  SmallAIOS   main      ✓ clean     3 commits ahead
  ModelGate   develop   ✗ dirty     2 files modified
```

**JSON (--json flag):**
```json
{
  "repos": [
    {
      "name": "SmallAIOS",
      "branch": "main",
      "clean": true,
      "ahead": 3,
      "behind": 0
    }
  ]
}
```

The JSON format is the same structure returned by MCP tool calls.

## Shell Completions

`smctl` generates shell completions via clap:

```bash
# Bash
smctl completions bash > ~/.local/share/bash-completion/completions/smctl

# Zsh
smctl completions zsh > ~/.zfunc/_smctl

# Fish
smctl completions fish > ~/.config/fish/completions/smctl.fish
```

## Environment Variables

| Variable | Description |
|---|---|
| `SMCTL_WORKSPACE` | Override workspace root path |
| `SMCTL_CONFIG` | Override user config path |
| `SMCTL_LOG` | Log level (trace, debug, info, warn, error) |
| `SMCTL_NO_COLOR` | Disable colored output (any value) |
| `SMCTL_EDITOR` | Editor for `smctl config edit` (falls back to `$EDITOR`) |
