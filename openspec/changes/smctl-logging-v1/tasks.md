# smctl-logging-v1 — Tasks

## Spec Documents

- [x] Author `proposal.md`
- [x] Author `design.md`
- [x] Author `tasks.md` (this file)
- [x] Author `specs/logging.md` — MSGID catalog, format contract, severity table

## Crate Bootstrap

- [x] Create `smctl-log/` directory with `Cargo.toml` and `src/`
- [x] Add `smctl-log` to the root `Cargo.toml` workspace members
- [x] Add `tracing`, `tracing-subscriber`, `time`, `gethostname`, `thiserror` dependencies (syslog crate deferred; see below)
- [x] Add `smctl-log` as a dependency of `smctl` (the binary crate)

## Formatter

- [x] Implement `Rfc5424` — a `FormatEvent` impl for `tracing-subscriber::fmt::Layer` (chose the FormatEvent idiom over a free-standing Layer so it composes cleanly with MakeWriter)
- [x] Implement `severity::from_tracing_level(Level) -> Severity`
- [x] Implement `msgid::MsgId` enum with `Display` producing the canonical `SMCTL-NNNN` string
- [x] Implement STRUCTURED-DATA serialization — escape `]` `"` `\` per RFC 5424 § 6.3.3
- [x] Implement RFC 3339 timestamp emission using `time` crate

## Transports

- [x] stderr — default active when no `--log-file`; otherwise gated on `--verbose`
- [x] File (`--log-file <path>`) — append-only, creates parent dir if absent, `Mutex<File>` for thread safety
- [ ] Syslog Unix socket (`--log-syslog`) — **deferred** within this change. Follow-up scope: add the `syslog` crate dep and a third transport layer. Not shipping in the first commit series; file as a follow-up task if a real target arrives before we finish.

## Subscriber Init

- [x] `smctl_log::init(config: &LoggingConfig)` — composes stderr + file layers per config
- [x] `LoggingConfig` struct — `level`, `stderr`, `file`, `facility`, `app_name`
- [x] `init` is idempotent via `OnceLock` — second call is a silent no-op
- [ ] Config precedence resolver for `workspace.toml` `[logging]` — deferred along with the manifest schema (below)

## CLI Integration

- [x] Add `--log-file` / `--log-level` global flags to `smctl/src/main.rs`
- [ ] `--log-syslog` flag — deferred with the syslog transport
- [x] Wire `SMCTL_LOG_FILE` / `SMCTL_LOG_LEVEL` env vars
- [x] Cross-wire existing `--verbose` / `--quiet` to `LogLevel`
- [x] Call `smctl_log::init` at the top of `main()`, after clap parsing

## Workspace Manifest Schema

- [ ] Add `[logging]` section to `smctl-workspace::WorkspaceManifest` — **deferred** within this change. First commit series exposes logging only via CLI flags and env vars. Follow-up adds the manifest schema + precedence resolver.

## Instrumentation (initial events)

- [x] `SMCTL-0001` — workspace initialized (in the `WorkspaceCommands::Init` handler)
- [x] `SMCTL-0002` — spec created (in the `SpecCommands::New` handler)
- [x] `SMCTL-0003` — spec archived (in the `SpecCommands::Archive` handler)
- [x] `SMCTL-0004` — feature branch started (in the `SpecCommands::New` auto-branch flow)
- [x] `SMCTL-0005` — feature branch finished (in the `SpecCommands::Archive` auto-merge flow)
- [x] `SMCTL-0006` — build started (in the `Commands::Build` handler)
- [x] `SMCTL-0007` — build completed (in the `Commands::Build` handler, on success)
- [x] `SMCTL-0008` — build failed (in the `Commands::Build` handler, on failure)
- [x] `SMCTL-0099` — uncategorized error (fallback; emit from `main()` error branch)

Note: instrumentation lives at the CLI dispatch layer rather than inside each `smctl-*` library crate. This keeps the core crates free of a `smctl-log` dependency in v1. A follow-up can push events down into the library crates if cross-surface callers (e.g., `smctl-mcp`) need the same events.

## Tests

- [x] Unit test: severity mapping covers all five `tracing` levels
- [x] Unit test: MSGID `Display` produces canonical zero-padded form
- [x] Unit test: STRUCTURED-DATA escaping for `]`, `"`, `\`
- [x] Unit test: PRI calculation matches RFC for a sample facility/severity
- [x] Integration test: real `tracing::info!` event produces a full RFC 5424 line (5 cases: Info, Error, missing-MSGID fallback, empty SD nilvalue, special-character escape)
- [x] Integration test: `smctl workspace init --log-file tmp.log` writes `<134>1 … SMCTL-0001 …` to the file
- [x] Integration test: `--log-level banana` fails with a clear error
- [x] Full workspace `cargo test` (79 tests pass), `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --check` clean

## Docs

- [ ] Update `README.md` with a short Logging section
- [ ] Update `CLAUDE.md` Design system section with a one-line logging pointer
- [ ] Update `.claude/skills/smallaios-design/SKILL.md` with a pointer to the MSGID catalog

## Archive

- [ ] Run `smctl spec archive smctl-logging-v1` when all non-deferred tasks are complete and the change has merged to `develop`
