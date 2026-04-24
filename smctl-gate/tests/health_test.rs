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

#[tokio::test]
async fn health_flags_timeout_when_server_delays_past_deadline() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_secs(2))
                .set_body_json(json!({
                    "status": "healthy",
                    "version": "0.0.0",
                    "uptime_secs": 0,
                    "model_count": 0,
                })),
        )
        .mount(&server)
        .await;

    // Build a client with a 1-second timeout so the 2-second server
    // delay reliably trips the deadline.
    let c = GateClient::new(GateConfig {
        url: server.uri(),
        timeout_secs: 1,
    })
    .unwrap();

    let err = c.health().await.unwrap_err();
    assert!(
        matches!(err, GateError::Timeout { .. }),
        "expected Timeout, got {err:?}"
    );
}

#[tokio::test]
async fn health_surfaces_404_as_http_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(404).set_body_string("no such endpoint"))
        .mount(&server)
        .await;

    let err = client(server.uri()).health().await.unwrap_err();
    match err {
        GateError::HttpError { status, body } => {
            assert_eq!(status, 404);
            assert_eq!(body, "no such endpoint");
        }
        other => panic!("expected HttpError(404), got {other:?}"),
    }
}
