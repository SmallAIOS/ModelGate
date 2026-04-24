//! Integration tests for smctl-gate against a mock ModelGate server.
//!
//! Uses `wiremock` to stub the HTTP surface defined in gate-api.md so tests
//! exercise the full reqwest path without a real ModelGate instance.

use serde_json::json;
use smctl_gate::{GateClient, GateConfig, GateError};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(url: String) -> GateClient {
    GateClient::new(GateConfig {
        url,
        timeout_secs: 5,
    })
    .expect("client builds")
}

#[tokio::test]
async fn health_returns_parsed_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy",
            "version": "0.2.0",
            "uptime_secs": 8040,
            "model_count": 3,
        })))
        .mount(&server)
        .await;

    let got = client(server.uri()).health().await.expect("health ok");
    assert_eq!(got.status, "healthy");
    assert_eq!(got.version, "0.2.0");
    assert_eq!(got.uptime_secs, 8040);
    assert_eq!(got.model_count, 3);
}

#[tokio::test]
async fn health_surfaces_http_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(503).set_body_string("degraded"))
        .mount(&server)
        .await;

    let err = client(server.uri()).health().await.unwrap_err();
    match err {
        GateError::HttpError { status, body } => {
            assert_eq!(status, 503);
            assert_eq!(body, "degraded");
        }
        other => panic!("expected HttpError, got {other:?}"),
    }
}

#[tokio::test]
async fn health_flags_connection_refused_against_dead_port() {
    let err = client("http://127.0.0.1:1".into()).health().await.unwrap_err();
    assert!(
        matches!(err, GateError::ConnectionRefused { .. }),
        "expected ConnectionRefused, got {err:?}"
    );
}
