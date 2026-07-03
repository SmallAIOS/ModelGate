# smctl-cli Specification (delta)

## MODIFIED Requirements

### Requirement: Subcommand surface

The `smctl` binary SHALL expose subcommands `workspace`, `worktree`, `flow`, `spec`, `build`, `quality`, `gate`, `verify`, `serve`, `config`, and `completions`, plus convenience aliases `feat`, `done`, `ss`, `sb`. Within `spec`, every single-spec verb (`validate`, `apply`, `archive`, `status`, `ff`) MUST accept either a bare spec name or a `repo:name` qualified form, and SHOULD accept a `--repo <name>` flag for explicit repo selection. `spec list` MUST aggregate across every registered repo.

#### Scenario: Help enumerates the subcommand set

- **WHEN** the operator runs `smctl --help`
- **THEN** stdout MUST list every subcommand declared above
- **AND** the listing MUST match the `Subcommand` derive declaration in `smctl/src/main.rs`

#### Scenario: Verify is reachable from the top level

- **WHEN** the operator runs `smctl verify --help`
- **THEN** the binary MUST resolve `verify` as a top-level subcommand
- **AND** MUST list its own subcommands (`policy`, `model`, `proof`, `protocol`, `discover`)

#### Scenario: spec list aggregates across repos

- **WHEN** the operator runs `smctl spec list` against a workspace registering two repos with active specs in each
- **THEN** stdout MUST list every active spec from every repo
- **AND** the output MUST identify the owning repo for each row

#### Scenario: spec validate resolves bare names unambiguously

- **WHEN** exactly one registered repo declares spec `foo-v1`
- **AND** the operator runs `smctl spec validate foo-v1`
- **THEN** the validator MUST run against that repo's spec without requiring `repo:foo-v1`

#### Scenario: spec validate refuses ambiguous bare names

- **WHEN** two registered repos each declare spec `foo-v1`
- **AND** the operator runs `smctl spec validate foo-v1`
- **THEN** the command MUST fail with a remediation clause that lists every match in the qualified `repo:name` form

### Requirement: Three-part error remediation

Every error message produced by an `smctl` subcommand SHALL contain three parts: what happened, what it means, what to do next (an executable command). When `spec list` and friends touch the openspec tree, the `next` part MUST reference `smctl spec list` so the operator can enumerate before retrying.

#### Scenario: Workspace not initialised

- **WHEN** the operator runs `smctl workspace status` outside a workspace
- **THEN** the error message MUST identify the missing `.smctl/workspace.toml`
- **AND** MUST suggest `smctl workspace init` as the next action

#### Scenario: Spec name not found anywhere

- **WHEN** the operator runs `smctl spec validate nonexistent` and no registered repo declares that spec
- **THEN** the error MUST cite the missing name
- **AND** MUST suggest `smctl spec list` as the next action
