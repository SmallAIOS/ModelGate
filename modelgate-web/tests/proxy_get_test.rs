//! Integration tests for modelgate-web GET proxy handlers.
//!
//! Spins up the real axum router with a GateClient pointed at a wiremock
//! ModelGate. Exercises the full request path: HTTP -> axum -> GateClient
//! -> wiremock -> GateClient -> axum -> HTTP. Covers status mapping from
//! every `GateError` variant that a GET route can produce.

use serde_json::json;
use smctl_gate::{GateClient, GateConfig};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn spawn_server(upstream_url: String) -> String {
    let client = GateClient::new(GateConfig {
        url: upstream_url,
        timeout_secs: 2,
    })
    .unwrap();
    let app = modelgate_web::router(client);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn api_health_round_trips_from_upstream() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "healthy",
            "version": "0.2.0",
            "uptime_secs": 42,
            "model_count": 1,
        })))
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::get(format!("{base}/api/health")).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["model_count"], 1);
}

#[tokio::test]
async fn api_models_round_trips_from_upstream() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"name": "phi-2", "format": "onnx", "size_bytes": 1024, "status": "loaded"},
        ])))
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::get(format!("{base}/api/models")).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body[0]["name"], "phi-2");
}

#[tokio::test]
async fn api_routes_round_trips_from_upstream() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/routes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::get(format!("{base}/api/routes")).await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array());
}

#[tokio::test]
async fn api_health_maps_upstream_unreachable_to_502() {
    // Point the GateClient at a port that isn't listening. No wiremock
    // needed — we're exercising the ConnectionRefused path.
    let base = spawn_server("http://127.0.0.1:1".to_string()).await;
    let resp = reqwest::get(format!("{base}/api/health")).await.unwrap();
    assert_eq!(resp.status(), 502);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "upstream_unreachable");
}

#[tokio::test]
async fn api_health_maps_upstream_timeout_to_504() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(std::time::Duration::from_secs(3))
                .set_body_json(json!({
                    "status": "healthy",
                    "version": "x",
                    "uptime_secs": 0,
                    "model_count": 0,
                })),
        )
        .mount(&upstream)
        .await;

    // spawn_server uses a 2s client timeout, so the 3s upstream delay
    // reliably trips the deadline.
    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::get(format!("{base}/api/health")).await.unwrap();
    assert_eq!(resp.status(), 504);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "upstream_timeout");
}

#[tokio::test]
async fn api_health_passes_through_upstream_5xx_status() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/health"))
        .respond_with(ResponseTemplate::new(503).set_body_string("degraded"))
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::get(format!("{base}/api/health")).await.unwrap();
    assert_eq!(resp.status(), 503);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "upstream_error");
    assert_eq!(body["status"], 503);
}
