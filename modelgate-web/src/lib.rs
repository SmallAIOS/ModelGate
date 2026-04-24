//! modelgate-web — Axum server that fronts the ModelGate dashboard SPA.
//!
//! Two jobs: (a) serve the built React bundle, (b) proxy JSON + SSE calls
//! through to the upstream ModelGate instance via
//! [`smctl_gate::GateClient`]. The crate is wired into the `smctl` CLI
//! under `smctl gate web`; it is not meant to run standalone.
//!
//! Status: scaffold only. Route handlers live in follow-up commits on
//! this branch. This lib currently exposes the public shape
//! (`WebServerConfig`, `serve`) and a health route so the workspace
//! builds end-to-end before we pull in the frontend.

use std::net::SocketAddr;

use axum::{Json, Router, routing::get};
use serde::Serialize;

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
    Router::new()
        .route("/api/_ping", get(ping))
        .with_state(client)
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

    tracing::info!(addr = %config.bind, "modelgate-web server started");

    axum::serve(listener, app)
        .await
        .map_err(|source| ServeError::Serve { source })
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
