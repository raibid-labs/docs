//! Default implementation of the TokenManager trait.
//!
//! This module provides [`DefaultTokenManager`], a complete implementation
//! of the [`TokenManager`] trait that handles token storage, refresh, and revocation.
//!
//! # Features
//!
//! - Automatic token refresh when expired
//! - Persistent storage via [`SecretStore`]
//! - Integration with OAuth provider configurations
//! - Configurable expiry buffer to refresh tokens before they expire
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "oauth")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use sigilforge_core::{
//!     MemoryStore,
//!     provider::ProviderRegistry,
//!     token_manager::DefaultTokenManager,
//!     TokenManager, ServiceId, AccountId,
//! };
//!
//! let store = MemoryStore::new();
//! let registry = ProviderRegistry::with_defaults();
//! let manager = DefaultTokenManager::new(store, registry);
//!
//! let service = ServiceId::new("github");
//! let account = AccountId::new("personal");
//!
//! let token = manager.ensure_access_token(&service, &account).await?;
//! println!("Access token: {}", token.access_token.expose());
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use chrono::{Duration, Utc};
use oauth2::{RefreshToken, TokenResponse, reqwest::async_http_client};

use crate::{
    model::{AccountId, CredentialType, ServiceId},
    provider::ProviderRegistry,
    store::{Secret, SecretStore},
    token::{Token, TokenError, TokenInfo, TokenManager, TokenSet},
};

#[cfg(feature = "oauth")]
use crate::oauth::create_oauth_client;

/// Default expiry buffer in minutes.
///
/// Tokens are considered expired if they expire within this many minutes.
/// This prevents race conditions where a token expires between fetching and using it.
const DEFAULT_EXPIRY_BUFFER_MINUTES: i64 = 5;

/// Default implementation of TokenManager.
///
/// This implementation:
/// - Stores tokens in a [`SecretStore`] backend
/// - Automatically refreshes expired tokens using refresh tokens
/// - Retrieves client credentials from the store
/// - Integrates with OAuth provider configurations
///
/// # Type Parameters
///
/// * `S` - The secret store implementation to use
pub struct DefaultTokenManager<S: SecretStore> {
    #[cfg_attr(test, doc(hidden))]
    pub store: S,
    providers: ProviderRegistry,
    http_client: reqwest::Client,
    expiry_buffer: Duration,
}

impl<S: SecretStore> DefaultTokenManager<S> {
    /// Create a new token manager with the given store and provider registry.
    ///
    /// Uses the default expiry buffer of 5 minutes.
    pub fn new(store: S, providers: ProviderRegistry) -> Self {
        Self {
            store,
            providers,
            http_client: reqwest::Client::new(),
            expiry_buffer: Duration::minutes(DEFAULT_EXPIRY_BUFFER_MINUTES),
        }
    }

    /// Create a new token manager with a custom expiry buffer.
    ///
    /// # Arguments
    ///
    /// * `store` - Secret store backend
    /// * `providers` - OAuth provider registry
    /// * `expiry_buffer_minutes` - Minutes before expiry to consider a token expired
    pub fn with_expiry_buffer(
        store: S,
        providers: ProviderRegistry,
        expiry_buffer_minutes: i64,
    ) -> Self {
        Self {
            store,
            providers,
            http_client: reqwest::Client::new(),
            expiry_buffer: Duration::minutes(expiry_buffer_minutes),
        }
    }

    /// Check if a token is expired or will expire soon.
    fn is_token_expired(&self, token: &Token) -> bool {
        token.expires_within(self.expiry_buffer)
    }

    /// Get the storage key for a credential.
    fn credential_key(
        &self,
        service: &ServiceId,
        account: &AccountId,
        cred_type: CredentialType,
    ) -> String {
        format!(
            "sigilforge/{}/{}/{}",
            service.as_str(),
            account.as_str(),
            cred_type.as_str()
        )
    }

    /// Store a secret value for a service/account/credential type.
    async fn store_credential(
        &self,
        service: &ServiceId,
        account: &AccountId,
        cred_type: CredentialType,
        value: &str,
    ) -> Result<(), TokenError> {
        let key = self.credential_key(service, account, cred_type);
        self.store.set(&key, &Secret::new(value)).await?;
        Ok(())
    }

    /// Retrieve a secret value for a service/account/credential type.
    async fn get_credential(
        &self,
        service: &ServiceId,
        account: &AccountId,
        cred_type: CredentialType,
    ) -> Result<Option<Secret>, TokenError> {
        let key = self.credential_key(service, account, cred_type);
        Ok(self.store.get(&key).await?)
    }

    /// Refresh an access token using a refresh token.
    #[cfg(feature = "oauth")]
    async fn refresh_access_token(
        &self,
        service: &ServiceId,
        account: &AccountId,
        refresh_token: &str,
    ) -> Result<TokenSet, TokenError> {
        // Get provider configuration
        let provider = self.providers.get(service.as_str()).ok_or_else(|| {
            TokenError::ProviderNotConfigured {
                provider: service.to_string(),
            }
        })?;

        // Get client credentials
        let client_id = self
            .get_credential(service, account, CredentialType::ClientId)
            .await?
            .ok_or_else(|| TokenError::RefreshFailed {
                message: format!("client ID not found for {}/{}", service, account),
            })?;

        let client_secret = self
            .get_credential(service, account, CredentialType::ClientSecret)
            .await?;

        // Create OAuth client
        let client = create_oauth_client(
            provider,
            client_id.expose(),
            client_secret.as_ref().map(|s| s.expose()),
            None::<String>,
        )?;

        // Execute refresh request
        let token_response = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| TokenError::RefreshFailed {
                message: format!("token refresh failed: {}", e),
            })?;

        // Extract token information
        let access_token = token_response.access_token().secret().to_string();
        let expires_in = token_response.expires_in();
        let scopes = token_response
            .scopes()
            .map(|s| s.iter().map(|scope| scope.to_string()).collect())
            .unwrap_or_default();

        let mut token = Token::new(access_token).with_scopes(scopes);

        // Set expiration if provided
        if let Some(duration) = expires_in {
            let expires_at = Utc::now()
                + chrono::Duration::from_std(duration).map_err(|e| TokenError::RefreshFailed {
                    message: format!("invalid expiration duration: {}", e),
                })?;
            token = token.with_expiry(expires_at);
        }

        let mut token_set = TokenSet::new(token);

        // Add refresh token (use new one if provided, otherwise keep existing)
        if let Some(new_refresh_token) = token_response.refresh_token() {
            token_set = token_set.with_refresh_token(new_refresh_token.secret());
        } else {
            token_set = token_set.with_refresh_token(refresh_token);
        }

        Ok(token_set)
    }

    /// Refresh an access token (stub for non-oauth builds).
    #[cfg(not(feature = "oauth"))]
    async fn refresh_access_token(
        &self,
        service: &ServiceId,
        account: &AccountId,
        _refresh_token: &str,
    ) -> Result<TokenSet, TokenError> {
        Err(TokenError::RefreshFailed {
            message: format!(
                "OAuth feature not enabled, cannot refresh token for {}/{}",
                service, account
            ),
        })
    }
}

