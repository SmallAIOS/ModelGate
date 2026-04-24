//! modelgate-web — Axum server that fronts the ModelGate dashboard SPA.
//!
//! Two jobs: (a) serve the built React bundle, (b) proxy JSON + SSE calls
//! through to the upstream ModelGate instance via
//! [`smctl_gate::GateClient`]. The crate is wired into the `smctl` CLI
//! under `smctl gate web`; it is not meant to run standalone.
//!
//! Status: Axum server + full JSON `/api/*` proxy surface (health,
//! models list/add/remove, routes list/set, inference). SSE log
//! passthrough and static-asset embedding land in follow-up commits.

use std::net::SocketAddr;

use axum::extract::{Multipart, Path, State};
use axum::http::{StatusCode, header};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router, routing::get};
use futures_util::StreamExt;
use include_dir::{Dir, include_dir};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;

/// Built SPA bundle. Produced by `npm run build` in
/// `ui/modelgate-web/`. `build.rs` verifies the directory exists
/// before this macro runs.
static SPA_DIST: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../ui/modelgate-web/dist");

pub const DEFAULT_BIND: SocketAddr = SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
    9378,
);

#[derive(Debug, Clone)]
pub struct WebServerConfig {
    pub bind: SocketAddr,
    pub gate: smctl_gate::GateConfig,
}

impl Default for WebServerConfig {
    fn default() -> Self {
        Self {
            bind: DEFAULT_BIND,
            gate: smctl_gate::GateConfig::default(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    #[error("failed to bind {addr}: {source}")]
    Bind {
        addr: SocketAddr,
        #[source]
        source: std::io::Error,
    },

    #[error("server error: {source}")]
    Serve {
        #[source]
        source: std::io::Error,
    },

    #[error("gate client construction failed: {source}")]
    Gate {
        #[source]
        source: smctl_gate::GateError,
    },
}

/// Build the Axum router for the dashboard.
///
/// The router carries the [`smctl_gate::GateClient`] as state so each
/// `/api/*` handler can reach the upstream without re-constructing the
/// HTTP layer per-request.
pub fn router(client: smctl_gate::GateClient) -> Router {
    use axum::routing::{delete, post};
    Router::new()
        .route("/api/_ping", get(ping))
        .route("/api/health", get(api_health))
        .route("/api/models", get(api_list_models).post(api_add_model))
        .route("/api/models/{name}", delete(api_remove_model))
        .route("/api/routes", get(api_list_routes).put(api_set_route))
        .route("/api/inference/{model}", post(api_test_inference))
        .route("/api/logs", get(api_stream_logs))
        .with_state(client)
        .fallback(serve_spa)
}

/// Serves the built SPA for any non-`/api/*` request.
///
/// Exact asset paths (e.g. `/assets/index-abc.js`) resolve directly
/// against the embedded bundle. Everything else falls back to
/// `index.html` so the hash-router in the SPA can handle client-side
/// routing without 404s on reload.
///
/// Unmatched `/api/*` paths return a JSON 404 instead of the SPA shell,
/// so machine clients that check content-type see an honest failure.
async fn serve_spa(req: axum::extract::Request) -> Response {
    let raw_path = req.uri().path();
    if raw_path.starts_with("/api/") || raw_path == "/api" {
        let body = serde_json::json!({
            "error": "not_found",
            "message": format!("no route for {raw_path}"),
        });
        return (StatusCode::NOT_FOUND, Json(body)).into_response();
    }

    let path = raw_path.trim_start_matches('/');
    let file = SPA_DIST
        .get_file(path)
        .or_else(|| SPA_DIST.get_file("index.html"));
    match file {
        Some(f) => {
            let mime = mime_guess::from_path(f.path())
                .first_or_octet_stream()
                .as_ref()
                .to_string();
            ([(header::CONTENT_TYPE, mime)], f.contents().to_vec()).into_response()
        }
        None => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "index.html missing from bundle",
        )
            .into_response(),
    }
}

#[derive(Debug, Serialize)]
struct Ping {
    ok: bool,
    server: &'static str,
}

async fn ping() -> Json<Ping> {
    Json(Ping {
        ok: true,
        server: "modelgate-web",
    })
}

async fn api_health(State(client): State<smctl_gate::GateClient>) -> Result<Response, ApiError> {
    let health = client.health().await.map_err(ApiError::from)?;
    Ok(Json(health).into_response())
}

async fn api_list_models(
    State(client): State<smctl_gate::GateClient>,
) -> Result<Response, ApiError> {
    let models = client.list_models().await.map_err(ApiError::from)?;
    Ok(Json(models).into_response())
}

async fn api_list_routes(
    State(client): State<smctl_gate::GateClient>,
) -> Result<Response, ApiError> {
    let routes = client.list_routes().await.map_err(ApiError::from)?;
    Ok(Json(routes).into_response())
}

async fn api_remove_model(
    State(client): State<smctl_gate::GateClient>,
    Path(name): Path<String>,
) -> Result<Response, ApiError> {
    client.remove_model(&name).await.map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

#[derive(Debug, Deserialize)]
struct SetRouteBody {
    model: String,
    endpoint: String,
}

async fn api_set_route(
    State(client): State<smctl_gate::GateClient>,
    Json(body): Json<SetRouteBody>,
) -> Result<Response, ApiError> {
    let route = client
        .set_route(&body.model, &body.endpoint)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(route).into_response())
}

async fn api_test_inference(
    State(client): State<smctl_gate::GateClient>,
    Path(model): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Response, ApiError> {
    // GateClient::test_inference reads the input from disk; stage the
    // posted JSON in a tempfile, call through, and clean up.
    let dir = tempfile::tempdir().map_err(|source| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        kind: "io",
        message: format!("failed to create temp dir: {source}"),
        extra: serde_json::Value::Null,
    })?;
    let tmp = dir.path().join("input.json");
    let bytes = serde_json::to_vec(&payload).map_err(|source| ApiError {
        status: StatusCode::BAD_REQUEST,
        kind: "bad_request",
        message: format!("failed to re-serialize JSON: {source}"),
        extra: serde_json::Value::Null,
    })?;
    tokio::fs::write(&tmp, &bytes)
        .await
        .map_err(|source| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            kind: "io",
            message: format!("failed to write temp input: {source}"),
            extra: serde_json::Value::Null,
        })?;

    let result = client
        .test_inference(&model, &tmp)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(result).into_response())
}

