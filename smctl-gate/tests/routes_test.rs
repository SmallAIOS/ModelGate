//! Integration tests for the routing surface.

use serde_json::json;
use smctl_gate::{GateClient, GateConfig, GateError};
use wiremock::matchers::{body_json, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(url: String) -> GateClient {
    GateClient::new(GateConfig {
        url,
        timeout_secs: 5,
    })
    .expect("client builds")
}

#[tokio::test]
async fn list_routes_returns_parsed_entries() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/routes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "model": "llama-7b",
                "endpoint": "/v1/chat/completions",
                "active": true,
                "request_count": 1234
            },
            {
                "model": "whisper-base",
                "endpoint": "/v1/audio/transcribe",
                "active": true,
                "request_count": 567
            }
        ])))
        .mount(&server)
        .await;

    let routes = client(server.uri()).list_routes().await.unwrap();
    assert_eq!(routes.len(), 2);
    assert!(routes[0].active);
    assert_eq!(routes[1].request_count, 567);
}

#[tokio::test]
async fn set_route_sends_expected_body_and_returns_route() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/routes"))
        .and(body_json(json!({
            "model": "phi-2",
            "endpoint": "/v1/chat/completions",
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "model": "phi-2",
            "endpoint": "/v1/chat/completions",
            "active": true,
            "request_count": 0,
        })))
        .mount(&server)
        .await;

    let got = client(server.uri())
        .set_route("phi-2", "/v1/chat/completions")
        .await
        .unwrap();
    assert_eq!(got.model, "phi-2");
    assert_eq!(got.endpoint, "/v1/chat/completions");
    assert!(got.active);
}

#[tokio::test]
async fn set_route_maps_404_to_model_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/routes"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let err = client(server.uri())
        .set_route("ghost", "/v1/x")
        .await
        .unwrap_err();
    match err {
        GateError::ModelNotFound { name } => assert_eq!(name, "ghost"),
        other => panic!("expected ModelNotFound, got {other:?}"),
    }
}
