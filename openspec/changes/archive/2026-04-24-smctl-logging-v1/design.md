# smctl-logging-v1 — Design Document

## Context

`design-system-v1` declared RFC 5424 conformance for all log output. This change picks the Rust crates, the severity mapping, the MSGID namespace, and the initial instrumentation surface. The goal is a reference implementation that downstream log-emitting crates (`smctl-mcp`, `smctl-gate`, `smctl-serve`, kernel telemetry bridges) inherit without having to re-litigate the choices.

## Goals / Non-Goals

### Goals

1. One place in the repo produces RFC 5424 log output: `smctl-log`.
2. Every crate that wants to log uses the `tracing` macros; no crate reimplements formatting.
3. MSGIDs are stable, short, and ASCII. Changing one is a breaking change to consumers.
4. Severity mapping is declared once and followed uniformly.
5. Local-only transports (stderr, file, Unix-socket syslog) cover the v1 surface.
6. Tests prove the wire format matches RFC 5424 for a representative event.

### Non-Goals

1. Not implementing RFC 5425 / 5426 / 6587 transports — deferred.
2. Not migrating every `println!` — CLI UX output stays as-is.
3. Not bridging to OpenTelemetry — deferred alternative.
4. Not building a log viewer, search UI, or query surface.
5. Not defining log rotation / retention — OS concern.
6. Not kernel-side telemetry — separate change.

## Decisions

### Decision 1: `tracing` + `tracing-subscriber` as the facade

**Choice:** Adopt the `tracing` ecosystem as the logging facade across all smctl crates.

**Rationale:** `tracing` is the de facto standard for Rust, handles spans and structured fields natively, and composes with `tracing-subscriber` layers so the RFC 5424 formatter can be written once and plugged in. The `log` crate is a lower-ceiling alternative that lacks structured fields and span context. Structured fields are required by our STRUCTURED-DATA rule, so `log` alone would force us to re-invent the feature.

**Alternatives considered:**

- *`log` + `env_logger`* — insufficient; no structured fields, no spans, no layer composition.
- *`slog`* — capable but its ecosystem is smaller and less active than `tracing` in 2025–2026.
- *Raw `println!` to stderr* — what we have now; not a contender.

### Decision 2: Custom `tracing-subscriber::Layer` for RFC 5424 formatting

**Choice:** Implement a custom `Layer` in `smctl-log::formatter` that emits RFC 5424. Use the external `syslog` crate for transport when writing to a local syslog Unix socket, but not for formatting.

**Rationale:** RFC 5424 is not large — roughly 300 lines of Rust covers header construction, PRI calculation, timestamp formatting (RFC 3339 subset), STRUCTURED-DATA serialization, and MSG encoding. Owning the formatter means the output matches our declared contract byte-for-byte. Wrapper crates (`tracing-syslog`) introduce a thin layer of abstraction and maintenance risk for little benefit at this scope.

The `syslog` crate remains useful for the socket-transport code (opening the Unix socket, handling fallbacks across `/dev/log` / `/var/run/syslog`). We use it for transport, not for format.

**Alternatives considered:**

- *`tracing-syslog` crate* — wrapper over `tracing` + `syslog`. Abandonment risk and it does not give us bit-exact control of STRUCTURED-DATA shape.
- *`tracing-gelf` / `tracing-bunyan-formatter` / `tracing-logfmt`* — these are different formats, not syslog. Not candidates.
- *Custom Layer + custom socket code* — avoids the `syslog` crate dep entirely. Small benefit, small cost — but the `syslog` crate handles platform differences (Linux `/dev/log`, macOS `/var/run/syslog`, BSD variants) for free. Accept the dep.

### Decision 3: Severity mapping

**Choice:** Map `tracing` levels to RFC 5424 severities with a stable, declared table. No caller-side overrides.

| `tracing::Level` | RFC 5424 numeric | RFC 5424 name |
|---|---|---|
| `ERROR` | 3 | `Error` |
| `WARN` | 4 | `Warning` |
| `INFO` | 6 | `Informational` |
| `DEBUG` | 7 | `Debug` |
| `TRACE` | 7 | `Debug` |

`Emergency` (0), `Alert` (1), `Critical` (2), and `Notice` (5) are not reachable via `tracing` levels. They are reserved for direct use via a `smctl_log::emit!()` macro with an explicit severity argument, for cases where a caller genuinely needs `Critical` — e.g., a safety-critical invariant broken at runtime.

**Rationale:** `tracing` only defines five levels; RFC 5424 defines eight. The missing three (`Emergency`, `Alert`, `Critical`, `Notice`) are less common and don't map cleanly. Providing an escape hatch macro is cleaner than overloading `tracing::Level`.

**Alternatives considered:**

- *Collapse `TRACE` → `Debug` and `DEBUG` → `Informational`* — confuses the debug-verbosity axis; callers expect `DEBUG` ≠ `INFO`. Rejected.
- *Emit `Notice` for `INFO`* — Notice (5) is "normal but significant" in syslog tradition, which is arguably a better semantic fit than Informational (6). But this would surprise `tracing` users who expect `INFO` ↔ `Informational`. Rejected in favor of the literal mapping.

### Decision 4: MSGID namespace `SMCTL-NNNN`

**Choice:** Four-digit zero-padded numeric namespace with the `SMCTL-` prefix. Example: `SMCTL-0001`.

**Rationale:** Short, ASCII-clean, visually distinct in log streams, allows 9999 MSGIDs in v1 which is ample. The prefix keeps the namespace unambiguous when logs from multiple SmallAIOS components land in the same SIEM.

