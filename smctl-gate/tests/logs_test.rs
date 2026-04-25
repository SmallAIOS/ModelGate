//! Integration tests for the SSE log-streaming surface.

use futures_util::StreamExt;
use smctl_gate::{GateClient, GateConfig, GateError, LogEntry};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(url: String) -> GateClient {
    GateClient::new(GateConfig {
        url,
        timeout_secs: 5,
    })
    .expect("client builds")
}

fn sse_frame(json: &str) -> String {
    format!("data: {json}\n\n")
}

#[tokio::test]
async fn stream_logs_parses_sse_data_frames() {
    let server = MockServer::start().await;

    let body = [
        sse_frame(r#"{"timestamp":"2026-04-24T10:00:00Z","level":"INFO","message":"request received","fields":{}}"#),
        sse_frame(r#"{"timestamp":"2026-04-24T10:00:01Z","level":"INFO","message":"inference complete","fields":{"latency_ms":142}}"#),
    ]
    .concat();

    Mock::given(method("GET"))
        .and(path("/api/v1/logs"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let mut stream = client(server.uri()).stream_logs().await.unwrap();
    let mut got: Vec<LogEntry> = Vec::new();
    while let Some(event) = stream.next().await {
        got.push(event.unwrap());
    }

    assert_eq!(got.len(), 2);
    assert_eq!(got[0].level, "INFO");
    assert_eq!(got[0].message, "request received");
    assert_eq!(got[1].message, "inference complete");
    assert_eq!(got[1].fields["latency_ms"], 142);
}

#[tokio::test]
async fn stream_logs_tolerates_sse_comments_and_other_fields() {
    let server = MockServer::start().await;

    // Include a comment line, an event: field (both should be ignored),
    // and a data line with no leading space after `data:`.
    let body = concat!(
        ": keepalive\n",
        "event: log\n",
        "data:{\"timestamp\":\"t\",\"level\":\"WARN\",\"message\":\"hi\"}\n",
        "\n",
    );

    Mock::given(method("GET"))
        .and(path("/api/v1/logs"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body.to_string()),
        )
        .mount(&server)
        .await;

    let mut stream = client(server.uri()).stream_logs().await.unwrap();
    let entry = stream.next().await.unwrap().unwrap();
    assert_eq!(entry.level, "WARN");
    assert_eq!(entry.message, "hi");
}

#[tokio::test]
async fn stream_logs_surfaces_http_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/logs"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;

    let err = match client(server.uri()).stream_logs().await {
        Ok(_) => panic!("expected error, got stream"),
        Err(e) => e,
    };
    match err {
        GateError::HttpError { status, body } => {
            assert_eq!(status, 500);
            assert_eq!(body, "boom");
        }
        other => panic!("expected HttpError, got {other:?}"),
    }
}
