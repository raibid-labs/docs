//! Reference resolution for `auth://` URIs and vals-style references.
//!
//! This module provides:
//! - [`ResolvedValue`] - The result of resolving a reference
//! - [`ReferenceResolver`] - Trait for resolving credential references
//! - Support for `auth://service/account/credential` URIs
//! - Optional support for `vals:ref+...` external references

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::model::CredentialRef;
use crate::store::Secret;

/// Error type for reference resolution.
#[derive(Debug, Error)]
pub enum ResolveError {
    /// The reference format is invalid.
    #[error("invalid reference format: {message}")]
    InvalidFormat { message: String },

    /// The referenced credential was not found.
    #[error("credential not found: {reference}")]
    NotFound { reference: String },

    /// The reference scheme is not supported.
    #[error("unsupported scheme: {scheme}")]
    UnsupportedScheme { scheme: String },

    /// Error from the underlying store.
    #[error("storage error: {0}")]
    StorageError(#[from] crate::store::StoreError),

    /// Error from token operations.
    #[error("token error: {0}")]
    TokenError(#[from] crate::token::TokenError),

    /// Error calling external resolver (e.g., vals).
    #[error("external resolver error: {message}")]
    ExternalError { message: String },
}

/// The result of resolving a reference.
#[derive(Debug, Clone)]
pub enum ResolvedValue {
    /// A secret value (token, API key, etc.).
    Secret(Secret),

    /// A non-secret string value.
    Plain(String),

    /// Multiple resolved values (for batch resolution).
    Multiple(Vec<(String, ResolvedValue)>),
}

impl ResolvedValue {
    /// Get the value as a string, exposing secrets.
    ///
    /// Use sparingly; prefer keeping values wrapped.
    pub fn expose(&self) -> String {
        match self {
            Self::Secret(s) => s.expose().to_string(),
            Self::Plain(s) => s.clone(),
            Self::Multiple(_) => "[multiple values]".to_string(),
        }
    }

    /// Check if this is a secret value.
    pub fn is_secret(&self) -> bool {
        matches!(self, Self::Secret(_))
    }
}

/// Trait for resolving credential references.
///
/// This is the primary interface for consumers to obtain credentials.
/// It handles:
/// - `auth://` URIs for Sigilforge-managed credentials
/// - Token refresh when needed
/// - Optional external reference resolution (vals-style)
///
/// # Example
///
/// ```rust,ignore
/// use sigilforge_core::ReferenceResolver;
///
/// async fn get_spotify_token(resolver: &impl ReferenceResolver) -> String {
///     let value = resolver.resolve("auth://spotify/personal/token").await.unwrap();
///     value.expose()
/// }
/// ```
#[async_trait]
pub trait ReferenceResolver: Send + Sync {
    /// Resolve a reference string to its value.
    ///
    /// Supported formats:
    /// - `auth://service/account/credential_type` - Sigilforge credential
    /// - `vals:ref+vault://path` - External vals reference (if enabled)
    async fn resolve(&self, reference: &str) -> Result<ResolvedValue, ResolveError>;

    /// Resolve a structured credential reference.
    ///
    /// This is more type-safe than string-based resolution.
    async fn resolve_ref(&self, cred_ref: &CredentialRef) -> Result<ResolvedValue, ResolveError>;

    /// Resolve multiple references in batch.
    ///
    /// More efficient than resolving one at a time when many references
    /// need to be resolved together.
    async fn resolve_batch(
        &self,
        references: &[String],
    ) -> Result<Vec<(String, Result<ResolvedValue, ResolveError>)>, ResolveError> {
        let mut results = Vec::with_capacity(references.len());
        for reference in references {
            let result = self.resolve(reference).await;
            results.push((reference.clone(), result));
        }
        Ok(results)
    }

    /// Check if a reference scheme is supported.
    fn supports_scheme(&self, scheme: &str) -> bool;
}

/// Configuration for reference resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolverConfig {
    /// Enable `auth://` URI resolution (default: true).
    pub enable_auth_scheme: bool,

    /// Enable vals-style reference resolution.
    pub enable_vals: bool,

    /// Path to vals binary (if not in PATH).
    pub vals_path: Option<String>,

    /// Cache resolved values for this duration (seconds).
    pub cache_ttl_secs: Option<u64>,
}

impl Default for ResolverConfig {
    fn default() -> Self {
        Self {
            enable_auth_scheme: true,
            enable_vals: false,
            vals_path: None,
            cache_ttl_secs: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolved_value_expose() {
        let secret = ResolvedValue::Secret(Secret::new("secret-token"));
        assert_eq!(secret.expose(), "secret-token");

        let plain = ResolvedValue::Plain("plain-value".to_string());
        assert_eq!(plain.expose(), "plain-value");
    }

    #[test]
    fn test_resolved_value_is_secret() {
        let secret = ResolvedValue::Secret(Secret::new("token"));
        assert!(secret.is_secret());

        let plain = ResolvedValue::Plain("value".to_string());
        assert!(!plain.is_secret());
    }
}
