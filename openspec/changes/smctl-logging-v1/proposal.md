# smctl-logging-v1 — Proposal

## Why

`design-system-v1` declared that all log output across SmallAIOS, ModelGate, and `smctl` conforms to **RFC 5424** (The Syslog Protocol). That declaration is a forward-looking contract — no crate currently emits it. This change is the first implementation, and the reference against which future log-emitting surfaces (`smctl-serve --mcp`, `smctl-gate`, eventual web dashboard, kernel-side telemetry bridges) will be measured.

Why this matters now rather than "when we need it":

- **Safety-critical auditability.** Aerospace (DO-178C DAL A) and automotive (ISO 26262) consumers expect SIEM-ingestible logs. Shipping any production surface without standards-conformant logs invites a migration later under time pressure.
- **MSGID stability is a contract.** MSGIDs must be stable identifiers. Deciding the namespace and initial catalog now prevents per-crate improvisation that later becomes a breaking change to correct.
- **`smctl-mcp` and `smctl-serve` are next.** A long-running server surface without structured logs is a debuggability hole. Landing the subsystem before those changes means their events land into an already-declared catalog.
- **Reference implementation forces decisions.** The spec section in `design-system-v1` names principles; it does not pick a Rust crate, a severity mapping, or a MSGID format. One concrete implementation collapses those decisions into code.

## What Changes

1. **New crate: `smctl-log`.** Library crate in the workspace. Owns the tracing subscriber initialization, the RFC 5424 formatter, the MSGID catalog, and severity mapping.
2. **Adopt `tracing` + `tracing-subscriber`** as the facade across the ecosystem. Internal APIs use the short `tracing` macros; the subscriber turns them into RFC 5424 on the wire.
3. **Pick a syslog emitter strategy.** Candidates: `tracing-syslog` (wrapper), `syslog` crate (transport only), or a custom `tracing-subscriber::Layer`. Decision recorded in `design.md`.
4. **Define the initial MSGID catalog.** Namespace `SMCTL-NNNN` (4-digit zero-padded). First batch covers workspace init, spec lifecycle, flow feature lifecycle, and build lifecycle. Catalog is part of the spec.
5. **Severity mapping.** `tracing` levels (`ERROR` / `WARN` / `INFO` / `DEBUG` / `TRACE`) map to RFC 5424 numeric codes. The mapping is declared in-spec, not per-caller.
6. **New global CLI flags.** `--log-syslog`, `--log-file <path>`, `--log-level <level>`. Env vars: `SMCTL_LOG_SYSLOG`, `SMCTL_LOG_FILE`, `SMCTL_LOG_LEVEL`.
7. **`[logging]` section in `workspace.toml`.** Same keys as the CLI flags, with CLI / env taking precedence.
8. **Instrument initial events.** Emit events at high-signal points: `smctl workspace init`, `smctl spec new / archive`, `smctl flow feature start / finish`, `smctl build` start / end. Not a full retrofit — retrofitting every existing `println!` is explicitly out of scope.

## Capabilities

### New Capabilities

- `smctl-log` — Rust library: tracing subscriber, RFC 5424 formatter, MSGID catalog, severity mapping, transport selection (stderr, file, local syslog).

### Modified Capabilities

- `smctl-cli` — Gains three logging flags and initializes the subscriber at startup. Existing `--verbose` / `--quiet` flags are cross-wired to `--log-level`.
- `smctl-workspace` — Gains a `[logging]` section in the manifest schema (optional, defaults sane).
- `smctl-workspace`, `smctl-spec`, `smctl-flow`, `smctl-build` — Emit `tracing` events at the instrumented sites. No API breakage.

## Impact

### Repository Home

`smctl-log/` as a new top-level Cargo workspace member.

### New Files

```
ModelGate/
├── Cargo.toml                              # Workspace — add smctl-log member
├── smctl-log/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                          # init(), re-exports
│       ├── formatter.rs                    # RFC 5424 formatter Layer
│       ├── msgid.rs                        # MSGID catalog (enum or const table)
│       └── severity.rs                     # tracing level ↔ RFC 5424 severity
├── smctl/
│   ├── Cargo.toml                          # Add smctl-log, tracing deps
│   └── src/main.rs                         # Initialize subscriber; new CLI flags
├── smctl-workspace/src/lib.rs              # [logging] section in manifest
└── openspec/changes/smctl-logging-v1/
    ├── proposal.md
    ├── design.md
    ├── tasks.md
    └── specs/
        └── logging.md                      # MSGID catalog + format contract
```

### Dependencies (new)

- `tracing` — logging facade
- `tracing-subscriber` — subscriber composition
- `tracing-core` — trait surface for the custom Layer
- `syslog` crate — **only** if we choose local-syslog Unix socket transport for v1; otherwise deferred

No change to `rmcp`, `cedar-policy`, `axum`, or other `smctl-tool-v1` dependencies.

### Out of Scope (deferred)

- **TCP / UDP / TLS transports** (RFC 5425, 5426, 6587). Local stderr, file, and Unix-socket syslog only in v1.
- **OpenTelemetry bridge.** OTel is a richer model worth considering later; v1 keeps the surface small.
- **Retrofitting every `println!` in the codebase.** UX output stays as-is. Only the high-signal instrumented sites emit `tracing` events.
- **Kernel-side syslog (SmallAIOS).** Kernel telemetry is a separate surface and a separate change.
- **Log rotation / retention.** Defer to the OS / logrotate / syslog daemon. `smctl-log` writes; it does not rotate.

## References

- `openspec/changes/design-system-v1/specs/design-system.md` — Logging and Telemetry section (the contract this change implements)
- [RFC 5424 — The Syslog Protocol](https://datatracker.ietf.org/doc/html/rfc5424)
- [`tracing` crate](https://docs.rs/tracing)
- [`tracing-subscriber`](https://docs.rs/tracing-subscriber)
- [`syslog` crate](https://docs.rs/syslog)
- [OpenTelemetry Logs Data Model](https://opentelemetry.io/docs/specs/otel/logs/data-model/) — deferred alternative
