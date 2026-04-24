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

use std::time::Duration;

use serde::{Deserialize, Serialize};

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
