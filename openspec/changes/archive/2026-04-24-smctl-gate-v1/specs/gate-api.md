# smctl ModelGate Control — API Specification

## Overview

The `smctl-gate` crate provides a CLI client for ModelGate's REST API. This spec defines the client-side interface: the `GateClient` struct, API contract, CLI subcommands, and error handling.

## GateClient

```rust
pub struct GateClient {
    base_url: String,
    client: reqwest::Client,
    timeout: Duration,
}

impl GateClient {
    /// Create a new client from configuration.
    /// Resolution order: explicit url > MODELGATE_URL env > workspace.toml > localhost:8080
    pub fn new(config: GateConfig) -> Result<Self, GateError>;

    // Health
    pub async fn health(&self) -> Result<HealthStatus, GateError>;

    // Models
    pub async fn list_models(&self) -> Result<Vec<Model>, GateError>;
    pub async fn add_model(&self, path: &Path) -> Result<Model, GateError>;
    pub async fn remove_model(&self, name: &str) -> Result<(), GateError>;

    // Routes
    pub async fn list_routes(&self) -> Result<Vec<Route>, GateError>;
    pub async fn set_route(&self, model: &str, endpoint: &str) -> Result<Route, GateError>;

    // Inference
    pub async fn test_inference(&self, model: &str, input: &Path) -> Result<InferenceResult, GateError>;

    // Logs
    pub async fn stream_logs(&self) -> Result<impl Stream<Item = LogEntry>, GateError>;
}
```

## Data Types

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,          // "healthy", "degraded", "unhealthy"
    pub version: String,
    pub uptime_secs: u64,
    pub model_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub format: String,          // "onnx", "gguf", etc.
    pub size_bytes: u64,
    pub registered_at: String,   // ISO 8601
    pub status: String,          // "loaded", "unloaded", "error"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Route {
    pub model: String,
    pub endpoint: String,
    pub active: bool,
    pub request_count: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferenceResult {
    pub model: String,
    pub output: serde_json::Value,
    pub latency_ms: u64,
    pub tokens_generated: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    pub fields: serde_json::Value,
}
```

## Configuration

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct GateConfig {
    pub url: String,
    pub timeout_secs: u64,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8080".into(),
            timeout_secs: 30,
        }
    }
}
```

### workspace.toml Extension

```toml
[gate]
url = "http://localhost:8080"
timeout_secs = 30
```

## CLI Subcommands

### `smctl gate status`

```
$ smctl gate status
ModelGate Status:
  URL:      http://localhost:8080
  Status:   healthy
  Version:  0.2.0
  Uptime:   2h 14m
  Models:   3 loaded

$ smctl gate status --json
{"status":"healthy","version":"0.2.0","uptime_secs":8040,"model_count":3}
```

### `smctl gate models list`

```
$ smctl gate models list
NAME              FORMAT   SIZE      STATUS    REGISTERED
llama-7b          gguf     3.8 GB    loaded    2026-03-01
whisper-base      onnx     142 MB    loaded    2026-02-28
clip-vit          onnx     338 MB    unloaded  2026-02-25
```

### `smctl gate models add <path>`

```
$ smctl gate models add ./models/phi-2.onnx
Uploading phi-2.onnx... [████████████████████████] 100% (1.2 GB)
Model 'phi-2' registered successfully.

$ smctl gate models add ./models/phi-2.onnx --dry-run
Would upload: ./models/phi-2.onnx (1.2 GB)
Would register as: phi-2 (format: onnx)
```

### `smctl gate models remove <name>`

```
$ smctl gate models remove clip-vit
Model 'clip-vit' removed.
```

### `smctl gate routes list`

```
$ smctl gate routes list
MODEL             ENDPOINT              ACTIVE   REQUESTS
llama-7b          /v1/chat/completions  yes      1,234
whisper-base      /v1/audio/transcribe  yes      567
```

### `smctl gate routes set <model> <endpoint>`

```
$ smctl gate routes set phi-2 /v1/chat/completions
Route set: phi-2 → /v1/chat/completions
```

### `smctl gate test <model> --input <file>`

```
$ smctl gate test llama-7b --input test-prompt.json
Inference result:
  Model:    llama-7b
  Latency:  142ms
  Tokens:   87
  Output:   {"text": "The SmallAIOS kernel is a unikernel designed for..."}
```

### `smctl gate logs`

```
$ smctl gate logs --follow
2026-03-08T10:14:22Z INFO  Request received: POST /v1/chat/completions model=llama-7b
2026-03-08T10:14:22Z INFO  Inference complete: model=llama-7b latency=142ms tokens=87
^C
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum GateError {
    #[error("Connection refused: {url} — is ModelGate running?")]
    ConnectionRefused { url: String },

    #[error("HTTP {status}: {body}")]
    HttpError { status: u16, body: String },

    #[error("Request timeout after {timeout_secs}s")]
    Timeout { timeout_secs: u64 },

    #[error("Failed to parse response: {source}")]
    ParseError { source: serde_json::Error },

    #[error("Model not found: {name}")]
    ModelNotFound { name: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },
}
```

All errors produce user-friendly messages. Connection errors suggest checking if ModelGate is running. HTTP 404s translate to domain-specific "not found" messages.

## Testing

Integration tests use `wiremock` to mock the ModelGate API:

```rust
#[tokio::test]
async fn test_list_models() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([
            {"name": "test-model", "format": "onnx", "size_bytes": 1000, "status": "loaded"}
        ])))
        .mount(&mock_server)
        .await;

    let client = GateClient::new(GateConfig {
        url: mock_server.uri(),
        timeout_secs: 5,
    }).unwrap();

    let models = client.list_models().await.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].name, "test-model");
}
```
