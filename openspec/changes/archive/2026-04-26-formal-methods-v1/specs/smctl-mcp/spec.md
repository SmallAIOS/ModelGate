# smctl-mcp Specification (delta)

## MODIFIED Requirements

### Requirement: Tool surface

`smctl-mcp` SHALL expose every smctl-managed action as an MCP tool. The tool catalog MUST include workspace (init / add / remove / sync), worktree (add / list / remove), flow (init / feature / release / hotfix), spec (new / validate / archive / list), build, and verify (policy / model / proof / protocol / discover).

#### Scenario: Tool catalog enumeration

- **WHEN** an MCP client requests the tool list
- **THEN** the server MUST return at least the tools enumerated above
- **AND** every tool's input schema MUST round-trip through `serde_json` without loss

#### Scenario: Verify policy exposed as MCP tool

- **WHEN** an MCP client invokes the `verify_policy` tool
- **THEN** the server MUST run `smctl verify policy --json` against the active workspace
- **AND** MUST return the JSON envelope verbatim as the tool result
