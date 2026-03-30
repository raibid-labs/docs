//! Device Authorization Grant flow (RFC 8628).
//!
//! This module implements the OAuth 2.0 Device Authorization Grant flow,
//! which is designed for devices with limited input capabilities or no browser.
//!
//! # Flow Overview
//!
//! 1. Request device and user codes from the authorization server
//! 2. Display the user code and verification URL to the user
//! 3. User visits the URL on another device and enters the code
//! 4. Poll the token endpoint until the user authorizes or denies
//! 5. Receive tokens once authorization is complete
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "oauth")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use sigilforge_core::oauth::device_code::DeviceCodeFlow;
//! use sigilforge_core::provider::ProviderRegistry;
//!
//! let registry = ProviderRegistry::with_defaults();
//! let github = registry.get("github").unwrap();
//!
//! let flow = DeviceCodeFlow::new(
//!     github.clone(),
//!     "my-client-id".to_string(),
//!     None,
//! )?;
//!
//! let device_auth = flow.request_device_code(vec!["repo".to_string()]).await?;
//!
//! println!("Visit {} and enter code: {}",
//!          device_auth.verification_uri,
//!          device_auth.user_code);
//!
//! let token_set = flow.poll_for_token(
//!     &device_auth.device_code,
//!     device_auth.interval,
//!     device_auth.expires_in,
//! ).await?;
//! # Ok(())
//! # }
//! ```

use oauth2::{
    basic::BasicClient,
    DeviceAuthorizationUrl, Scope, StandardDeviceAuthorizationResponse,
    reqwest::async_http_client,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;

use crate::provider::ProviderConfig;
use crate::token::{Token, TokenSet, TokenError};
use super::create_oauth_client;

/// Device authorization response.
///
/// Contains the codes and URIs needed for the user to authorize the device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAuthorization {
    /// The device verification code (keep this secret).
    pub device_code: String,

    /// The user verification code to display to the user.
    pub user_code: String,

    /// The URI where the user should go to authorize.
    pub verification_uri: String,

    /// Optional URI with the user code embedded (for QR codes).
    pub verification_uri_complete: Option<String>,

    /// Minimum interval in seconds between polling requests.
    pub interval: u64,

    /// Time in seconds until the device code expires.
    pub expires_in: u64,
}

/// Device code flow implementation for OAuth 2.0 device authorization grant.
///
/// This flow is designed for devices with limited input capabilities or
/// headless environments where a browser is not available.
pub struct DeviceCodeFlow {
    config: ProviderConfig,
    client_id: String,
    client_secret: Option<String>,
}

impl DeviceCodeFlow {
    /// Create a new device code flow.
    ///
    /// # Arguments
    ///
    /// * `config` - OAuth provider configuration
    /// * `client_id` - OAuth client ID
    /// * `client_secret` - Optional client secret
    pub fn new(
        config: ProviderConfig,
        client_id: String,
        client_secret: Option<String>,
    ) -> Result<Self, TokenError> {
        if !config.supports_device_code {
            return Err(TokenError::OAuthError {
                message: format!("Provider {} does not support device code flow", config.id),
            });
        }

        Ok(Self {
            config,
            client_id,
            client_secret,
        })
    }

