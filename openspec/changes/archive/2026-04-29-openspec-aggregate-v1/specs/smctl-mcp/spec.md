# smctl-mcp Specification (delta)

## MODIFIED Requirements

### Requirement: Tool surface

`smctl-mcp` SHALL expose every smctl-managed action as an MCP tool. The tool catalog MUST include workspace (init / add / remove / sync), worktree (add / list / remove), flow (init / feature / release / hotfix), spec (new / validate / archive / list), build, and verify (policy / model / proof / protocol / discover). The `smctl_spec_list` / `_validate` / `_archive` tools MUST aggregate across every registered repo using the same resolution rules `smctl spec` applies on the CLI.

#### Scenario: Tool catalog enumeration

- **WHEN** an MCP client requests the tool list
- **THEN** the server MUST return at least the tools enumerated above
- **AND** every tool's input schema MUST round-trip through `serde_json` without loss

#### Scenario: Verify policy exposed as MCP tool

- **WHEN** an MCP client invokes the `verify_policy` tool
- **THEN** the server MUST run `smctl verify policy --json` against the active workspace
- **AND** MUST return the JSON envelope verbatim as the tool result

#### Scenario: Spec list aggregates across repos

- **WHEN** an MCP client invokes `smctl_spec_list` against a workspace registering two repos
- **THEN** the response MUST include every active spec from every repo
- **AND** each entry MUST carry a `repo` field naming the owning repository

#### Scenario: Spec validate resolves names

- **WHEN** an MCP client invokes `smctl_spec_validate` with `name="foo-v1"` and the name is unambiguous across repos
- **THEN** the server MUST validate that spec
- **AND** MUST return a result envelope that includes the resolved repo name
