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

    pub async fn remove_model(&self, name: &str) -> Result<(), GateError> {
        let url = format!(
            "{}/api/v1/models/{}",
            self.base_url,
            urlencoding_path(name)
        );
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
