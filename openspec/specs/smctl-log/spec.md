# smctl-log Specification

## Purpose

`smctl-log` is the RFC 5424 tracing subscriber and MSGID catalog used by every other crate in the workspace. It owns the wire format, the transport selection, and the message-id allocation. Callers emit events with the `tracing` macros; the subscriber renders them.
## Requirements
### Requirement: RFC 5424 wire format

`smctl-log` SHALL emit log records that conform to [RFC 5424](https://datatracker.ietf.org/doc/html/rfc5424). Every record MUST include the priority, timestamp, hostname, app-name, procid, msgid, structured-data, and message fields in the canonical order.

#### Scenario: Stderr line is RFC 5424 conformant

- **WHEN** the subscriber emits a record at INFO severity for `MsgId::WorkspaceInitialized`
- **THEN** the line MUST start with the priority `<134>` (LOCAL0 / Informational)
- **AND** MUST contain the MSGID `SMCTL-0001`
- **AND** MUST place structured-data inside `[SMCTL@32473 ...]` brackets

### Requirement: MSGID range allocation

The MSGID catalog SHALL allocate ranges per producer crate. The allocation table MUST be:

- `SMCTL-0001..0099` — smctl core (workspace, spec, flow, build)
- `SMCTL-0200..0299` — smctl-mcp
- `SMCTL-0300..0399` — modelgate-web
- `SMCTL-0400..0499` — smctl-quality
- `SMCTL-0500..0599` — smctl-verify

#### Scenario: Web MSGID is in range

- **WHEN** the operator inspects `MsgId::WebServerStarted.code()`
- **THEN** the returned value MUST be `301`
- **AND** the value MUST satisfy `(300..=399).contains(&code)`

#### Scenario: Quality MSGID is in range

- **WHEN** the operator inspects `MsgId::QualityCheckStarted.code()`
- **THEN** the returned value MUST be `400`
- **AND** the value MUST satisfy `(400..=499).contains(&code)`

#### Scenario: Verify MSGID is in range

- **WHEN** the operator inspects `MsgId::VerifyStarted.code()`
- **THEN** the returned value MUST be `501`
- **AND** the value MUST satisfy `(500..=599).contains(&code)`

### Requirement: Multi-transport delivery

`smctl-log` SHALL support three transports that may run together: stderr (default), file (append-only), and the local Unix syslog socket.

#### Scenario: File transport opens append-mode

- **WHEN** the operator runs a command with `--log-file /var/log/smctl.log`
- **THEN** the subscriber MUST open the file in append mode
- **AND** MUST create parent directories that do not yet exist

#### Scenario: Syslog transport falls back to stderr

- **WHEN** the operator runs with `--log-syslog` on a host where the local Unix socket cannot be opened
- **THEN** the subscriber MUST emit one `SMCTL-0099` warning describing the open failure
- **AND** MUST continue emitting subsequent records to stderr

### Requirement: Configuration precedence

The CLI SHALL resolve logging configuration in the precedence order: CLI flags > env vars > `[logging]` in `workspace.toml` > built-in defaults.

#### Scenario: CLI flag overrides workspace.toml

- **WHEN** the manifest declares `[logging] level = "warn"` and the operator passes `--log-level debug`
- **THEN** the active level MUST be `Debug`

### Requirement: Default severity per MSGID

Every MSGID SHALL declare a default severity that callers MAY override but MUST NOT silently violate.

#### Scenario: Build failure defaults to Error

- **WHEN** the operator inspects `MsgId::BuildFailed.default_severity()`
- **THEN** the result MUST be `Severity::Error`

#### Scenario: Web upstream warnings default to Warning

- **WHEN** the operator inspects `MsgId::WebUpstreamUnreachable.default_severity()`
- **THEN** the result MUST be `Severity::Warning`
