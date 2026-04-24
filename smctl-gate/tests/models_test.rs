//! Integration tests for the model management surface.
//!
//! Covers list / add / remove against a wiremock ModelGate, plus the
//! ModelNotFound 404 path. Upload path is exercised end-to-end including
//! the streaming progress callback.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::json;
use smctl_gate::{GateClient, GateConfig, GateError};
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(url: String) -> GateClient {
    GateClient::new(GateConfig {
        url,
        timeout_secs: 10,
    })
    .expect("client builds")
}

#[tokio::test]
async fn list_models_returns_parsed_entries() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {
                "name": "llama-7b",
                "format": "gguf",
                "size_bytes": 4_000_000_000u64,
                "registered_at": "2026-03-01",
                "status": "loaded",
            },
            {
                "name": "whisper-base",
                "format": "onnx",
                "size_bytes": 142_000_000u64,
                "registered_at": "2026-02-28",
                "status": "loaded",
            }
        ])))
        .mount(&server)
        .await;

    let models = client(server.uri()).list_models().await.unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(models[0].name, "llama-7b");
    assert_eq!(models[1].format, "onnx");
}

#[tokio::test]
async fn list_models_empty_array() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let models = client(server.uri()).list_models().await.unwrap();
    assert!(models.is_empty());
}

#[tokio::test]
async fn add_model_streams_file_and_reports_progress() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "name": "phi-2",
            "format": "onnx",
            "size_bytes": 1024,
            "registered_at": "2026-04-24",
            "status": "loaded",
        })))
        .mount(&server)
        .await;

    // Write a 1 KiB test file so the streaming + progress path has something
    // non-trivial to push through reqwest.
    let tmp = tempfile_like();
    std::fs::write(&tmp, vec![0u8; 1024]).unwrap();

    let observed = Arc::new(AtomicU64::new(0));
    let observed_for_cb = observed.clone();
    let progress: smctl_gate::ProgressCallback = Arc::new(move |sent, _total| {
        observed_for_cb.store(sent, Ordering::SeqCst);
    });

    let model = client(server.uri())
        .add_model(&tmp, Some(progress))
        .await
        .unwrap();

    assert_eq!(model.name, "phi-2");
    assert_eq!(observed.load(Ordering::SeqCst), 1024);

    let _ = std::fs::remove_file(&tmp);
}

#[tokio::test]
async fn add_model_errors_on_missing_file() {
    let server = MockServer::start().await;
    // No mock: if this request ever fires, wiremock will 404 it, but we
    // expect add_model to fail before the HTTP round-trip.
    let err = client(server.uri())
        .add_model(std::path::Path::new("/does/not/exist.onnx"), None)
        .await
        .unwrap_err();
    assert!(matches!(err, GateError::FileNotFound { .. }), "got {err:?}");
}

#[tokio::test]
async fn remove_model_succeeds_on_204() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/models/phi-2"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    client(server.uri()).remove_model("phi-2").await.unwrap();
}

#[tokio::test]
async fn remove_model_maps_404_to_model_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path_regex(r"^/api/v1/models/.*$"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let err = client(server.uri())
        .remove_model("ghost")
        .await
        .unwrap_err();
    match err {
        GateError::ModelNotFound { name } => assert_eq!(name, "ghost"),
        other => panic!("expected ModelNotFound, got {other:?}"),
    }
}

/// Hand-rolled tempfile path — we deliberately avoid `tempfile` here to
/// keep the dev-deps set minimal. The test cleans up explicitly.
fn tempfile_like() -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("smctl-gate-add-{nanos}.bin"))
}
