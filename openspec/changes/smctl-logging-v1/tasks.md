# smctl-logging-v1 — Tasks

## Spec Documents

- [x] Author `proposal.md`
- [x] Author `design.md`
- [x] Author `tasks.md` (this file)
- [x] Author `specs/logging.md` — MSGID catalog, format contract, severity table

## Crate Bootstrap

- [ ] Create `smctl-log/` directory with `Cargo.toml` and `src/`
- [ ] Add `smctl-log` to the root `Cargo.toml` workspace members
- [ ] Add `tracing`, `tracing-subscriber`, `syslog`, `chrono` (or `time`) dependencies
- [ ] Add `smctl-log` as a dependency of `smctl` (the binary crate)

## Formatter

- [ ] Implement `formatter::Rfc5424Layer` — a `tracing_subscriber::Layer` that serializes events to RFC 5424
- [ ] Implement `severity::tracing_level_to_syslog(Level) -> u8`
- [ ] Implement `msgid::MsgId` enum with `Display` producing the canonical `SMCTL-NNNN` string
- [ ] Implement STRUCTURED-DATA serialization — escape `]` `"` `\` per RFC 5424 § 6.3.3
- [ ] Implement RFC 3339 timestamp emission using `chrono` or `time`

## Transports

- [ ] `transport::Stderr` — writes one RFC 5424 line per event to stderr
- [ ] `transport::File` — append-only file writer; creates parent dir if missing
- [ ] `transport::Syslog` — Unix socket via `syslog` crate; fall back to stderr on open failure with a one-time WARN

## Subscriber Init

- [ ] `smctl_log::init(config: &LoggingConfig)` — composes Layers per config
- [ ] `LoggingConfig` struct — `transports: Vec<Transport>`, `level: Level`, `file: Option<PathBuf>`, `facility: SyslogFacility`
- [ ] Config precedence resolver: CLI flags > env > workspace.toml > defaults
- [ ] Ensure `init` is idempotent (calling twice is a no-op, not a panic)

## CLI Integration

- [ ] Add `--log-syslog` / `--log-file` / `--log-level` global flags to `smctl/src/main.rs`
- [ ] Wire `SMCTL_LOG_SYSLOG` / `SMCTL_LOG_FILE` / `SMCTL_LOG_LEVEL` env vars
- [ ] Cross-wire existing `--verbose` / `--quiet` to `--log-level` (verbose bumps level, quiet suppresses)
- [ ] Call `smctl_log::init` at the top of `main()`, after clap parsing

## Workspace Manifest Schema

- [ ] Add `[logging]` section to `smctl-workspace::WorkspaceManifest`
- [ ] Parser accepts the section as optional; defaults applied when absent
- [ ] Round-trip test: write → read preserves values

## Instrumentation (initial events)

- [ ] `SMCTL-0001` — workspace initialized (in `smctl-workspace::init`)
- [ ] `SMCTL-0002` — spec created (in `smctl-spec::new`)
- [ ] `SMCTL-0003` — spec archived (in `smctl-spec::archive`)
- [ ] `SMCTL-0004` — feature branch started (in `smctl-flow::feature::start`)
- [ ] `SMCTL-0005` — feature branch finished (in `smctl-flow::feature::finish`)
- [ ] `SMCTL-0006` — build started (in `smctl-build::build`)
- [ ] `SMCTL-0007` — build completed (in `smctl-build::build`)
- [ ] `SMCTL-0008` — build failed (in `smctl-build::build`)
- [ ] `SMCTL-0099` — uncategorized error (fallback; emit from a top-level catch in `smctl/src/main.rs`)

## Tests

- [ ] Unit test: formatter emits RFC 5424-shaped line for a sample event
- [ ] Unit test: STRUCTURED-DATA escaping for `]`, `"`, `\`
- [ ] Unit test: severity mapping covers all five `tracing` levels
- [ ] Unit test: MSGID `Display` produces canonical zero-padded form
- [ ] Integration test: `smctl workspace init --log-file tmp.log` writes a line containing `SMCTL-0001`
- [ ] Full workspace `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`

## Docs

- [ ] Update `README.md` with a short Logging section
- [ ] Update `CLAUDE.md` Design system section with a one-line logging pointer
- [ ] Update `.claude/skills/smallaios-design/SKILL.md` with a pointer to the MSGID catalog

## Archive

- [ ] Run `smctl spec archive smctl-logging-v1` when all non-deferred tasks are complete and the change has merged to `develop`