    /// Request device and user codes from the authorization server.
    ///
    /// # Arguments
    ///
    /// * `scopes` - OAuth scopes to request
    ///
    /// # Returns
    ///
    /// Device authorization information including the user code and verification URI.
    pub async fn request_device_code(
        &self,
        scopes: Vec<String>,
    ) -> Result<DeviceAuthorization, TokenError> {
        // Construct device authorization URL
        // For GitHub: https://github.com/login/device/code
        // For Google: https://oauth2.googleapis.com/device/code
        let device_auth_url = self.get_device_auth_url()?;

        let client = self.create_client_with_device_url(&device_auth_url)?;

        // Build device authorization request
        let mut device_auth_request = client.exchange_device_code().map_err(|e| {
            TokenError::OAuthError {
                message: format!("failed to create device code request: {}", e),
            }
        })?;

        for scope in scopes {
            device_auth_request = device_auth_request.add_scope(Scope::new(scope));
        }

        // Execute the request
        let device_auth_response: StandardDeviceAuthorizationResponse = device_auth_request
            .request_async(async_http_client)
            .await
            .map_err(|e| TokenError::OAuthError {
                message: format!("device code request failed: {}", e),
            })?;

        Ok(DeviceAuthorization {
            device_code: device_auth_response.device_code().secret().to_string(),
            user_code: device_auth_response.user_code().secret().to_string(),
            verification_uri: device_auth_response.verification_uri().to_string(),
            verification_uri_complete: device_auth_response
                .verification_uri_complete()
                .map(|uri| uri.secret().to_string()),
            interval: device_auth_response.interval().as_secs(),
            expires_in: device_auth_response.expires_in().as_secs(),
        })
    }

    /// Poll the token endpoint until the user authorizes or the request expires.
    ///
    /// # Arguments
    ///
    /// * `device_auth` - Device authorization response from request_device_code
    ///
    /// # Returns
    ///
    /// Token set once the user has authorized.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The user denies the request
    /// - The device code expires
    /// - The authorization server returns an error
    pub async fn poll_for_token(
        &self,
        device_auth: &DeviceAuthorization,
    ) -> Result<TokenSet, TokenError> {
        let _device_auth_url = self.get_device_auth_url()?;

        // Recreate the device authorization response for the oauth2 crate
        // We need to store and pass the original response, but for simplicity
        // we'll use a manual polling approach
        let poll_interval = Duration::from_secs(device_auth.interval);
        let timeout = Duration::from_secs(device_auth.expires_in);
        let start_time = std::time::Instant::now();

        loop {
            // Check for timeout
            if start_time.elapsed() > timeout {
                return Err(TokenError::OAuthError {
                    message: "device code expired".to_string(),
                });
            }

            // Wait before polling
            sleep(poll_interval).await;

            // Build the token request manually
            let token_result = reqwest::Client::new()
                .post(&self.config.token_url)
                .form(&[
                    ("client_id", self.client_id.as_str()),
                    ("device_code", device_auth.device_code.as_str()),
                    ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ])
                .send()
                .await;

            match token_result {
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();

                    if status.is_success() {
                        // Parse the successful token response
                        let token_data: serde_json::Value = serde_json::from_str(&body)
                            .map_err(|e| TokenError::OAuthError {
                                message: format!("failed to parse token response: {}", e),
                            })?;

                        let access_token = token_data["access_token"]
                            .as_str()
                            .ok_or_else(|| TokenError::OAuthError {
                                message: "missing access_token in response".to_string(),
                            })?
                            .to_string();

                        let expires_in = token_data["expires_in"].as_u64();
                        let scopes = token_data["scope"]
                            .as_str()
                            .map(|s| s.split_whitespace().map(String::from).collect())
                            .unwrap_or_default();

                        let mut token = Token::new(access_token).with_scopes(scopes);

                        if let Some(seconds) = expires_in {
                            let expires_at = chrono::Utc::now()
                                + chrono::Duration::seconds(seconds as i64);
                            token = token.with_expiry(expires_at);
                        }

                        let mut token_set = TokenSet::new(token);

                        if let Some(refresh_token) = token_data["refresh_token"].as_str() {
                            token_set = token_set.with_refresh_token(refresh_token);
                        }

                        return Ok(token_set);
                    } else {
                        // Parse error response
                        if let Ok(error_data) = serde_json::from_str::<serde_json::Value>(&body) {
                            let error_code = error_data["error"].as_str().unwrap_or("unknown");

                            match error_code {
                                "authorization_pending" => {
                                    tracing::debug!("Authorization pending, continuing to poll...");
                                    continue;
                                }
                                "slow_down" => {
                                    tracing::warn!("Polling too fast, slowing down...");
                                    sleep(Duration::from_secs(5)).await;
                                    continue;
                                }
                                "access_denied" => {
                                    return Err(TokenError::OAuthError {
                                        message: "user denied authorization".to_string(),
                                    });
                                }
                                "expired_token" => {
                                    return Err(TokenError::OAuthError {
                                        message: "device code expired".to_string(),
                                    });
                                }
                                _ => {
                                    return Err(TokenError::OAuthError {
                                        message: format!("OAuth error: {}", error_code),
                                    });
                                }
                            }
                        } else {
                            return Err(TokenError::OAuthError {
                                message: format!("unexpected error response: {}", body),
                            });
                        }
                    }
                }
                Err(e) => {
                    return Err(TokenError::NetworkError {
                        message: format!("network error during polling: {}", e),
                    });
                }
            }
        }
    }

    /// Get the device authorization URL for the provider.
    ///
    /// This constructs the device authorization endpoint URL based on the provider.
    fn get_device_auth_url(&self) -> Result<String, TokenError> {
        // Provider-specific device authorization URLs
        match self.config.id.as_str() {
            "github" => Ok("https://github.com/login/device/code".to_string()),
            "google" => Ok("https://oauth2.googleapis.com/device/code".to_string()),
            _ => {
                // Try to infer from token URL
                if let Some(base) = self.config.token_url.rsplit_once('/') {
                    Ok(format!("{}/device/code", base.0))
                } else {
                    Err(TokenError::OAuthError {
                        message: format!(
                            "device authorization URL not configured for provider {}",
                            self.config.id
                        ),
                    })
                }
            }
        }
    }

    /// Create an OAuth client with device authorization URL.
    fn create_client_with_device_url(
        &self,
        device_auth_url: &str,
    ) -> Result<BasicClient, TokenError> {
        let mut client = create_oauth_client(
            &self.config,
            &self.client_id,
            self.client_secret.as_ref(),
            None::<String>,
        )?;

        let device_url = DeviceAuthorizationUrl::new(device_auth_url.to_string())
            .map_err(|e| TokenError::OAuthError {
                message: format!("invalid device authorization URL: {}", e),
            })?;

        client = client.set_device_authorization_url(device_url);

        Ok(client)
    }
}

