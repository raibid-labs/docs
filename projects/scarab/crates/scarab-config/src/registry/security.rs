//! Security verification for plugin downloads

use super::types::{PluginEntry, SecurityConfig};
use crate::error::{ConfigError, Result};
use sha2::{Digest, Sha256};

/// Plugin security verifier
pub struct PluginVerifier {
    config: SecurityConfig,
}

impl PluginVerifier {
    /// Create new verifier with configuration
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Verify plugin content against entry metadata
    pub fn verify(&self, content: &[u8], entry: &PluginEntry, version: &str) -> Result<()> {
        // Find version metadata
        let version_meta = entry
            .versions
            .iter()
            .find(|v| v.version == version)
            .ok_or_else(|| {
                ConfigError::ValidationError(format!("Version {} not found", version))
            })?;

        // Verify checksum if required
        if self.config.require_checksum {
            self.verify_checksum(content, &version_meta.checksum)?;
        }

        // Verify signature if required
        if self.config.require_signature {
            if let Some(signature) = &version_meta.signature {
                self.verify_signature(content, signature)?;
            } else if !self.config.allow_unsigned {
                return Err(ConfigError::SecurityError(
                    "Plugin signature required but not provided".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Verify SHA256 checksum
    fn verify_checksum(&self, content: &[u8], expected: &str) -> Result<()> {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let computed = format!("{:x}", hasher.finalize());

        if computed != expected {
            return Err(ConfigError::SecurityError(format!(
                "Checksum mismatch: expected {}, got {}",
                expected, computed
            )));
        }

        Ok(())
    }

    /// Verify GPG signature
    ///
    /// NOTE: This is a placeholder implementation. In production, this should:
    /// 1. Use a GPG library (e.g., sequoia-pgp or gpgme)
    /// 2. Verify the detached signature against the content
    /// 3. Check the signing key against trusted_keys
    /// 4. Validate key expiration and revocation
    fn verify_signature(&self, _content: &[u8], _signature: &str) -> Result<()> {
        // TODO: Implement GPG signature verification
        // For now, just check if we have trusted keys configured
        if self.config.trusted_keys.is_empty() {
            tracing::warn!("GPG signature verification requested but no trusted keys configured");
        }

        // Placeholder: accept all signatures for now
        if !self.config.allow_unsigned {
            tracing::warn!("GPG signature verification not yet implemented, skipping");
        }

        Ok(())
    }

    /// Compute SHA256 checksum of content
    pub fn compute_checksum(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Validate plugin file format
    pub fn validate_plugin_format(content: &[u8], filename: &str) -> Result<PluginFormat> {
        // Check for Fusabi bytecode magic number (.fzb)
        if content.starts_with(b"FZB\x00") {
            return Ok(PluginFormat::FusabiBytecode);
        }

        // Check for Fusabi source by extension and content
        if filename.ends_with(".fsx") {
            // Basic validation: check if it looks like F# source
            if let Ok(text) = std::str::from_utf8(content) {
                if text.contains("module") || text.contains("let") || text.contains("type") {
                    return Ok(PluginFormat::FusabiSource);
                }
            }
        }

        Err(ConfigError::ValidationError(
            "Invalid plugin format: expected .fzb or .fsx file".to_string(),
        ))
    }
}

/// Plugin file format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginFormat {
    /// Compiled Fusabi bytecode (.fzb)
    FusabiBytecode,
    /// Fusabi source code (.fsx)
    FusabiSource,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::types::{PluginStats, PluginVersion};

    #[test]
    fn test_compute_checksum() {
        let content = b"hello world";
        let checksum = PluginVerifier::compute_checksum(content);
        // Known SHA256 of "hello world"
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn test_verify_checksum_success() {
        let config = SecurityConfig {
            require_checksum: true,
            require_signature: false,
            trusted_keys: vec![],
            allow_unsigned: true,
        };

        let verifier = PluginVerifier::new(config);
        let content = b"test content";
        let checksum = PluginVerifier::compute_checksum(content);

        let entry = create_test_entry(&checksum);
        assert!(verifier.verify(content, &entry, "1.0.0").is_ok());
    }

    #[test]
    fn test_verify_checksum_failure() {
        let config = SecurityConfig {
            require_checksum: true,
            require_signature: false,
            trusted_keys: vec![],
            allow_unsigned: true,
        };

        let verifier = PluginVerifier::new(config);
        let content = b"test content";
        let wrong_checksum = "0000000000000000000000000000000000000000000000000000000000000000";

        let entry = create_test_entry(wrong_checksum);
        assert!(verifier.verify(content, &entry, "1.0.0").is_err());
    }

    #[test]
    fn test_validate_plugin_format() {
        // Test .fsx source format
        let fsx_content = b"module MyPlugin\nlet foo = 42";
        let result = PluginVerifier::validate_plugin_format(fsx_content, "test.fsx");
        assert!(matches!(result, Ok(PluginFormat::FusabiSource)));

        // Test invalid format
        let invalid_content = b"random content";
        let result = PluginVerifier::validate_plugin_format(invalid_content, "test.txt");
        assert!(result.is_err());
    }

    fn create_test_entry(checksum: &str) -> PluginEntry {
        PluginEntry {
            name: "test-plugin".to_string(),
            description: "Test".to_string(),
            readme: None,
            author: "Test".to_string(),
            author_email: None,
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            latest_version: "1.0.0".to_string(),
            versions: vec![PluginVersion {
                version: "1.0.0".to_string(),
                download_url: "https://example.com/plugin.fzb".to_string(),
                checksum: checksum.to_string(),
                signature: None,
                changelog: None,
                api_version: "0.1.0".to_string(),
                min_scarab_version: "0.1.0".to_string(),
                size: 1024,
                released_at: 0,
                prerelease: false,
            }],
            tags: vec![],
            stats: PluginStats::default(),
            created_at: 0,
            updated_at: 0,
        }
    }
}
