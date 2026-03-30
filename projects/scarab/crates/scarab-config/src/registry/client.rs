//! Registry HTTP client for remote API communication

use super::manifest::RegistryManifest;
use super::types::PluginDownload;
use crate::error::{ConfigError, Result};
use std::time::Duration;

/// HTTP client for registry API
pub struct RegistryClient {
    /// Base registry URL
    registry_url: String,
    /// HTTP client
    client: reqwest::Client,
}

impl RegistryClient {
    /// Create new registry client
    pub fn new(registry_url: String) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("scarab/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            registry_url,
            client,
        }
    }

    /// Fetch complete registry manifest
    pub async fn fetch_manifest(&self) -> Result<RegistryManifest> {
        let url = format!("{}/v1/manifest.json", self.registry_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        if !response.status().is_success() {
            return Err(ConfigError::ValidationError(format!(
                "Failed to fetch manifest: HTTP {}",
                response.status()
            )));
        }

        let content = response
            .text()
            .await
            .map_err(|e| ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        RegistryManifest::from_json(&content)
    }

    /// Download plugin by name and version
    pub async fn download_plugin(&self, name: &str, version: &str) -> Result<PluginDownload> {
        let url = format!("{}/v1/plugins/{}/{}/download", self.registry_url, name, version);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        if !response.status().is_success() {
            return Err(ConfigError::ValidationError(format!(
                "Failed to download plugin: HTTP {}",
                response.status()
            )));
        }

        // Get checksum from header
        let checksum = response
            .headers()
            .get("X-Plugin-Checksum")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        // Get signature from header
        let signature = response
            .headers()
            .get("X-Plugin-Signature")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let content = response
            .bytes()
            .await
            .map_err(|e| ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?
            .to_vec();

        Ok(PluginDownload {
            content,
            checksum: checksum.unwrap_or_default(),
            signature,
        })
    }

    /// Check if registry is reachable
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.registry_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(response.status().is_success())
    }

    /// Get registry URL
    pub fn registry_url(&self) -> &str {
        &self.registry_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = RegistryClient::new("https://registry.scarab.dev".to_string());
        assert_eq!(client.registry_url(), "https://registry.scarab.dev");
    }
}
