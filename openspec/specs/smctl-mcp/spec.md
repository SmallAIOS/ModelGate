# smctl-mcp Specification

## Purpose

`smctl-mcp` is the Model Context Protocol server that exposes `smctl` tools and resources to AI coding assistants. It runs in the same process as `smctl serve --mcp` and supports both the stdio and SSE transports.
## Requirements
### Requirement: Stdio transport for MCP

`smctl-mcp` SHALL accept a stdio transport in which the MCP protocol owns stdout and the smctl-log subscriber routes every event to stderr.

#### Scenario: Server binds to stdio

- **WHEN** the operator runs `smctl serve --mcp --stdio`
- **THEN** the server MUST emit `SMCTL-0200` to stderr describing the stdio binding
- **AND** stdout MUST contain only MCP protocol bytes

### Requirement: SSE transport for MCP

`smctl-mcp` SHALL accept an SSE / streamable-HTTP transport bound to a TCP port (default 9377). The protocol bytes MUST flow over the TCP socket; stdout MUST be free for unrelated output.

#### Scenario: Server binds to SSE on the default port

- **WHEN** the operator runs `smctl serve --mcp --sse`
- **THEN** the server MUST listen on `127.0.0.1:9377`
- **AND** MUST emit `SMCTL-0200` describing the SSE bind address

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

### Requirement: Resource surface

`smctl-mcp` SHALL expose smctl-managed read-only data as MCP resources, including workspace config, workspace status, flow branch lists, and spec list (plus per-spec resources templated by name).

#### Scenario: Workspace status resource

- **WHEN** an MCP client reads the `workspace_status` resource
- **THEN** the server MUST emit `SMCTL-0207` to log the resource read
- **AND** MUST return the same JSON document `smctl workspace status --json` produces

### Requirement: Subscriber ownership

`smctl-mcp` MUST NOT install its own tracing subscriber. The `smctl` binary SHALL initialise `smctl_log::init` once before invoking `start_server`.

#### Scenario: Subscriber is initialised by the binary

- **WHEN** the server starts via `smctl serve --mcp --stdio`
- **THEN** `smctl_log::init` MUST have been called exactly once before the first MCP byte is written