/// Receive a multipart upload from the browser and stream it to the
/// upstream via `GateClient::add_model`. The file is staged to a
/// tempfile because `GateClient::add_model` takes a `&Path`; streaming
/// through to the upstream without a tempfile is a future optimization
/// once `smctl-gate` grows a byte-stream variant.
async fn api_add_model(
    State(client): State<smctl_gate::GateClient>,
    mut multipart: Multipart,
) -> Result<Response, ApiError> {
    let dir = tempfile::tempdir().map_err(|source| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        kind: "io",
        message: format!("failed to create temp dir: {source}"),
        extra: serde_json::Value::Null,
    })?;

    let mut file_path: Option<std::path::PathBuf> = None;

    while let Some(field) = multipart.next_field().await.map_err(|source| ApiError {
        status: StatusCode::BAD_REQUEST,
        kind: "bad_request",
        message: format!("multipart read failed: {source}"),
        extra: serde_json::Value::Null,
    })? {
        let field_name = field.name().unwrap_or_default().to_string();
        if field_name != "file" {
            // Silently drop other parts (e.g. "name") — GateClient derives
            // the model name from the file stem.
            continue;
        }

        let filename = field.file_name().unwrap_or("upload.bin").to_string();
        let target = dir.path().join(&filename);
        let mut out = tokio::fs::File::create(&target)
            .await
            .map_err(|source| ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                kind: "io",
                message: format!("failed to open temp file: {source}"),
                extra: serde_json::Value::Null,
            })?;

        let mut field = field;
        while let Some(chunk) = field.chunk().await.map_err(|source| ApiError {
            status: StatusCode::BAD_REQUEST,
            kind: "bad_request",
            message: format!("multipart chunk read failed: {source}"),
            extra: serde_json::Value::Null,
        })? {
            out.write_all(&chunk).await.map_err(|source| ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                kind: "io",
                message: format!("failed to write temp file: {source}"),
                extra: serde_json::Value::Null,
            })?;
        }
        out.flush().await.map_err(|source| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            kind: "io",
            message: format!("failed to flush temp file: {source}"),
            extra: serde_json::Value::Null,
        })?;

        file_path = Some(target);
        break;
    }

    let Some(file_path) = file_path else {
        return Err(ApiError {
            status: StatusCode::BAD_REQUEST,
            kind: "bad_request",
            message: "multipart upload missing `file` part".into(),
            extra: serde_json::Value::Null,
        });
    };

    let model = client
        .add_model(&file_path, None)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(model).into_response())
}

/// SSE passthrough for the upstream log stream. The browser connects
/// once and receives a live event-stream of `LogEntry` JSON payloads.
/// Each entry is re-emitted as one SSE `data:` frame.
async fn api_stream_logs(
    State(client): State<smctl_gate::GateClient>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, std::convert::Infallible>>>, ApiError>
{
    let upstream = client.stream_logs().await.map_err(ApiError::from)?;

    let events = upstream.map(|result| {
        let event = match result {
            Ok(entry) => match serde_json::to_string(&entry) {
                Ok(json) => Event::default().data(json),
                Err(e) => Event::default()
                    .event("error")
                    .data(format!("log entry re-serialisation failed: {e}")),
            },
            Err(e) => Event::default()
                .event("error")
                .data(format!("upstream log stream error: {e}")),
        };
        Ok::<_, std::convert::Infallible>(event)
    });

    Ok(Sse::new(events))
}

// --- Error mapping ---

/// Handler-local wrapper that maps a `GateError` to the HTTP status and
/// JSON body defined in `openspec/changes/modelgate-web-v1/specs/web-server.md`.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub kind: &'static str,
    pub message: String,
    pub extra: serde_json::Value,
}

