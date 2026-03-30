//! OAuth 2.0 flow implementations.
//!
//! This module provides OAuth 2.0 flow implementations:
//! - [`pkce`] - Authorization Code flow with PKCE
//! - [`device_code`] - Device Authorization Grant flow
//!
//! # Features
//!
//! This module is only available when the `oauth` feature is enabled.

#[cfg(feature = "oauth")]
pub mod pkce;

#[cfg(feature = "oauth")]
pub mod device_code;

#[cfg(feature = "oauth")]
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl,
};
#[cfg(feature = "oauth")]
use crate::provider::ProviderConfig;
#[cfg(feature = "oauth")]
use crate::token::TokenError;

/// Create an OAuth2 client from a provider configuration.
///
/// # Arguments
///
/// * `config` - Provider configuration
/// * `client_id` - OAuth client ID
/// * `client_secret` - Optional OAuth client secret (required for confidential clients)
/// * `redirect_uri` - Redirect URI for the authorization code flow
///
/// # Returns
///
/// A configured OAuth2 basic client ready for use in flows.
#[cfg(feature = "oauth")]
pub fn create_oauth_client(
    config: &ProviderConfig,
    client_id: impl Into<String>,
    client_secret: Option<impl Into<String>>,
    redirect_uri: Option<impl Into<String>>,
) -> Result<BasicClient, TokenError> {
    let auth_url = AuthUrl::new(config.auth_url.clone())
        .map_err(|e| TokenError::OAuthError {
            message: format!("invalid auth URL: {}", e),
        })?;

    let token_url = TokenUrl::new(config.token_url.clone())
        .map_err(|e| TokenError::OAuthError {
            message: format!("invalid token URL: {}", e),
        })?;

    let mut client = BasicClient::new(
        ClientId::new(client_id.into()),
        client_secret.map(|s| ClientSecret::new(s.into())),
        auth_url,
        Some(token_url),
    );

    if let Some(redirect) = redirect_uri {
        let redirect_url = RedirectUrl::new(redirect.into())
            .map_err(|e| TokenError::OAuthError {
                message: format!("invalid redirect URL: {}", e),
            })?;
        client = client.set_redirect_uri(redirect_url);
    }

    Ok(client)
}

/// Generate a random alphanumeric string of the specified length.
///
/// Used for generating state parameters and other random values in OAuth flows.
#[cfg(feature = "oauth")]
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

#[cfg(all(test, feature = "oauth"))]
mod tests {
    use super::*;

    #[test]
    fn test_create_oauth_client() {
        let config = ProviderConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            auth_url: "https://example.com/auth".to_string(),
            token_url: "https://example.com/token".to_string(),
            revoke_url: None,
            default_scopes: vec![],
            supports_pkce: true,
            supports_device_code: false,
        };

        let client = create_oauth_client(
            &config,
            "test-client-id",
            Some("test-secret"),
            Some("http://localhost:8080/callback"),
        );

        assert!(client.is_ok());
    }

    #[test]
    fn test_create_oauth_client_invalid_urls() {
        let config = ProviderConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            auth_url: "not a valid url".to_string(),
            token_url: "https://example.com/token".to_string(),
            revoke_url: None,
            default_scopes: vec![],
            supports_pkce: true,
            supports_device_code: false,
        };

        let client = create_oauth_client(
            &config,
            "test-client-id",
            Some("test-secret"),
            Some("http://localhost:8080/callback"),
        );

        assert!(client.is_err());
    }

    #[test]
    fn test_generate_random_string() {
        let s1 = generate_random_string(32);
        let s2 = generate_random_string(32);

        assert_eq!(s1.len(), 32);
        assert_eq!(s2.len(), 32);
        assert_ne!(s1, s2); // Should be different
        assert!(s1.chars().all(|c| c.is_alphanumeric()));
    }
}
