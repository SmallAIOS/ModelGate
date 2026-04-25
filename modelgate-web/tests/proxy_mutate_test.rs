//! Integration tests for modelgate-web mutating proxy handlers:
//! POST /api/models, DELETE /api/models/:name, PUT /api/routes,
//! POST /api/inference/:model.

use serde_json::json;
use smctl_gate::{GateClient, GateConfig};
use wiremock::matchers::{body_json, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn spawn_server(upstream_url: String) -> String {
    let client = GateClient::new(GateConfig {
        url: upstream_url,
        timeout_secs: 5,
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
async fn delete_model_forwards_to_upstream() {
    let upstream = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/models/phi-2"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::Client::new()
        .delete(format!("{base}/api/models/phi-2"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}

#[tokio::test]
async fn delete_model_maps_upstream_404_to_model_not_found() {
    let upstream = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path_regex(r"^/api/v1/models/.*$"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::Client::new()
        .delete(format!("{base}/api/models/ghost"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "model_not_found");
    assert_eq!(body["name"], "ghost");
}

#[tokio::test]
async fn put_route_forwards_body_and_returns_route() {
    let upstream = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/routes"))
        .and(body_json(
            json!({"model": "phi-2", "endpoint": "/v1/chat/completions"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "model": "phi-2",
            "endpoint": "/v1/chat/completions",
            "active": true,
            "request_count": 0,
        })))
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::Client::new()
        .put(format!("{base}/api/routes"))
        .json(&json!({"model": "phi-2", "endpoint": "/v1/chat/completions"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["model"], "phi-2");
    assert_eq!(body["active"], true);
}

#[tokio::test]
async fn put_route_maps_upstream_404_to_model_not_found() {
    let upstream = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/api/v1/routes"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::Client::new()
        .put(format!("{base}/api/routes"))
        .json(&json!({"model": "ghost", "endpoint": "/nope"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "model_not_found");
    assert_eq!(body["name"], "ghost");
}

#[tokio::test]
async fn post_inference_round_trips_payload() {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/inference/llama-7b"))
        .and(body_json(json!({"prompt": "hello"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "model": "llama-7b",
            "output": {"text": "world"},
            "latency_ms": 42,
            "tokens_generated": 3,
        })))
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::Client::new()
        .post(format!("{base}/api/inference/llama-7b"))
        .json(&json!({"prompt": "hello"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["model"], "llama-7b");
    assert_eq!(body["latency_ms"], 42);
}

#[tokio::test]
async fn post_models_streams_multipart_upload() {
    let upstream = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name": "phi-2",
            "format": "onnx",
            "size_bytes": 1024,
            "registered_at": "2026-04-24",
            "status": "loaded",
        })))
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(vec![0u8; 1024])
            .file_name("phi-2.onnx")
            .mime_str("application/octet-stream")
            .unwrap(),
    );

    let resp = reqwest::Client::new()
        .post(format!("{base}/api/models"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "phi-2");
}

#[tokio::test]
async fn post_models_rejects_missing_file_part() {
    let upstream = MockServer::start().await;
    let base = spawn_server(upstream.uri()).await;

    let form = reqwest::multipart::Form::new().text("name", "phi-2");
    let resp = reqwest::Client::new()
        .post(format!("{base}/api/models"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "bad_request");
}
