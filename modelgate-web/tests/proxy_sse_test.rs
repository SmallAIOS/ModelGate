//! Integration test for the SSE log passthrough at /api/logs.

use smctl_gate::{GateClient, GateConfig};
use wiremock::matchers::{method, path};
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

fn sse_frame(json: &str) -> String {
    format!("data: {json}\n\n")
}

#[tokio::test]
async fn logs_forwards_sse_frames_from_upstream() {
    let upstream = MockServer::start().await;
    let body = [
        sse_frame(
            r#"{"timestamp":"2026-04-24T12:00:00Z","level":"INFO","message":"hello","fields":{}}"#,
        ),
        sse_frame(
            r#"{"timestamp":"2026-04-24T12:00:01Z","level":"WARN","message":"oops","fields":{"k":1}}"#,
        ),
    ]
    .concat();

    Mock::given(method("GET"))
        .and(path("/api/v1/logs"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::Client::new()
        .get(format!("{base}/api/logs"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert!(
        content_type.starts_with("text/event-stream"),
        "content-type should be SSE, got {content_type}"
    );

    let body = resp.text().await.unwrap();

    // The response should contain one `data:` frame per upstream log
    // entry, with the LogEntry JSON preserved intact.
    assert!(
        body.contains("\"message\":\"hello\""),
        "expected first entry in SSE body: {body}"
    );
    assert!(
        body.contains("\"message\":\"oops\""),
        "expected second entry in SSE body: {body}"
    );
    assert_eq!(
        body.matches("data:").count(),
        2,
        "expected exactly two SSE data frames, got:\n{body}"
    );
}

#[tokio::test]
async fn logs_maps_upstream_5xx_to_upstream_error() {
    let upstream = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/logs"))
        .respond_with(ResponseTemplate::new(503).set_body_string("not ready"))
        .mount(&upstream)
        .await;

    let base = spawn_server(upstream.uri()).await;
    let resp = reqwest::Client::new()
        .get(format!("{base}/api/logs"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 503);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "upstream_error");
    assert_eq!(body["status"], 503);
}