impl ApiError {
    fn body(&self) -> serde_json::Value {
        let mut body = serde_json::json!({
            "error": self.kind,
            "message": self.message,
        });
        if let Some(obj) = body.as_object_mut()
            && let Some(extra_obj) = self.extra.as_object()
        {
            for (k, v) in extra_obj {
                obj.insert(k.clone(), v.clone());
            }
        }
        body
    }
}

impl From<smctl_gate::GateError> for ApiError {
    fn from(err: smctl_gate::GateError) -> Self {
        use smctl_gate::GateError as G;
        let message = err.to_string();
        match err {
            G::ConnectionRefused { ref url } => {
                tracing::warn!(
                    msgid = %smctl_log::MsgId::WebUpstreamUnreachable,
                    upstream = %url,
                    "upstream ModelGate unreachable"
                );
                ApiError {
                    status: StatusCode::BAD_GATEWAY,
                    kind: "upstream_unreachable",
                    message,
                    extra: serde_json::Value::Null,
                }
            }
            G::Timeout { timeout_secs } => {
                tracing::warn!(
                    msgid = %smctl_log::MsgId::WebUpstreamTimeout,
                    timeout_secs = timeout_secs,
                    "upstream ModelGate timed out"
                );
                ApiError {
                    status: StatusCode::GATEWAY_TIMEOUT,
                    kind: "upstream_timeout",
                    message,
                    extra: serde_json::Value::Null,
                }
            }
            G::HttpError { status, body } => ApiError {
                status: StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY),
                kind: "upstream_error",
                message: format!("HTTP {status}: {body}"),
                extra: serde_json::json!({ "status": status, "body": body }),
            },
            G::ModelNotFound { name } => ApiError {
                status: StatusCode::NOT_FOUND,
                kind: "model_not_found",
                message,
                extra: serde_json::json!({ "name": name }),
            },
            G::FileNotFound { path } => ApiError {
                status: StatusCode::BAD_REQUEST,
                kind: "file_not_found",
                message,
                extra: serde_json::json!({ "path": path }),
            },
            G::ParseError { .. } => ApiError {
                status: StatusCode::BAD_GATEWAY,
                kind: "upstream_parse_error",
                message,
                extra: serde_json::Value::Null,
            },
            G::Transport { .. } => ApiError {
                status: StatusCode::BAD_GATEWAY,
                kind: "transport",
                message,
                extra: serde_json::Value::Null,
            },
            G::Io { .. } => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                kind: "io",
                message,
                extra: serde_json::Value::Null,
            },
            G::InvalidUrl { .. } => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                kind: "invalid_config",
                message,
                extra: serde_json::Value::Null,
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body())).into_response()
    }
}

// Unused for now but exported so future handlers don't need to rewrite
// the import list. Silences dead-code warnings when the proxy only uses
// GET routes.
#[allow(dead_code)]
fn _reserve_path_extractor(_: Path<String>) {}

/// Bind and serve the dashboard until the process is signalled.
pub async fn serve(config: WebServerConfig) -> Result<(), ServeError> {
    let client = smctl_gate::GateClient::new(config.gate.clone())
        .map_err(|source| ServeError::Gate { source })?;
    let app = router(client);

    let listener = tokio::net::TcpListener::bind(config.bind)
        .await
        .map_err(|source| ServeError::Bind {
            addr: config.bind,
            source,
        })?;

    tracing::info!(
        msgid = %smctl_log::MsgId::WebServerStarted,
        addr = %config.bind,
        upstream = %config.gate.url,
        "modelgate-web server started"
    );

    let serve_result = axum::serve(listener, app)
        .await
        .map_err(|source| ServeError::Serve { source });

    tracing::info!(
        msgid = %smctl_log::MsgId::WebServerStopped,
        addr = %config.bind,
        "modelgate-web server stopped"
    );

    serve_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_bind_is_local_9378() {
        let cfg = WebServerConfig::default();
        assert_eq!(cfg.bind.port(), 9378);
        assert!(cfg.bind.ip().is_loopback());
    }

    #[tokio::test]
    async fn router_answers_ping() {
        let client = smctl_gate::GateClient::new(smctl_gate::GateConfig::default()).unwrap();
        let app = router(client);

        // Pin the server to an ephemeral port so multiple test cases can
        // run in parallel without clashing.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let body: serde_json::Value = reqwest::get(format!("http://{addr}/api/_ping"))
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(body["ok"], true);
        assert_eq!(body["server"], "modelgate-web");

        server.abort();
    }
}
