# smctl-gate Specification

## Purpose

`smctl-gate` is the ModelGate control-plane client. It backs `smctl gate <verb>` with a reqwest-based HTTP client that talks to a running ModelGate instance over its REST API, plus an SSE log stream.

## Requirements

### Requirement: Endpoint resolution precedence

`smctl-gate` SHALL resolve the ModelGate endpoint URL in the precedence order: `--url` CLI flag > `MODELGATE_URL` env var > `[gate] url` in `workspace.toml` > built-in default `http://localhost:8080`.

#### Scenario: CLI flag overrides workspace.toml

- **WHEN** the manifest declares `[gate] url = "http://gate:9000"` and the operator passes `--url http://other:9001`
- **THEN** the active URL MUST be `http://other:9001`

#### Scenario: Default localhost

- **WHEN** no flag, env var, or manifest entry is set
- **THEN** the active URL MUST be `http://localhost:8080`

### Requirement: Six gate verbs

`smctl gate` SHALL expose verbs `status`, `models {list,add,remove}`, `routes {list,set}`, `test`, `logs`, and `web`.

#### Scenario: Verb help lists models subverbs

- **WHEN** the operator runs `smctl gate models --help`
- **THEN** stdout MUST list `list`, `add`, `remove`

### Requirement: Streaming model upload

`smctl gate models add <path>` SHALL stream the file at `<path>` to the upstream rather than buffering it in memory. Progress MUST be reported to the operator on a TTY at no finer cadence than once per percent.

#### Scenario: Multi-gigabyte upload does not OOM

- **WHEN** the operator runs `smctl gate models add ./model.bin` against a 4 GiB file
- **THEN** the smctl process resident-set size MUST stay below 256 MiB throughout the upload

### Requirement: Targeted error remediation

Each `GateError` variant SHALL render with a remediation clause that names a concrete next action. ConnectionRefused suggests starting ModelGate; Timeout suggests raising `--timeout`; ModelNotFound suggests running `smctl gate models list`.

#### Scenario: ConnectionRefused remediation

- **WHEN** `smctl gate status` is run against an unreachable URL
- **THEN** stderr MUST contain a remediation line that references `--url` or `MODELGATE_URL`

### Requirement: SSE log stream with Ctrl+C shutdown

`smctl gate logs [--follow]` SHALL open an SSE connection to `/api/v1/logs` and render each `LogEntry` as one line. Pressing Ctrl+C MUST close the stream cleanly and exit 0.

#### Scenario: Ctrl+C exits cleanly

- **WHEN** the operator runs `smctl gate logs --follow` and presses Ctrl+C
- **THEN** the process MUST exit 0
- **AND** stderr MUST contain a "log stream closed" notice on a TTY

### Requirement: HTTP error mapping

`smctl-gate` SHALL map HTTP responses to typed `GateError` variants: 404 from a model endpoint becomes `ModelNotFound`; connection refusal becomes `ConnectionRefused`; timeout becomes `Timeout`; other non-2xx responses become `HttpError { status, body }`.

#### Scenario: 404 on remove maps to ModelNotFound

- **WHEN** the upstream returns 404 for `DELETE /api/v1/models/ghost`
- **THEN** the client MUST raise `GateError::ModelNotFound { name: "ghost" }`
