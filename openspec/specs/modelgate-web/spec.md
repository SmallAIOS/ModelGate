# modelgate-web Specification

## Purpose

`modelgate-web` is the Axum server that serves the ModelGate dashboard SPA plus a JSON / SSE proxy at `/api/*`. It is launched via `smctl gate web`. The crate reuses `smctl_gate::GateClient` for every upstream call so the proxy never duplicates HTTP code.

## Requirements

### Requirement: Default loopback bind

`modelgate-web` SHALL default to binding `127.0.0.1:9378`. Non-loopback binds MUST emit a warning because the server has no authentication layer.

#### Scenario: Default bind is loopback

- **WHEN** the operator runs `smctl gate web` with no flags
- **THEN** the server MUST listen on `127.0.0.1:9378`

#### Scenario: Public bind warns

- **WHEN** the operator runs `smctl gate web --host 0.0.0.0`
- **THEN** the server MUST emit a stderr warning naming the lack of authentication
- **AND** MUST still proceed with the bind

### Requirement: Embedded SPA bundle

`modelgate-web` SHALL embed the React bundle from `ui/modelgate-web/dist/` at compile time via `include_dir!`. The build SHALL fail with a remediation clause when the dist directory is absent.

#### Scenario: Missing dist halts the Cargo build

- **WHEN** `cargo build -p modelgate-web` runs against a tree where `ui/modelgate-web/dist/` does not exist
- **THEN** the `build.rs` script MUST exit with a non-zero status
- **AND** the printed message MUST include `cd ui/modelgate-web && npm install && npm run build`

### Requirement: JSON proxy mapping

`modelgate-web` SHALL proxy these routes through `smctl_gate::GateClient`:

| Method + path | GateClient call |
|---|---|
| `GET /api/health` | `health()` |
| `GET /api/models` | `list_models()` |
| `POST /api/models` (multipart) | `add_model()` |
| `DELETE /api/models/{name}` | `remove_model()` |
| `GET /api/routes` | `list_routes()` |
| `PUT /api/routes` | `set_route()` |
| `POST /api/inference/{model}` | `test_inference()` |
| `GET /api/logs` (SSE) | `stream_logs()` |

#### Scenario: Health round-trip

- **WHEN** a client GETs `/api/health` against a server whose upstream returns `{ "status": "healthy", ... }`
- **THEN** the proxy MUST return 200 with the same JSON body

### Requirement: GateError to HTTP mapping

`modelgate-web` SHALL translate every `GateError` variant into a stable HTTP status and JSON envelope:

| Variant | HTTP | Body `error` field |
|---|---|---|
| `ConnectionRefused` | 502 | `upstream_unreachable` |
| `Timeout` | 504 | `upstream_timeout` |
| `HttpError { status, body }` | passthrough `status` | `upstream_error` |
| `ModelNotFound` | 404 | `model_not_found` |
| `FileNotFound` | 400 | `file_not_found` |
| `ParseError` | 502 | `upstream_parse_error` |

#### Scenario: Upstream timeout becomes 504

- **WHEN** the upstream takes longer than the client timeout to respond
- **THEN** the proxy MUST return HTTP 504 with `{ "error": "upstream_timeout", ... }`
- **AND** MUST emit `SMCTL-0304` at Warning severity

### Requirement: Static SPA fallback

`modelgate-web` SHALL serve the embedded `index.html` as the fallback for any non-`/api/*` GET. Unknown `/api/*` paths MUST return JSON 404 rather than the SPA shell.

#### Scenario: Hash-routed deep link reload

- **WHEN** a client GETs `/some-deep-link` that does not exist as a bundled asset
- **THEN** the server MUST return the embedded `index.html` so the SPA hash router can take over

#### Scenario: Unknown API path

- **WHEN** a client GETs `/api/not-a-real-route`
- **THEN** the server MUST return 404 with `Content-Type: application/json`
