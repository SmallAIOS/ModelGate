//! smctl-gate — ModelGate control-plane client.
//!
//! Provides the machinery behind `smctl gate <verb>`. Each verb maps to a
//! REST call against a running ModelGate instance. The client resolves its
//! endpoint from (in priority order): the `--gate-url` CLI flag, the
//! `MODELGATE_URL` environment variable, the `[gate]` section of
//! `workspace.toml`, and finally the `http://localhost:8080` default.
//!
//! This crate is a log producer only: it emits tracing events via the
//! `smctl-log` MSGID catalog but never installs its own subscriber. The
//! `smctl` binary (or any embedder) owns the subscriber.
//!
//! Current scope: the `status` verb (GET /health) is implemented end-to-end.
//! Remaining verbs (models, routes, test, logs) are scaffolded in tasks.md
//! and land in follow-up commits on this branch.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio_util::io::ReaderStream;

pub const DEFAULT_URL: &str = "http://localhost:8080";
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateConfig {
    pub url: String,
    pub timeout_secs: u64,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_URL.into(),
            timeout_secs: DEFAULT_TIMEOUT_SECS,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GateError {
    #[error("connection refused: {url} — is ModelGate running?")]
    ConnectionRefused { url: String },

    #[error("request timeout after {timeout_secs}s")]
    Timeout { timeout_secs: u64 },

    #[error("HTTP {status}: {body}")]
    HttpError { status: u16, body: String },

    #[error("failed to parse response: {source}")]
    ParseError {
        #[source]
        source: serde_json::Error,
    },

    #[error("transport error: {source}")]
    Transport {
        #[source]
        source: reqwest::Error,
    },

    #[error("invalid endpoint URL: {url}")]
    InvalidUrl { url: String },

    #[error("model not found: {name}")]
    ModelNotFound { name: String },

    #[error("file not found: {path}")]
    FileNotFound { path: String },

    #[error("I/O error reading {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

impl GateError {
    fn from_reqwest(err: reqwest::Error, url: &str, timeout_secs: u64) -> Self {
        if err.is_timeout() {
            return GateError::Timeout { timeout_secs };
        }
        if err.is_connect() {
            return GateError::ConnectionRefused { url: url.into() };
        }
        GateError::Transport { source: err }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime_secs: u64,
    pub model_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Route {
    pub model: String,
    pub endpoint: String,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub request_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
    #[serde(default)]
    pub fields: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub model: String,
    pub output: serde_json::Value,
    pub latency_ms: u64,
    #[serde(default)]
    pub tokens_generated: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Model {
    pub name: String,
    pub format: String,
    pub size_bytes: u64,
    #[serde(default)]
    pub registered_at: String,
    pub status: String,
}

/// Progress callback for streaming uploads.
///
/// Called after each chunk with `(bytes_sent_so_far, total_bytes)`.
/// `total_bytes` is `None` when the source has no known length (rare;
/// file uploads always report a length).
pub type ProgressCallback = Arc<dyn Fn(u64, Option<u64>) + Send + Sync>;

#[derive(Debug, Clone)]
pub struct GateClient {
    base_url: String,
    client: reqwest::Client,
    timeout_secs: u64,
}

impl GateClient {
    pub fn new(config: GateConfig) -> Result<Self, GateError> {
        if config.url.is_empty() {
            return Err(GateError::InvalidUrl { url: config.url });
        }
        reqwest::Url::parse(&config.url).map_err(|_| GateError::InvalidUrl {
            url: config.url.clone(),
        })?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|source| GateError::Transport { source })?;

        Ok(Self {
            base_url: config.url.trim_end_matches('/').to_string(),
            client,
            timeout_secs: config.timeout_secs,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn health(&self) -> Result<HealthStatus, GateError> {
        let url = format!("{}/health", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| GateError::from_reqwest(e, &self.base_url, self.timeout_secs))?;
        parse_json(resp).await
    }

    pub async fn list_models(&self) -> Result<Vec<Model>, GateError> {
        let url = format!("{}/api/v1/models", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| GateError::from_reqwest(e, &self.base_url, self.timeout_secs))?;
        parse_json(resp).await
    }

    /// Register a model by streaming the file at `path` to the gateway.
    ///
    /// The model name is derived from the file stem. `progress`, if set,
    /// is invoked after each chunk with `(bytes_sent, total_bytes)`.
    pub async fn add_model(
        &self,
        path: &Path,
        progress: Option<ProgressCallback>,
    ) -> Result<Model, GateError> {
        let file = tokio::fs::File::open(path).await.map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                GateError::FileNotFound {
                    path: path.display().to_string(),
                }
            } else {
                GateError::Io {
                    path: path.display().to_string(),
                    source,
                }
            }
        })?;
        let total = file.metadata().await.ok().map(|m| m.len());

        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| GateError::FileNotFound {
                path: path.display().to_string(),
            })?
            .to_string();
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(&name)
            .to_string();

        let stream = ReaderStream::new(file);
        let mut sent: u64 = 0;
        let tracked = stream.map(move |chunk| {
            if let Ok(ref bytes) = chunk {
                sent += bytes.len() as u64;
                if let Some(cb) = &progress {
                    cb(sent, total);
                }
            }
            chunk
        });

        let body = reqwest::Body::wrap_stream(tracked);
        let part = match total {
            Some(t) => reqwest::multipart::Part::stream_with_length(body, t),
            None => reqwest::multipart::Part::stream(body),
        }
        .file_name(filename);

        let form = reqwest::multipart::Form::new()
            .text("name", name.clone())
            .part("file", part);

        let url = format!("{}/api/v1/models", self.base_url);
        let resp = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| GateError::from_reqwest(e, &self.base_url, self.timeout_secs))?;
        parse_json(resp).await
    }

    pub async fn list_routes(&self) -> Result<Vec<Route>, GateError> {
        let url = format!("{}/api/v1/routes", self.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| GateError::from_reqwest(e, &self.base_url, self.timeout_secs))?;
        parse_json(resp).await
    }

    pub async fn set_route(&self, model: &str, endpoint: &str) -> Result<Route, GateError> {
        let url = format!("{}/api/v1/routes", self.base_url);
        let body = serde_json::json!({
            "model": model,
            "endpoint": endpoint,
        });
        let resp = self
            .client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| GateError::from_reqwest(e, &self.base_url, self.timeout_secs))?;

        // A 404 on PUT /api/v1/routes means the target model is unknown.
        let status = resp.status();
        if status.as_u16() == 404 {
            return Err(GateError::ModelNotFound {
                name: model.to_string(),
            });
        }
        parse_json(resp).await
    }

    /// Open an SSE connection to the log stream at
    /// `GET /api/v1/logs`. The returned stream yields `LogEntry` values
    /// parsed from SSE `data:` events. Caller is responsible for
    /// dropping the stream to close the connection.
    pub async fn stream_logs(
        &self,
    ) -> Result<
        std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<LogEntry, GateError>> + Send>>,
        GateError,
    > {
        let url = format!("{}/api/v1/logs", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "text/event-stream")
            .send()
            .await
            .map_err(|e| GateError::from_reqwest(e, &self.base_url, self.timeout_secs))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(GateError::HttpError {
                status: status.as_u16(),
                body,
            });
        }

        let byte_stream = resp
            .bytes_stream()
            .map(|r| r.map_err(|source| GateError::Transport { source }));

        Ok(Box::pin(sse_log_stream(byte_stream)))
    }

    /// Run a test inference against `model` with a JSON payload loaded
    /// from `input`.
    pub async fn test_inference(
        &self,
        model: &str,
        input: &Path,
    ) -> Result<InferenceResult, GateError> {
        let bytes = tokio::fs::read(input).await.map_err(|source| {
            if source.kind() == std::io::ErrorKind::NotFound {
                GateError::FileNotFound {
                    path: input.display().to_string(),
                }
            } else {
                GateError::Io {
                    path: input.display().to_string(),
                    source,
                }
            }
        })?;

        let payload: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|source| GateError::ParseError { source })?;

        let url = format!(
            "{}/api/v1/inference/{}",
            self.base_url,
            urlencoding_path(model)
        );
        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| GateError::from_reqwest(e, &self.base_url, self.timeout_secs))?;

        let status = resp.status();
        if status.as_u16() == 404 {
            return Err(GateError::ModelNotFound {
                name: model.to_string(),
            });
        }
        parse_json(resp).await
    }

    pub async fn remove_model(&self, name: &str) -> Result<(), GateError> {
        let url = format!("{}/api/v1/models/{}", self.base_url, urlencoding_path(name));
        let resp = self
            .client
            .delete(&url)
            .send()
            .await
            .map_err(|e| GateError::from_reqwest(e, &self.base_url, self.timeout_secs))?;

        let status = resp.status();
        if status.is_success() {
            return Ok(());
        }
        if status.as_u16() == 404 {
            return Err(GateError::ModelNotFound {
                name: name.to_string(),
            });
        }
        let body = resp.text().await.unwrap_or_default();
        Err(GateError::HttpError {
            status: status.as_u16(),
            body,
        })
    }
}