#[async_trait]
impl<S: SecretStore + Send + Sync + 'static> TokenManager for DefaultTokenManager<S> {
    async fn ensure_access_token(
        &self,
        service: &ServiceId,
        account: &AccountId,
    ) -> Result<Token, TokenError> {
        // Try to get existing token set
        if let Some(token_set) = self.get_token_set(service, account).await? {
            // Check if the access token is still valid
            if !self.is_token_expired(&token_set.access_token) {
                tracing::debug!(
                    "Using cached access token for {}/{}",
                    service,
                    account
                );
                return Ok(token_set.access_token);
            }

            // Token is expired, try to refresh
            if let Some(refresh_token) = &token_set.refresh_token {
                tracing::info!(
                    "Access token expired for {}/{}, attempting refresh",
                    service,
                    account
                );

                match self
                    .refresh_access_token(service, account, refresh_token.expose())
                    .await
                {
                    Ok(new_token_set) => {
                        // Store the new token set
                        self.store_token_set(service, account, new_token_set.clone())
                            .await?;

                        tracing::info!(
                            "Successfully refreshed access token for {}/{}",
                            service,
                            account
                        );

                        return Ok(new_token_set.access_token);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to refresh token for {}/{}: {}",
                            service,
                            account,
                            e
                        );
                        return Err(TokenError::Expired {
                            message: format!("token refresh failed: {}", e),
                        });
                    }
                }
            } else {
                return Err(TokenError::Expired {
                    message: "token expired and no refresh token available".to_string(),
                });
            }
        }

        // No token found
        Err(TokenError::NotFound {
            service: service.to_string(),
            account: account.to_string(),
        })
    }

    async fn get_token_set(
        &self,
        service: &ServiceId,
        account: &AccountId,
    ) -> Result<Option<TokenSet>, TokenError> {
        // Try to retrieve access token
        let access_token_secret = self
            .get_credential(service, account, CredentialType::AccessToken)
            .await?;

        let access_token_str = match access_token_secret {
            Some(secret) => secret,
            None => return Ok(None),
        };

        // Build token with expiry if available
        let mut token = Token::new(access_token_str.expose());

        // Try to get expiry timestamp
        if let Some(expiry_secret) = self
            .get_credential(service, account, CredentialType::TokenExpiry)
            .await?
        {
            if let Ok(timestamp) = expiry_secret.expose().parse::<i64>() {
                if let Some(expires_at) = chrono::DateTime::from_timestamp(timestamp, 0) {
                    token = token.with_expiry(expires_at);
                }
            }
        }

        // Try to get scopes
        if let Some(scopes_secret) = self
            .get_credential(service, account, CredentialType::TokenScopes)
            .await?
        {
            let scopes: Vec<String> = scopes_secret
                .expose()
                .split(',')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect();
            token = token.with_scopes(scopes);
        }

        // Try to get refresh token
        let refresh_token = self
            .get_credential(service, account, CredentialType::RefreshToken)
            .await?;

        let mut token_set = TokenSet::new(token);
        if let Some(refresh) = refresh_token {
            token_set = token_set.with_refresh_token(refresh.expose());
        }

        Ok(Some(token_set))
    }

    async fn store_token_set(
        &self,
        service: &ServiceId,
        account: &AccountId,
        token_set: TokenSet,
    ) -> Result<(), TokenError> {
        // Store access token
        self.store_credential(
            service,
            account,
            CredentialType::AccessToken,
            token_set.access_token.access_token.expose(),
        )
        .await?;

        // Store expiry if available
        if let Some(expires_at) = token_set.access_token.expires_at {
            let timestamp = expires_at.timestamp().to_string();
            self.store_credential(
                service,
                account,
                CredentialType::TokenExpiry,
                &timestamp,
            )
            .await?;
        }

        // Store scopes if available
        if !token_set.access_token.scopes.is_empty() {
            let scopes_str = token_set.access_token.scopes.join(",");
            self.store_credential(
                service,
                account,
                CredentialType::TokenScopes,
                &scopes_str,
            )
            .await?;
        }

        // Store refresh token if available
        if let Some(refresh_token) = &token_set.refresh_token {
            self.store_credential(
                service,
                account,
                CredentialType::RefreshToken,
                refresh_token.expose(),
            )
            .await?;
        }

        tracing::debug!("Stored token set for {}/{}", service, account);

        Ok(())
    }

    async fn revoke_tokens(
        &self,
        service: &ServiceId,
        account: &AccountId,
    ) -> Result<(), TokenError> {
        // Delete all token-related credentials
        let access_key = self.credential_key(service, account, CredentialType::AccessToken);
        let refresh_key = self.credential_key(service, account, CredentialType::RefreshToken);
        let expiry_key = self.credential_key(service, account, CredentialType::TokenExpiry);

        // Delete all (ignore errors for missing keys)
        let _ = self.store.delete(&access_key).await;
        let _ = self.store.delete(&refresh_key).await;
        let _ = self.store.delete(&expiry_key).await;

        tracing::info!("Revoked tokens for {}/{}", service, account);

        Ok(())
    }

    async fn introspect_token(
        &self,
        service: &ServiceId,
        account: &AccountId,
    ) -> Result<TokenInfo, TokenError> {
        // Get the current token set
        let token_set = self
            .get_token_set(service, account)
            .await?
            .ok_or_else(|| TokenError::NotFound {
                service: service.to_string(),
                account: account.to_string(),
            })?;

        // Build token info from what we know
        let active = !self.is_token_expired(&token_set.access_token);

        Ok(TokenInfo {
            active,
            subject: None,
            client_id: None,
            scopes: token_set.access_token.scopes.clone(),
            expires_at: token_set.access_token.expires_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::MemoryStore;

    #[tokio::test]
    async fn test_token_manager_not_found() {
        let store = MemoryStore::new();
        let registry = ProviderRegistry::new();
        let manager = DefaultTokenManager::new(store, registry);

        let service = ServiceId::new("test");
        let account = AccountId::new("test");

        let result = manager.ensure_access_token(&service, &account).await;
        assert!(matches!(result, Err(TokenError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_token_manager_store_and_retrieve() {
        let store = MemoryStore::new();
        let registry = ProviderRegistry::new();
        let manager = DefaultTokenManager::new(store, registry);

        let service = ServiceId::new("test");
        let account = AccountId::new("test");

        // Create and store a token set
        let token = Token::new("test-access-token")
            .with_expiry(Utc::now() + chrono::Duration::hours(1));
        let token_set = TokenSet::new(token).with_refresh_token("test-refresh-token");

        manager
            .store_token_set(&service, &account, token_set)
            .await
            .unwrap();

        // Retrieve it
        let retrieved = manager
            .get_token_set(&service, &account)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(
            retrieved.access_token.access_token.expose(),
            "test-access-token"
        );
        assert!(retrieved.refresh_token.is_some());
    }

    #[tokio::test]
    async fn test_token_manager_ensure_valid_token() {
        let store = MemoryStore::new();
        let registry = ProviderRegistry::new();
        let manager = DefaultTokenManager::new(store, registry);

        let service = ServiceId::new("test");
        let account = AccountId::new("test");

        // Store a valid token
        let token = Token::new("valid-token")
            .with_expiry(Utc::now() + chrono::Duration::hours(1));
        let token_set = TokenSet::new(token);

        manager
            .store_token_set(&service, &account, token_set)
            .await
            .unwrap();

        // Ensure access token should return the cached token
        let result = manager.ensure_access_token(&service, &account).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().access_token.expose(), "valid-token");
    }

    #[tokio::test]
    async fn test_token_manager_revoke_tokens() {
        let store = MemoryStore::new();
        let registry = ProviderRegistry::new();
        let manager = DefaultTokenManager::new(store, registry);

        let service = ServiceId::new("test");
        let account = AccountId::new("test");

        // Store a token
        let token = Token::new("test-token");
        let token_set = TokenSet::new(token);

        manager
            .store_token_set(&service, &account, token_set)
            .await
            .unwrap();

        // Revoke it
        manager.revoke_tokens(&service, &account).await.unwrap();

        // Verify it's gone
        let result = manager.get_token_set(&service, &account).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_token_manager_introspect() {
        let store = MemoryStore::new();
        let registry = ProviderRegistry::new();
        let manager = DefaultTokenManager::new(store, registry);

        let service = ServiceId::new("test");
        let account = AccountId::new("test");

        // Store a token with scopes
        let token = Token::new("test-token")
            .with_expiry(Utc::now() + chrono::Duration::hours(1))
            .with_scopes(vec!["read".to_string(), "write".to_string()]);
        let token_set = TokenSet::new(token);

        manager
            .store_token_set(&service, &account, token_set)
            .await
            .unwrap();

        // Introspect it
        let info = manager.introspect_token(&service, &account).await.unwrap();
        assert!(info.active);
        assert_eq!(info.scopes, vec!["read", "write"]);
    }
}
