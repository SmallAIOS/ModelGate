# ModelGate web — Server Specification

## Overview

The `modelgate-web` crate is an Axum application that serves (a) the built SPA bundle at `/` and (b) a thin JSON + SSE proxy at `/api/*` to the upstream ModelGate instance via `smctl_gate::GateClient`. The binary is not meant to run standalone — it is launched by the `smctl gate web` CLI subcommand.

## Public Surface

### Rust API

```rust
pub struct WebServerConfig {
    pub bind: SocketAddr,       // default 127.0.0.1:9378
    pub gate: smctl_gate::GateConfig,
}

pub async fn serve(config: WebServerConfig) -> Result<(), ServeError>;
```

Port `9378` is deliberately adjacent to the MCP server's 9377 (`smctl-mcp-v1` chose 9377 because M-C-P spells on a phone keypad). `9378` = MCP+1, memorable, still well clear of the IANA well-known range.

### HTTP Routes

| Route | Method | Backing `GateClient` call | Notes |
|---|---|---|---|
| `/` | GET | — | Serves `index.html` (embedded) |
| `/assets/*` | GET | — | Serves bundled JS/CSS (embedded) |
| `/api/health` | GET | `health()` | JSON |
| `/api/models` | GET | `list_models()` | JSON |
| `/api/models` | POST | `add_model()` | multipart form; streams to upstream |
| `/api/models/:name` | DELETE | `remove_model()` | 204 on success, 404 on `ModelNotFound` |
| `/api/routes` | GET | `list_routes()` | JSON |
| `/api/routes` | PUT | `set_route()` | JSON body `{model, endpoint}` |
| `/api/inference/:model` | POST | `test_inference()` | JSON in / JSON out |
| `/api/logs` | GET | `stream_logs()` | `text/event-stream` (SSE) passthrough |

Any route under `/api/*` that `smctl-gate` does not implement returns **501 Not Implemented** with a JSON body `{"error":"not_implemented","message":"..."}`. Stubbed panels in the SPA render the "not yet available" state against this status.

### Error Mapping

`GateError` → HTTP status:

| `GateError` variant | HTTP | Body |
|---|---|---|
| `ConnectionRefused` | 502 Bad Gateway | `{"error":"upstream_unreachable","message":...}` |
| `Timeout` | 504 Gateway Timeout | `{"error":"upstream_timeout","message":...}` |
| `HttpError { status, body }` | passthrough `status` | `{"error":"upstream_error","status":N,"body":"..."}` |
| `ModelNotFound { name }` | 404 Not Found | `{"error":"model_not_found","name":"..."}` |
| `FileNotFound { path }` | 400 Bad Request | `{"error":"file_not_found","path":"..."}` |
| `ParseError` | 502 Bad Gateway | `{"error":"upstream_parse_error","message":...}` |
| `InvalidUrl` | 500 Internal Server Error | `{"error":"invalid_config","message":...}` |
| `Io` | 500 Internal Server Error | `{"error":"io","message":...}` |
| `Transport` | 502 Bad Gateway | `{"error":"transport","message":...}` |

## CLI Entry Point

```
smctl gate web
    [--host 127.0.0.1]   # bind address; 0.0.0.0 is gated behind a second flag
    [--port 9378]        # bind port
    [--open]             # open http://<host>:<port> in the default browser
    [--dev]              # reserved: proxy / to a running vite dev server instead of embedded assets
```

`smctl gate web` inherits `--gate-url`, `--timeout`, `MODELGATE_URL`, and the `[gate]` section of `workspace.toml` via the same resolver used by `smctl gate status`.

## Logging

Server-side logs are emitted via the `smctl-log` MSGID catalog. The existing catalog reserves a future range for `modelgate-web`; the first MSGIDs land as part of the implementation commit:

- `SMCTL-0301` — web server started
- `SMCTL-0302` — web server stopped
- `SMCTL-0303` — upstream unreachable (proxied request)
- `SMCTL-0304` — upstream timeout (proxied request)

Each MSGID is defined in the catalog before first use, per the discipline established by `smctl-logging-v1`.

## Assets

The built SPA lives at `ui/modelgate-web/dist/` at Cargo-build time. The `modelgate-web` crate embeds it via `include_dir!("../ui/modelgate-web/dist")`. If that directory is missing, the crate fails to compile with a clear message pointing the operator at `npm run build` (build.rs check).

## Security

- Default bind is `127.0.0.1`. Binding to `0.0.0.0` requires an explicit `--bind-public` flag (to be added when auth lands; out of scope for this change).
- No cookies, no sessions. The SPA is stateless beyond `localStorage`.
- The SPA never exposes `MODELGATE_URL` to the browser — all upstream traffic goes through `/api/*`.
- Ambient CSRF risk is bounded by the localhost-only bind; once remote binds are enabled, a same-origin `Origin` header check is the first mitigation.
