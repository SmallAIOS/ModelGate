# Logging Specification

## Overview

This specification is the concrete RFC 5424 implementation contract for SmallAIOS, ModelGate, and `smctl`. It implements the `design-system-v1` Logging and Telemetry section. When the two conflict, `design-system-v1` is the abstract principle and this document is the implementation — resolve by making this document conform.

## Wire Format

All log events emitted by `smctl-log` follow [RFC 5424 § 6](https://datatracker.ietf.org/doc/html/rfc5424#section-6) exactly:

```
<PRI>VERSION SP TIMESTAMP SP HOSTNAME SP APP-NAME SP PROCID SP MSGID SP STRUCTURED-DATA SP MSG
```

Field rules:

- **PRI** — `<>`-bracketed integer = `facility * 8 + severity`. Facility defaults to `local0` (16); overridable by config. Severity comes from the mapping table below.
- **VERSION** — always `1`.
- **TIMESTAMP** — RFC 3339 with sub-second precision in microseconds, UTC with `Z` suffix. Example: `2026-04-19T18:23:14.123456Z`.
- **HOSTNAME** — result of `gethostname(2)`, or `-` (nilvalue) if resolution fails.
- **APP-NAME** — `smctl` for the CLI binary. Future daemon surfaces (`smctl-serve`) **MAY** override via explicit `LoggingConfig::app_name`.
- **PROCID** — OS pid as decimal.
- **MSGID** — from the canonical catalog in this document. Free-form strings are **forbidden**; the formatter **MUST** reject events with a non-catalog MSGID in debug builds and substitute `SMCTL-0099` in release builds, logging a WARN about the substitution.
- **STRUCTURED-DATA** — `[SMCTL@32473 key1="value1" key2="value2" …]` where `32473` is a placeholder Enterprise Number (see Open Questions). Multiple elements are permitted. Use `-` (nilvalue) when no structured data applies.
- **MSG** — Optional human-readable message. BOM prefix (`EF BB BF`) is **MUST NOT** be emitted; all messages are UTF-8 without BOM.

## Severity Mapping

`tracing::Level` to RFC 5424 severity:

| `tracing` | RFC 5424 numeric | RFC 5424 name |
|---|---|---|
| `ERROR` | 3 | `Error` |
| `WARN`  | 4 | `Warning` |
| `INFO`  | 6 | `Informational` |
| `DEBUG` | 7 | `Debug` |
| `TRACE` | 7 | `Debug` |

Severities 0 (`Emergency`), 1 (`Alert`), 2 (`Critical`), and 5 (`Notice`) are **not** produced by `tracing` macros. Call sites that need them **MUST** use `smctl_log::emit!(severity = Severity::Critical, msgid = MsgId::Whatever, …)`.

## Facility

Default facility: `local0` (16). Overridable via `[logging] facility = "…"` in `workspace.toml`. Accepted values map to RFC 5424 facility codes:

| Name | Numeric | Use |
|---|---|---|
| `daemon` | 3 | reserved for `smctl-serve` |
| `local0` | 16 | smctl CLI default |
| `local1` | 17 | reserved for smctl-gate |
| `local2` | 18 | reserved for smctl-mcp |
| `local3`–`local7` | 19–23 | available |

Other RFC 5424 facility names (`kern`, `user`, `mail`, `auth`, …) **MUST NOT** be used — they belong to the OS.

## MSGID Catalog — v1

MSGIDs are **immutable**. A MSGID's meaning **MUST NOT** change once published; the only permitted evolution is adding new MSGIDs. Ranges reserved for future crates:

- `SMCTL-0001`–`SMCTL-0099` — `smctl` (CLI + core)
- `SMCTL-0100`–`SMCTL-0199` — `smctl-gate` (future)
- `SMCTL-0200`–`SMCTL-0299` — `smctl-mcp` (future)
- `SMCTL-0300`–`SMCTL-0399` — `smctl-serve` (future)
- `SMCTL-0400`–`SMCTL-0999` — unallocated

### Events defined in this change

| MSGID | Severity | Symbolic name | STRUCTURED-DATA keys | Meaning |
|---|---|---|---|---|
| `SMCTL-0001` | `Informational` | `WorkspaceInitialized` | `path`, `name` | `smctl workspace init` succeeded |
| `SMCTL-0002` | `Informational` | `SpecCreated` | `name`, `path`, `branch` | `smctl spec new <name>` succeeded |
| `SMCTL-0003` | `Informational` | `SpecArchived` | `name` | `smctl spec archive <name>` succeeded |
| `SMCTL-0004` | `Informational` | `FeatureStarted` | `name`, `repos` | `smctl flow feature start` succeeded |
| `SMCTL-0005` | `Informational` | `FeatureFinished` | `name`, `merged_to` | `smctl flow feature finish` succeeded |
| `SMCTL-0006` | `Informational` | `BuildStarted` | `repo`, `parallel` | `smctl build` started |
| `SMCTL-0007` | `Informational` | `BuildCompleted` | `repo`, `duration_ms`, `passed_count`, `failed_count` | `smctl build` completed; `failed_count` is 0 |
| `SMCTL-0008` | `Error` | `BuildFailed` | `repo`, `duration_ms`, `passed_count`, `failed_count`, `first_failure` | `smctl build` completed with failures |
| `SMCTL-0099` | `Error` | `Uncategorized` | `error`, `backtrace` (if available) | Fallback for any `anyhow::Error` reaching `main()` |

STRUCTURED-DATA keys **MUST** be snake_case ASCII. Values are UTF-8 strings. Numeric values are emitted as their decimal string representation.

## Transports

v1 supports three transports. Multiple may be active simultaneously.

### stderr (default)

- Destination: `std::io::stderr()`
- Format: RFC 5424 line-oriented, `\n`-delimited
- Activation: default if no other transport specified

### File (`--log-file <path>`)

- Destination: the given path, opened in append mode (`O_APPEND` on Unix)
- Creates parent directory if absent, with mode `0o755`
- One RFC 5424 message per line
- No rotation — delegate to `logrotate` or platform equivalent
- On write failure: emit a WARN on stderr; subsequent events continue to try, do not panic

### Local syslog (`--log-syslog`)

- Destination: local syslog Unix socket via the `syslog` crate
- Platform paths: `/dev/log` (Linux), `/var/run/syslog` (macOS), others delegated to the crate
- On socket-open failure: emit a WARN on stderr explaining the fallback; **MUST** activate the stderr transport automatically so no events are lost
- Windows: unsupported in v1; specifying `--log-syslog` on Windows emits a WARN and falls back to stderr

## Configuration Precedence

Highest wins:

1. CLI flags: `--log-level`, `--log-file`, `--log-syslog`
2. Env vars: `SMCTL_LOG_LEVEL`, `SMCTL_LOG_FILE`, `SMCTL_LOG_SYSLOG`
3. `workspace.toml` `[logging]` section
4. Defaults: `transports = ["stderr"]`, `level = "INFO"`, `facility = "local0"`

## Level Filtering

The level applies to **all active transports** uniformly. Per-transport level filtering is out of scope for v1. `tracing::Level::INFO` by default; `-v` bumps to `DEBUG`, `-vv` to `TRACE`, `-q` bumps to `WARN`, `-qq` to `ERROR`.

## Color and Decoration

Log output **MUST NOT** contain ANSI color escapes, emoji, or forbidden Unicode pictographs (see `design-system-v1` § Emoji and ornament). Color and decoration belong to the interactive CLI UX layer (`println!` output), not the structured log stream. `tracing-subscriber::fmt` layer is **not** used.

## Idempotency

`smctl_log::init` is idempotent. Calling it twice:

- Does not double-register layers
- Does not panic
- Does not create a second file handle on the log file
- Does emit a DEBUG event noting the second call was a no-op

## Error Handling

Errors in the logging pipeline **MUST NOT** propagate to user code. A log-write failure is a warning, not a fault. The formatter catches its own errors and emits a synthesized `SMCTL-0099` event to stderr describing the failure, exactly once per minute (rate-limited to avoid log-loop storms).

## Conformance Testing

A conformance test suite **MUST** verify:

1. A sample event serializes to a byte-for-byte expected RFC 5424 line.
2. STRUCTURED-DATA escaping of `]`, `"`, `\` per RFC 5424 § 6.3.3.
3. All five `tracing` levels map to the declared severity.
4. MSGID `Display` produces the `SMCTL-NNNN` form with leading zeros.
5. Setting `--log-file` produces a file containing the expected MSGID.

## Future Work (not in v1)

- RFC 5425 TLS transport
- RFC 5426 UDP transport
- RFC 6587 TCP framing
- OpenTelemetry Logs Data Model bridge
- Per-transport level filtering
- Log rotation hooks
- `smctl log tail` subcommand
- Windows syslog support (Event Log bridge)
- Kernel-side telemetry bridge (SmallAIOS-side)

## Open Questions

1. **Enterprise Number for STRUCTURED-DATA.** The placeholder `32473` is IANA's example number; production use requires a real PEN. Follow-up: apply for one via IANA or use a vendor-neutral scheme. Not blocking v1 but must resolve before any external SIEM ingestion.
2. **Whether to require `msgid` as an enum or accept strings.** Leaning enum with an escape hatch (`tracing::event!(msgid = "SMCTL-0007", …)` is valid but the `MsgId` enum is recommended).
3. **Whether `smctl-log::init` returns a guard (like `tracing::subscriber::set_default`).** If so, the CLI needs to hold it in `main()`. If not, use `set_global_default` which is fire-and-forget. Leaning global default for a short-lived CLI, guard for `smctl-serve`.
