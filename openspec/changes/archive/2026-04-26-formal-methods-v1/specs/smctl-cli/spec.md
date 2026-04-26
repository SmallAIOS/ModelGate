# smctl-cli Specification (delta)

## MODIFIED Requirements

### Requirement: Subcommand surface

The `smctl` binary SHALL expose subcommands `workspace`, `worktree`, `flow`, `spec`, `build`, `quality`, `gate`, `verify`, `serve`, `config`, and `completions`, plus convenience aliases `feat`, `done`, `ss`, `sb`.

#### Scenario: Help enumerates the subcommand set

- **WHEN** the operator runs `smctl --help`
- **THEN** stdout MUST list every subcommand declared above
- **AND** the listing MUST match the `Subcommand` derive declaration in `smctl/src/main.rs`

#### Scenario: Verify is reachable from the top level

- **WHEN** the operator runs `smctl verify --help`
- **THEN** the binary MUST resolve `verify` as a top-level subcommand
- **AND** MUST list its own subcommands (`policy`, `model`, `proof`, `protocol`, `discover`)
