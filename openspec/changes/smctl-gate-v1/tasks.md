# smctl ModelGate Control — Tasks

## Crate Setup

- [ ] Create `smctl-gate` crate with Cargo.toml (reqwest, tokio dependencies)
- [ ] Add `smctl-gate` as workspace member in root Cargo.toml
- [ ] Add `smctl-gate` dependency to `smctl` binary crate
- [ ] Verify workspace builds with `cargo build --workspace`

## API Client Core

- [ ] Define `GateClient` struct with base URL, timeout, reqwest client
- [ ] Implement endpoint configuration resolution (CLI flag > env > workspace.toml > default)
- [ ] Define error types for gate operations (connection, HTTP, parse errors)
- [ ] Add `[gate]` section to workspace.toml schema

## Health & Status

- [ ] Implement `GET /health` client method
- [ ] Implement `smctl gate status` CLI subcommand
- [ ] Display instance version, uptime, model count, health state

## Model Management

- [ ] Implement `GET /api/v1/models` — list models
- [ ] Implement `POST /api/v1/models` — register model (with file streaming)
- [ ] Implement `DELETE /api/v1/models/{name}` — remove model
- [ ] Implement `smctl gate models list` CLI subcommand
- [ ] Implement `smctl gate models add <path>` CLI subcommand with progress bar
- [ ] Implement `smctl gate models remove <name>` CLI subcommand

## Route Configuration

- [ ] Implement `GET /api/v1/routes` — list routes
- [ ] Implement `PUT /api/v1/routes` — set route
- [ ] Implement `smctl gate routes list` CLI subcommand
- [ ] Implement `smctl gate routes set <model> <endpoint>` CLI subcommand

## Inference Testing

- [ ] Implement `POST /api/v1/inference/{model}` — test inference
- [ ] Implement `smctl gate test <model> --input <file>` CLI subcommand
- [ ] Display inference result, latency, and model metadata

## Log Streaming

- [ ] Implement `GET /api/v1/logs` SSE client for log streaming
- [ ] Implement `smctl gate logs [--follow]` CLI subcommand
- [ ] Handle graceful shutdown on Ctrl+C during log streaming

## Integration Testing

- [ ] Set up `wiremock` mock ModelGate server
- [ ] Test `gate status` against mock server
- [ ] Test `gate models list/add/remove` against mock server
- [ ] Test `gate routes list/set` against mock server
- [ ] Test `gate test` inference round-trip against mock server
- [ ] Test error handling (connection refused, 404, 500 responses)
- [ ] Test timeout and retry behavior

## CLI Integration

- [ ] Add `gate` subcommand to smctl command tree
- [ ] Wire `--json` output for all gate commands
- [ ] Wire `--dry-run` for mutating gate commands (models add, routes set)
- [ ] Add gate commands to shell completions

## Verify

- [ ] All gate subcommands work against mock server (integration tests)
- [ ] `--json` output is valid JSON for all gate commands
- [ ] Connection errors produce helpful messages (not panics)
- [ ] `cargo test --workspace` passes with new crate
- [ ] `cargo clippy --workspace -- -D warnings` passes
