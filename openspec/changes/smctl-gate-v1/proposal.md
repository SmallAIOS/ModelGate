# smctl ModelGate Control — Proposal

## Why

`smctl` v0.1.3 manages the SmallAIOS multi-repo workspace, git flow, specs, and builds. However, it has no way to interact with running ModelGate instances — the model inference gateway that routes requests to ONNX models. Developers currently must use raw HTTP requests or write ad-hoc scripts to check ModelGate health, register models, configure routes, test inference, and view logs.

Key problems this solves:

- **No CLI for ModelGate operations.** Developers use curl or custom scripts to interact with ModelGate's API during development.
- **No model lifecycle management.** Adding, removing, and inspecting registered ONNX models requires direct API calls.
- **No routing visibility.** The inference routing table is opaque without a structured query interface.
- **No integrated health monitoring.** Checking whether ModelGate instances are running and healthy is manual.

## What Changes

Add `smctl gate` subcommands that provide a control-plane CLI for ModelGate instances, mirroring how `kubectl` interacts with Kubernetes clusters.

### New Crate

- **`smctl-gate`** — ModelGate API client crate (reqwest-based) implementing health checks, model management, route configuration, inference testing, and log streaming.

### New CLI Surface

- `smctl gate status` — Show running ModelGate instances and health
- `smctl gate models list` — List registered ONNX models with metadata
- `smctl gate models add <path>` — Register a new ONNX model
- `smctl gate models remove <name>` — Unregister a model
- `smctl gate routes list` — Show inference routing table
- `smctl gate routes set <model> <endpoint>` — Configure routing for a model
- `smctl gate test <model> --input <file>` — Run test inference against a model
- `smctl gate logs [--follow]` — Stream ModelGate logs

## Capabilities

### New Capabilities

- `smctl-gate` crate — ModelGate API client with reqwest
- Gate CLI subcommands — Full model/route/health management
- Mock ModelGate server — For integration testing

### Modified Capabilities

- `smctl` CLI binary — Add `gate` subcommand to existing command tree
- MCP tool registry (future) — Gate tools will be exposed as MCP tools once smctl-mcp-v1 ships

## Impact

### New Files

```
smctl-gate/
├── Cargo.toml
└── src/
    ├── lib.rs          # API client, types, error handling
    ├── client.rs       # reqwest-based HTTP client
    ├── models.rs       # Model CRUD operations
    ├── routes.rs       # Routing table operations
    └── health.rs       # Health check and status
```

### Modified Files

- `Cargo.toml` — Add `smctl-gate` workspace member
- `smctl/Cargo.toml` — Add `smctl-gate` dependency
- `smctl/src/main.rs` — Add `gate` subcommand

### Dependencies

- `reqwest` (0.12+) — HTTP client for ModelGate API
- `tokio` (1.x) — Async runtime for HTTP operations
- `wiremock` or `mockito` — Mock HTTP server for integration tests

## References

- [smctl design document](../smctl-tool-v1/design.md) — Decision 7 (ModelGate subcommands)
- [kubectl CLI conventions](https://kubernetes.io/docs/reference/kubectl/)
- [ModelGate architecture](../../README.md)
