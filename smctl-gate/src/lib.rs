use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// ModelGate connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateConfig {
    pub base_url: String,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
        }
    }
}

/// Health status of a ModelGate instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateStatus {
    pub healthy: bool,
    pub url: String,
    pub version: Option<String>,
    pub models_loaded: Option<usize>,
}

/// Registered model info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: String,
    pub format: String,
    pub loaded: bool,
    pub hash: Option<String>,
}

/// Routing table entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub model: String,
    pub endpoint: String,
    pub active: bool,
}

/// Inference test result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceResult {
    pub model: String,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub latency_ms: u64,
    pub error: Option<String>,
}

/// Security policy info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyInfo {
    pub mode: String,
    pub labels_count: usize,
    pub boundaries_count: usize,
    pub whitelist_count: usize,
}

/// Trust boundary info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryInfo {
    pub name: String,
    pub classification: String,
    pub integrity: String,
    pub has_cedar_rules: bool,
    pub has_formal_proofs: bool,
}

/// ModelGate API client.
#[derive(Debug, Clone)]
pub struct GateClient {
    pub config: GateConfig,
    client: reqwest::Client,
}

impl GateClient {
    pub fn new(config: GateConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Check gate health.
    pub async fn status(&self) -> Result<GateStatus> {
        let url = format!("{}/health", self.config.base_url);
        match self.client.get(&url).send().await {
            Ok(resp) if resp.status().is_success() => {
                let body: serde_json::Value = resp.json().await.unwrap_or_default();
                Ok(GateStatus {
                    healthy: true,
                    url: self.config.base_url.clone(),
                    version: body
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    models_loaded: body
                        .get("models_loaded")
                        .and_then(|v| v.as_u64())
                        .map(|n| n as usize),
                })
            }
            Ok(_resp) => Ok(GateStatus {
                healthy: false,
                url: self.config.base_url.clone(),
                version: None,
                models_loaded: None,
            }),
            Err(_e) => Ok(GateStatus {
                healthy: false,
                url: self.config.base_url.clone(),
                version: None,
                models_loaded: None,
            }),
        }
    }

    /// List registered models.
    pub async fn models_list(&self) -> Result<Vec<ModelInfo>> {
        let url = format!("{}/models", self.config.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("failed to connect to ModelGate")?;

        let models: Vec<ModelInfo> = resp.json().await.context("invalid response")?;
        Ok(models)
    }

    /// Register a model.
    pub async fn models_add(&self, path: &str) -> Result<ModelInfo> {
        let url = format!("{}/models", self.config.base_url);
        let body = serde_json::json!({ "path": path });
        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("failed to register model")?;

        let model: ModelInfo = resp.json().await.context("invalid response")?;
        Ok(model)
    }

    /// Remove a model.
    pub async fn models_remove(&self, name: &str) -> Result<()> {
        let url = format!("{}/models/{}", self.config.base_url, name);
        self.client
            .delete(&url)
            .send()
            .await
            .context("failed to remove model")?;
        Ok(())
    }

    /// List routes.
    pub async fn routes_list(&self) -> Result<Vec<RouteEntry>> {
        let url = format!("{}/routes", self.config.base_url);
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .context("failed to list routes")?;
        let routes: Vec<RouteEntry> = resp.json().await.context("invalid response")?;
        Ok(routes)
    }

    /// Set a route.
    pub async fn routes_set(&self, model: &str, endpoint: &str) -> Result<()> {
        let url = format!("{}/routes", self.config.base_url);
        let body = serde_json::json!({ "model": model, "endpoint": endpoint });
        self.client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("failed to set route")?;
        Ok(())
    }

    /// Run test inference.
    pub async fn test_inference(
        &self,
        model: &str,
        input: &serde_json::Value,
    ) -> Result<InferenceResult> {
        let url = format!("{}/inference/{}", self.config.base_url, model);
        let start = std::time::Instant::now();
        let resp = self
            .client
            .post(&url)
            .json(input)
            .send()
            .await
            .context("failed to run inference")?;

        let latency = start.elapsed().as_millis() as u64;

        if resp.status().is_success() {
            let output: serde_json::Value = resp.json().await.unwrap_or_default();
            Ok(InferenceResult {
                model: model.to_string(),
                success: true,
                output: Some(output),
                latency_ms: latency,
                error: None,
            })
        } else {
            let error = resp.text().await.unwrap_or_default();
            Ok(InferenceResult {
                model: model.to_string(),
                success: false,
                output: None,
                latency_ms: latency,
                error: Some(error),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_gate_config() {
        let config = GateConfig::default();
        assert_eq!(config.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_gate_client_creation() {
        let client = GateClient::new(GateConfig::default());
        assert_eq!(client.config.base_url, "http://localhost:8080");
    }
}