async fn parse_json<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, GateError> {
    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        return Err(GateError::HttpError {
            status: status.as_u16(),
            body,
        });
    }
    let bytes = resp
        .bytes()
        .await
        .map_err(|source| GateError::Transport { source })?;
    serde_json::from_slice(&bytes).map_err(|source| GateError::ParseError { source })
}

/// Minimal path-segment percent-encoding — enough for model names that may
/// contain `/`, spaces, or other URL-reserved characters. We deliberately
/// avoid pulling in `urlencoding` or `percent-encoding` for the few
/// characters that matter in a model name.
fn urlencoding_path(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for b in segment.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Parse an SSE byte stream into `LogEntry` values.
///
/// Implements the minimum of the SSE grammar relevant to this use case:
/// lines are separated by `\n` (with optional `\r`), events are
/// terminated by a blank line, and `data:` field values are concatenated
/// with `\n` between them. All other SSE fields (`event:`, `id:`,
/// `retry:`, comments starting with `:`) are ignored.
fn sse_log_stream<S>(
    byte_stream: S,
) -> impl futures_util::Stream<Item = Result<LogEntry, GateError>> + Send
where
    S: futures_util::Stream<Item = Result<bytes::Bytes, GateError>> + Send + 'static,
{
    async_stream::stream! {
        let mut byte_stream = Box::pin(byte_stream);
        let mut buf: Vec<u8> = Vec::new();
        let mut data_lines: Vec<String> = Vec::new();

        while let Some(chunk) = byte_stream.next().await {
            let chunk = match chunk {
                Ok(b) => b,
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };
            buf.extend_from_slice(&chunk);

            // Drain complete lines from the buffer.
            while let Some(nl) = buf.iter().position(|b| *b == b'\n') {
                let line_bytes: Vec<u8> = buf.drain(..=nl).collect();
                let line_str = String::from_utf8_lossy(&line_bytes[..line_bytes.len() - 1]);
                let line = line_str.trim_end_matches('\r');

                if line.is_empty() {
                    if !data_lines.is_empty() {
                        let combined = data_lines.join("\n");
                        data_lines.clear();
                        match serde_json::from_str::<LogEntry>(&combined) {
                            Ok(entry) => yield Ok(entry),
                            Err(source) => yield Err(GateError::ParseError { source }),
                        }
                    }
                } else if let Some(rest) = line.strip_prefix("data:") {
                    data_lines.push(rest.strip_prefix(' ').unwrap_or(rest).to_string());
                }
                // other SSE fields are ignored
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_uses_localhost() {
        let cfg = GateConfig::default();
        assert_eq!(cfg.url, "http://localhost:8080");
        assert_eq!(cfg.timeout_secs, 30);
    }

    #[test]
    fn new_rejects_empty_url() {
        let err = GateClient::new(GateConfig {
            url: String::new(),
            timeout_secs: 5,
        })
        .unwrap_err();
        assert!(matches!(err, GateError::InvalidUrl { .. }));
    }

    #[test]
    fn new_rejects_malformed_url() {
        let err = GateClient::new(GateConfig {
            url: "not a url".into(),
            timeout_secs: 5,
        })
        .unwrap_err();
        assert!(matches!(err, GateError::InvalidUrl { .. }));
    }

    #[test]
    fn new_trims_trailing_slash() {
        let c = GateClient::new(GateConfig {
            url: "http://example.test:9000/".into(),
            timeout_secs: 5,
        })
        .unwrap();
        assert_eq!(c.base_url(), "http://example.test:9000");
    }
}
