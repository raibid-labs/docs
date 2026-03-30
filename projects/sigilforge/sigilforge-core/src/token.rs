//! Token management for OAuth and API access.
//!
//! This module provides:
//! - [`Token`] - A single access or refresh token with metadata
//! - [`TokenSet`] - A complete set of tokens for an account
//! - [`TokenInfo`] - Introspection info about a token
//! - [`TokenManager`] - Trait for token lifecycle management

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::model::{AccountId, ServiceId};
use crate::store::Secret;

/// Error type for token operations.
#[derive(Debug, Error)]
pub enum TokenError {
    /// No token is available for the requested service/account.
    #[error("no token available for {service}/{account}")]
    NotFound { service: String, account: String },

    /// The token has expired and could not be refreshed.
    #[error("token expired and refresh failed: {message}")]
    Expired { message: String },

    /// Token refresh failed.
    #[error("token refresh failed: {message}")]
    RefreshFailed { message: String },

    /// OAuth flow failed.
    #[error("OAuth flow failed: {message}")]
    OAuthError { message: String },

    /// Storage error during token operations.
    #[error("storage error: {0}")]
    StorageError(#[from] crate::store::StoreError),

    /// Network error during token refresh.
    #[error("network error: {message}")]
    NetworkError { message: String },

    /// The provider configuration is missing or invalid.
    #[error("provider not configured: {provider}")]
    ProviderNotConfigured { provider: String },
}

/// A single token with its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// The token value (access or refresh token).
    pub access_token: Secret,

    /// Token type (usually "Bearer").
    pub token_type: String,

    /// When this token expires (None if unknown or non-expiring).
    pub expires_at: Option<DateTime<Utc>>,

    /// OAuth scopes associated with this token.
    pub scopes: Vec<String>,
}

impl Token {
    /// Create a new token.
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            access_token: Secret::new(access_token),
            token_type: "Bearer".to_string(),
            expires_at: None,
            scopes: Vec::new(),
        }
    }

    /// Create a token with an expiration time.
    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Create a token with scopes.
    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Check if this token has expired.
    ///
    /// Returns `false` if no expiration is set.
    pub fn is_expired(&self) -> bool {
        self.expires_at.map(|exp| exp < Utc::now()).unwrap_or(false)
    }

    /// Check if this token will expire within the given duration.
    pub fn expires_within(&self, duration: chrono::Duration) -> bool {
        self.expires_at
            .map(|exp| exp < Utc::now() + duration)
            .unwrap_or(false)
    }
}

/// A complete set of tokens for an account.
///
/// Contains the access token and optionally a refresh token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSet {
    /// The current access token.
    pub access_token: Token,

    /// The refresh token (if available).
    pub refresh_token: Option<Secret>,

    /// When this token set was last refreshed.
    pub refreshed_at: DateTime<Utc>,
}

impl TokenSet {
    /// Create a new token set with just an access token.
    pub fn new(access_token: Token) -> Self {
        Self {
            access_token,
            refresh_token: None,
            refreshed_at: Utc::now(),
        }
    }

    /// Create a token set with both access and refresh tokens.
    pub fn with_refresh_token(mut self, refresh_token: impl Into<String>) -> Self {
        self.refresh_token = Some(Secret::new(refresh_token));
        self
    }
}

/// Information about a token obtained through introspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Whether the token is currently active/valid.
    pub active: bool,

    /// The subject (user identifier) if available.
    pub subject: Option<String>,

    /// The client ID the token was issued to.
    pub client_id: Option<String>,

    /// OAuth scopes associated with this token.
    pub scopes: Vec<String>,

    /// When this token expires.
    pub expires_at: Option<DateTime<Utc>>,
}

/// Trait for managing token lifecycle.
///
/// Implementations handle:
/// - Retrieving valid access tokens
/// - Refreshing expired tokens
/// - Running initial OAuth flows
///
/// # Example
///
/// ```rust,ignore
/// use sigilforge_core::{TokenManager, ServiceId, AccountId};
///
/// async fn call_spotify_api(manager: &impl TokenManager) -> Result<(), TokenError> {
///     let service = ServiceId::new("spotify");
///     let account = AccountId::new("personal");
///     
///     let token = manager.ensure_access_token(&service, &account).await?;
///     // Use token.access_token.expose() for API calls
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait TokenManager: Send + Sync {
    /// Get a valid access token, refreshing if necessary.
    ///
    /// This is the primary method consumers should use. It:
    /// 1. Checks for an existing access token
    /// 2. If expired, attempts to refresh using the refresh token
    /// 3. Returns an error if no valid token can be obtained
    async fn ensure_access_token(
        &self,
        service: &ServiceId,
        account: &AccountId,
    ) -> Result<Token, TokenError>;

    /// Get the current token set without refreshing.
    ///
    /// Returns `None` if no tokens are stored for this service/account.
    async fn get_token_set(
        &self,
        service: &ServiceId,
        account: &AccountId,
    ) -> Result<Option<TokenSet>, TokenError>;

    /// Store a new token set for an account.
    ///
    /// Called after successful OAuth flows or token refreshes.
    async fn store_token_set(
        &self,
        service: &ServiceId,
        account: &AccountId,
        token_set: TokenSet,
    ) -> Result<(), TokenError>;

    /// Revoke tokens for an account.
    ///
    /// This should:
    /// 1. Attempt to revoke the token with the provider (if supported)
    /// 2. Remove the tokens from storage
    async fn revoke_tokens(
        &self,
        service: &ServiceId,
        account: &AccountId,
    ) -> Result<(), TokenError>;

    /// Introspect a token to get its current status.
    ///
    /// Not all providers support token introspection.
    async fn introspect_token(
        &self,
        service: &ServiceId,
        account: &AccountId,
    ) -> Result<TokenInfo, TokenError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_is_expired() {
        let expired_token = Token::new("test")
            .with_expiry(Utc::now() - chrono::Duration::hours(1));
        assert!(expired_token.is_expired());

        let valid_token = Token::new("test")
            .with_expiry(Utc::now() + chrono::Duration::hours(1));
        assert!(!valid_token.is_expired());

        let no_expiry_token = Token::new("test");
        assert!(!no_expiry_token.is_expired());
    }

    #[test]
    fn test_token_expires_within() {
        let token = Token::new("test")
            .with_expiry(Utc::now() + chrono::Duration::minutes(5));

        assert!(token.expires_within(chrono::Duration::minutes(10)));
        assert!(!token.expires_within(chrono::Duration::minutes(2)));
    }
}
