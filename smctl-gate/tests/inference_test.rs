//! Integration tests for the inference surface.

use std::path::PathBuf;

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

fn write_tmp_json(payload: &serde_json::Value) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let p = std::env::temp_dir().join(format!("smctl-gate-inference-{nanos}.json"));
    std::fs::write(&p, serde_json::to_vec(payload).unwrap()).unwrap();
    p
}

#[tokio::test]
async fn test_inference_forwards_payload_and_parses_result() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/inference/llama-7b"))
        .and(body_json(json!({"prompt": "hello"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "model": "llama-7b",
            "output": {"text": "world"},
            "latency_ms": 142,
            "tokens_generated": 87,
        })))
        .mount(&server)
        .await;

    let input = write_tmp_json(&json!({"prompt": "hello"}));
    let result = client(server.uri())
        .test_inference("llama-7b", &input)
        .await
        .unwrap();

    assert_eq!(result.model, "llama-7b");
    assert_eq!(result.latency_ms, 142);
    assert_eq!(result.tokens_generated, Some(87));
    assert_eq!(result.output, json!({"text": "world"}));

    let _ = std::fs::remove_file(&input);
}

#[tokio::test]
async fn test_inference_maps_404_to_model_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/inference/ghost"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let input = write_tmp_json(&json!({"prompt": "hello"}));
    let err = client(server.uri())
        .test_inference("ghost", &input)
        .await
        .unwrap_err();

    match err {
        GateError::ModelNotFound { name } => assert_eq!(name, "ghost"),
        other => panic!("expected ModelNotFound, got {other:?}"),
    }

    let _ = std::fs::remove_file(&input);
}

#[tokio::test]
async fn test_inference_errors_on_missing_input_file() {
    let server = MockServer::start().await;
    let err = client(server.uri())
        .test_inference("any", std::path::Path::new("/does/not/exist.json"))
        .await
        .unwrap_err();
    assert!(matches!(err, GateError::FileNotFound { .. }), "got {err:?}");
}

#[tokio::test]
async fn test_inference_errors_on_non_json_input() {
    // Input file exists but isn't valid JSON.
    let p = {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let p = std::env::temp_dir().join(format!("smctl-gate-bad-{nanos}.json"));
        std::fs::write(&p, b"not json at all").unwrap();
        p
    };

    let server = MockServer::start().await;
    let err = client(server.uri())
        .test_inference("any", &p)
        .await
        .unwrap_err();
    assert!(matches!(err, GateError::ParseError { .. }), "got {err:?}");

    let _ = std::fs::remove_file(&p);
}
