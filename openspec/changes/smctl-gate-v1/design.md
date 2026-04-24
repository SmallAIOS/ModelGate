# smctl ModelGate Control — Design Document

## Context

ModelGate is the AI model inference gateway for the SmallAIOS ecosystem. It routes inference requests to registered ONNX models, manages model lifecycle, and provides health monitoring. Currently there is no CLI interface for interacting with running ModelGate instances — developers use raw HTTP calls. This change adds `smctl gate` subcommands as a control-plane CLI.

## Goals / Non-Goals

### Goals

1. Provide a `kubectl`-style CLI for ModelGate instance management
2. Support model registration, listing, and removal
3. Support routing table inspection and configuration
4. Enable test inference from the command line
5. Provide health checks and log streaming
6. Integration tests using a mock ModelGate server

### Non-Goals

1. Not replacing the ModelGate HTTP API — `smctl gate` is a client, not a server
2. Not managing ModelGate deployment — this is a local dev tool, not infrastructure management
3. Not handling model training or conversion — only model registration and inference
4. Not implementing authentication/authorization — deferred to when ModelGate adds auth

## Decisions

### Decision 1: reqwest-based API client

**Choice:** Use `reqwest` as the HTTP client for communicating with ModelGate's REST API.

**Rationale:** `reqwest` is the standard async HTTP client in the Rust ecosystem. It supports connection pooling, timeouts, retries, and TLS — all needed for a reliable API client. It's already used broadly across Rust CLI tools.

### Decision 2: ModelGate endpoint configuration

**Choice:** ModelGate instance URL is configured via:
1. `--gate-url` CLI flag (highest priority)
2. `MODELGATE_URL` environment variable
3. `gate.url` in workspace.toml
4. Default: `http://localhost:8080`

```toml
# workspace.toml
[gate]
url = "http://localhost:8080"
timeout_secs = 30
```

**Rationale:** Follows the same three-tier config pattern (CLI > env > workspace) used throughout smctl. The default localhost URL works for local development without any configuration.

### Decision 3: Structured output matching smctl conventions

**Choice:** All `smctl gate` commands support `--json` for structured output, consistent with the rest of smctl.

**Rationale:** Consistency with existing smctl commands. Also enables future MCP tool integration — when smctl-mcp-v1 ships, gate commands automatically get MCP tool equivalents.

### Decision 4: Mock server for integration testing

**Choice:** Use `wiremock` crate to create a mock ModelGate HTTP server for integration tests.

**Rationale:** Integration tests need a predictable HTTP server without running a real ModelGate instance. `wiremock` provides request matching, response stubbing, and verification — purpose-built for HTTP client testing in Rust.

### Decision 5: API contract based on ModelGate REST endpoints

**Choice:** The `smctl-gate` client targets the following ModelGate REST API surface:

| Endpoint | Method | smctl command |
|---|---|---|
| `/health` | GET | `smctl gate status` |
| `/api/v1/models` | GET | `smctl gate models list` |
| `/api/v1/models` | POST | `smctl gate models add` |
| `/api/v1/models/{name}` | DELETE | `smctl gate models remove` |
| `/api/v1/routes` | GET | `smctl gate routes list` |
| `/api/v1/routes` | PUT | `smctl gate routes set` |
| `/api/v1/inference/{model}` | POST | `smctl gate test` |
| `/api/v1/logs` | GET (SSE) | `smctl gate logs` |

**Rationale:** RESTful conventions. The API surface is minimal but covers the core model gateway operations. The exact endpoints may need adjustment once the ModelGate server API is finalized — the client should be resilient to minor endpoint changes.

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| ModelGate API not yet finalized | Define client against expected API; use trait abstraction so implementation can adapt |
| Network errors during gate commands | Implement retry with backoff for transient failures; clear error messages for connection refused |
| Model file size for `models add` | Stream upload with progress bar; don't load entire file into memory |
| Log streaming may be long-lived | `gate logs --follow` uses SSE with graceful shutdown on Ctrl+C |

## Open Questions

1. Should `smctl gate` support multiple ModelGate instances (e.g., dev, staging, prod profiles)?
2. Should `smctl gate models add` support registering models from URLs (not just local paths)?
3. What model metadata should be displayed in `models list` (size, format, last used, inference count)?