MSGIDs are **immutable once published**. Changing a MSGID's meaning is a breaking change for consumers. New events claim the next unused number.

**Alternatives considered:**

- *Symbolic MSGIDs* (`WORKSPACE_INIT`, `BUILD_FAILED`) — more readable but longer and harder to grep. Chose numeric for brevity; the catalog in `specs/logging.md` provides the symbolic mapping.
- *UUIDs* — too long, no sort order.
- *Five digits* — unnecessary; 9999 is plenty at alpha scope.

### Initial MSGID catalog

Declared in `specs/logging.md`. v1 covers:

- `SMCTL-0001` — workspace initialized
- `SMCTL-0002` — spec created
- `SMCTL-0003` — spec archived
- `SMCTL-0004` — feature branch started
- `SMCTL-0005` — feature branch finished
- `SMCTL-0006` — build started
- `SMCTL-0007` — build completed
- `SMCTL-0008` — build failed
- `SMCTL-0099` — uncategorized error (fallback)

Range allocation: `SMCTL-0001..0099` reserved for smctl itself. `SMCTL-0100..0199` reserved for `smctl-gate` (future), `SMCTL-0200..0299` for `smctl-mcp`, etc. This avoids namespace collisions as the ecosystem grows.

### Decision 5: Transports in scope for v1

**Choice:** Three transports, selected at subscriber init:

1. **stderr** (default for interactive use) — RFC 5424 format written to stderr.
2. **File** (`--log-file <path>`) — RFC 5424 format, one message per line, append-only.
3. **Local syslog** (`--log-syslog`) — Unix socket via the `syslog` crate. `/dev/log` on Linux, `/var/run/syslog` on macOS.

Multiple transports can be active simultaneously (e.g., stderr + file for local debugging while also shipping to syslog).

**Deferred:** RFC 5425 (TLS), RFC 5426 (UDP), RFC 6587 (TCP framing). Those land in `smctl-logging-v2` when a remote aggregator target is demanded.

**Rationale:** Local transports cover every immediate need (developer iteration, CI capture, local-machine audit). Remote transports add configuration surface (endpoint URLs, TLS cert management, retry semantics) that is not justified for v1.

### Decision 6: `[logging]` section in workspace.toml

Schema:

```toml
[logging]
# One or more of: "stderr", "file", "syslog"
transports = ["stderr"]

# Only read if "file" in transports
file = "/var/log/smctl.log"

# Only read if "syslog" in transports
facility = "local0"

# ERROR / WARN / INFO / DEBUG / TRACE (case-insensitive)
level = "INFO"
```

CLI flags and env vars override this section. Precedence (highest first):

1. CLI flags (`--log-level`, `--log-file`, `--log-syslog`)
2. Env vars (`SMCTL_LOG_LEVEL`, `SMCTL_LOG_FILE`, `SMCTL_LOG_SYSLOG`)
3. `workspace.toml` `[logging]` section
4. Defaults (stderr only, INFO level)

### Decision 7: Hostname, APP-NAME, PROCID

- **APP-NAME** = `"smctl"` (constant).
- **PROCID** = OS pid at subscriber init, rendered as decimal.
- **HOSTNAME** = result of `gethostname()`, falling back to the RFC 5424 nilvalue (`-`) if resolution fails.

No configuration knobs for these in v1. If someone needs to override APP-NAME (e.g. for `smctl-serve` running as a daemon), that is addressed in a successor change.

## Risks / Trade-offs

- **Platform variance in `/dev/log` resolution.** The `syslog` crate handles this, but not every UNIX has a local syslog socket available. Behavior: if the socket cannot be opened, `smctl-log::init()` logs a WARN to stderr and falls back to stderr-only. Does not panic.
- **Format evolution.** If a future change wants to add structured fields that RFC 5424 STRUCTURED-DATA syntax can't cleanly express, we'd be stuck with a compromise or a breaking format change. Accept — RFC 5424 SD is expressive enough for everything in scope.
- **Performance.** RFC 5424 formatting and STRUCTURED-DATA escaping is not free. For a CLI tool this is negligible; for `smctl-serve --mcp` running at high event rates, revisit.
- **MSGID catalog ossification.** MSGIDs added here cannot be repurposed. Accept — that is the point. But we should resist adding a MSGID until an event is genuinely worth logging; don't pre-populate the catalog for events that may never fire.
- **Windows.** `/dev/log` does not exist on Windows. v1 syslog transport is Unix-only. stderr and file transports work on Windows. Document this.

## Migration Plan

No migration. This is additive. Existing `println!` / `eprintln!` calls remain. Code that emits `tracing` events today (there is none) will transparently start going through the new formatter.

When subsequent changes (e.g., `smctl-mcp`, `smctl-gate`) need logs, they call `tracing::event!` / `tracing::error!` / `tracing::info!` with a canonical MSGID via structured field `msgid = "SMCTL-NNNN"`. The subscriber does the rest.

## Open Questions

1. **Should MSGIDs also appear as a code-level enum?** Pro: compile-time catalog, can't typo. Con: forces every emitter crate to depend on `smctl-log` just for the enum. Leaning: yes, publish `smctl_log::MsgId` as an enum with `Display` impl that produces the canonical string. Crates can still use a free-form string if they have a reason.
2. **Does the workspace.toml `[logging]` section belong in `smctl-workspace` or `smctl-log`?** The schema lives in `smctl-workspace` because workspace.toml is its domain, but the parsing helper could live in either. Leaning: schema in `smctl-workspace`, helper in `smctl-log`.
3. **Do we want a `smctl log tail` subcommand?** A convenience to stream the current log file with formatting. Not scoped for v1; file as a follow-up.
