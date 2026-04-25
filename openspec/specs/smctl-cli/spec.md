# smctl-cli Specification

## Purpose

`smctl` is the unified CLI for managing the SmallAIOS multi-repo workspace. It binds workspace configuration, git flow branching, OpenSpec workflow, and dependency-ordered builds into a single tool. This capability defines the top-level command surface and the global flags every subcommand inherits.

## Requirements

### Requirement: Workspace flag resolution

The `smctl` binary SHALL accept a global `--workspace <path>` flag (env: `SMCTL_WORKSPACE`) that points at a directory containing `.smctl/workspace.toml`.

#### Scenario: Explicit workspace flag

- **WHEN** the operator runs `smctl --workspace /path/to/ws workspace status`
- **THEN** the command MUST resolve `/path/to/ws/.smctl/workspace.toml` as the manifest
- **AND** MUST NOT walk parent directories looking for an alternative manifest

#### Scenario: Workspace flag absent

- **WHEN** the operator runs `smctl workspace status` without `--workspace`
- **THEN** the command MUST walk upward from the current directory looking for `.smctl/workspace.toml`
- **AND** MUST fail with a remediation clause if no manifest is found

### Requirement: Output format selection

The `smctl` binary SHALL accept a global `--json` flag that switches every subcommand's stdout to a machine-readable JSON document.

#### Scenario: JSON output for status

- **WHEN** the operator runs `smctl workspace status --json`
- **THEN** stdout MUST contain a single JSON document parseable by `serde_json`
- **AND** stderr MUST contain only RFC 5424 log lines

#### Scenario: TTY-aware JSON fallback

- **WHEN** the operator runs a subcommand whose stdout is not a TTY
- **AND** that subcommand declares JSON-on-non-TTY behaviour (per safety-quality-v1 Decision 9)
- **THEN** the subcommand SHALL emit JSON regardless of the `--json` flag

### Requirement: Dry-run preview

The `smctl` binary SHALL accept a global `--dry-run` flag. When set, mutating subcommands MUST describe what they would do and exit with code 10 (`exit_code::DRY_RUN`) without performing any side effects.

#### Scenario: Build dry-run

- **WHEN** the operator runs `smctl --dry-run build SmallAIOS`
- **THEN** stdout MUST print "would build in order: SmallAIOS"
- **AND** the process MUST exit with code 10

### Requirement: Subcommand surface

The `smctl` binary SHALL expose subcommands `workspace`, `worktree`, `flow`, `spec`, `build`, `quality`, `gate`, `serve`, `config`, and `completions`, plus convenience aliases `feat`, `done`, `ss`, `sb`.

#### Scenario: Help enumerates the subcommand set

- **WHEN** the operator runs `smctl --help`
- **THEN** stdout MUST list every subcommand declared above
- **AND** the listing MUST match the `Subcommand` derive declaration in `smctl/src/main.rs`

### Requirement: Three-part error remediation

Every error message produced by an `smctl` subcommand SHALL contain three parts: what happened, what it means, what to do next (an executable command).

#### Scenario: Workspace not initialised

- **WHEN** the operator runs `smctl workspace status` outside a workspace
- **THEN** the error message MUST identify the missing `.smctl/workspace.toml`
- **AND** MUST suggest `smctl workspace init` as the next action
