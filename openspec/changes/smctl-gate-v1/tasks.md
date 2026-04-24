# smctl ModelGate Control — Tasks

## Crate Setup

- [x] Create `smctl-gate` crate with Cargo.toml (reqwest, tokio dependencies)
- [x] Add `smctl-gate` as workspace member in root Cargo.toml
- [x] Add `smctl-gate` dependency to `smctl` binary crate
- [x] Verify workspace builds with `cargo build --workspace`

## API Client Core

- [x] Define `GateClient` struct with base URL, timeout, reqwest client
- [x] Implement endpoint configuration resolution (CLI flag > env > workspace.toml > default)
- [x] Define error types for gate operations (connection, HTTP, parse errors)
- [x] Add `[gate]` section to workspace.toml schema

## Health & Status

- [x] Implement `GET /health` client method
- [x] Implement `smctl gate status` CLI subcommand
- [x] Display instance version, uptime, model count, health state

## Model Management

- [x] Implement `GET /api/v1/models` — list models
- [x] Implement `POST /api/v1/models` — register model (with file streaming)
- [x] Implement `DELETE /api/v1/models/{name}` — remove model
- [x] Implement `smctl gate models list` CLI subcommand
- [x] Implement `smctl gate models add <path>` CLI subcommand with progress bar
- [x] Implement `smctl gate models remove <name>` CLI subcommand

## Route Configuration

- [x] Implement `GET /api/v1/routes` — list routes
- [x] Implement `PUT /api/v1/routes` — set route
- [x] Implement `smctl gate routes list` CLI subcommand
- [x] Implement `smctl gate routes set <model> <endpoint>` CLI subcommand

## Inference Testing

- [x] Implement `POST /api/v1/inference/{model}` — test inference
- [x] Implement `smctl gate test <model> --input <file>` CLI subcommand
- [x] Display inference result, latency, and model metadata

## Log Streaming

- [x] Implement `GET /api/v1/logs` SSE client for log streaming
- [x] Implement `smctl gate logs [--follow]` CLI subcommand
- [x] Handle graceful shutdown on Ctrl+C during log streaming

## Integration Testing

- [x] Set up `wiremock` mock ModelGate server
- [x] Test `gate status` against mock server
- [x] Test `gate models list/add/remove` against mock server
- [x] Test `gate routes list/set` against mock server
- [x] Test `gate test` inference round-trip against mock server
- [x] Test error handling (connection refused, 404, 500 responses)
- [x] Test timeout and retry behavior — timeout covered; retry deferred (see Open Questions in design.md)

## CLI Integration

- [x] Add `gate` subcommand to smctl command tree
- [x] Wire `--json` output for all gate commands
- [x] Wire `--dry-run` for mutating gate commands (models add, routes set)
- [x] Add gate commands to shell completions — handled by clap_complete via the Subcommand derive

## Verify

- [x] All gate subcommands work against mock server (integration tests)
- [x] `--json` output is valid JSON for all gate commands
- [x] Connection errors produce helpful messages (not panics)
- [x] `cargo test --workspace` passes with new crate
- [x] `cargo clippy --workspace -- -D warnings` passes