#[cfg(all(test, feature = "oauth"))]
mod tests {
    use super::*;

    #[test]
    fn test_device_code_flow_new() {
        let config = ProviderConfig {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            revoke_url: None,
            default_scopes: vec![],
            supports_pkce: true,
            supports_device_code: true,
        };

        let flow = DeviceCodeFlow::new(config, "client-id".to_string(), None);
        assert!(flow.is_ok());
    }

    #[test]
    fn test_device_code_flow_unsupported_provider() {
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

        let flow = DeviceCodeFlow::new(config, "client-id".to_string(), None);
        assert!(flow.is_err());
    }

    #[test]
    fn test_get_device_auth_url_github() {
        let config = ProviderConfig {
            id: "github".to_string(),
            name: "GitHub".to_string(),
            auth_url: "https://github.com/login/oauth/authorize".to_string(),
            token_url: "https://github.com/login/oauth/access_token".to_string(),
            revoke_url: None,
            default_scopes: vec![],
            supports_pkce: true,
            supports_device_code: true,
        };

        let flow = DeviceCodeFlow::new(config, "client-id".to_string(), None).unwrap();
        let url = flow.get_device_auth_url().unwrap();

        assert_eq!(url, "https://github.com/login/device/code");
    }

    #[test]
    fn test_get_device_auth_url_google() {
        let config = ProviderConfig {
            id: "google".to_string(),
            name: "Google".to_string(),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            token_url: "https://oauth2.googleapis.com/token".to_string(),
            revoke_url: None,
            default_scopes: vec![],
            supports_pkce: true,
            supports_device_code: true,
        };

        let flow = DeviceCodeFlow::new(config, "client-id".to_string(), None).unwrap();
        let url = flow.get_device_auth_url().unwrap();

        assert_eq!(url, "https://oauth2.googleapis.com/device/code");
    }
}
