//! Authorization Code flow with PKCE (Proof Key for Code Exchange).
//!
//! This module implements the OAuth 2.0 Authorization Code flow with PKCE,
//! which is the recommended flow for native and single-page applications.
//!
//! # Flow Overview
//!
//! 1. Generate PKCE code verifier and challenge
//! 2. Build authorization URL with state and PKCE challenge
//! 3. User authorizes in browser
//! 4. Receive authorization code via redirect
//! 5. Exchange code for tokens using PKCE verifier
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "oauth")]
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use sigilforge_core::oauth::pkce::PkceFlow;
//! use sigilforge_core::provider::ProviderRegistry;
//!
//! let registry = ProviderRegistry::with_defaults();
//! let github = registry.get("github").unwrap();
//!
//! let flow = PkceFlow::new(
//!     github.clone(),
//!     "my-client-id".to_string(),
//!     Some("my-client-secret".to_string()),
//!     "http://localhost:8080/callback".to_string(),
//! )?;
//!
//! let (auth_url, _csrf_state) = flow.build_authorization_url(vec!["repo".to_string()]);
//! println!("Visit: {}", auth_url);
//!
//! // After user authorizes and you receive the code...
//! let token_set = flow.exchange_code("authorization-code").await?;
//! # Ok(())
//! # }
//! ```

use oauth2::{
    AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, Scope,
    TokenResponse, reqwest::async_http_client,
};
use std::sync::{Arc, Mutex};

use crate::provider::ProviderConfig;
use crate::token::{Token, TokenSet, TokenError};
use super::create_oauth_client;

/// PKCE flow implementation for OAuth 2.0 authorization code flow.
///
/// This struct manages the PKCE code verifier/challenge and provides methods
/// for building authorization URLs and exchanging authorization codes for tokens.
pub struct PkceFlow {
    config: ProviderConfig,
    client_id: String,
    client_secret: Option<String>,
    redirect_uri: String,
    verifier: Arc<Mutex<Option<PkceCodeVerifier>>>,
}

impl PkceFlow {
    /// Create a new PKCE flow.
    ///
    /// # Arguments
    ///
    /// * `config` - OAuth provider configuration
    /// * `client_id` - OAuth client ID
    /// * `client_secret` - Optional client secret (for confidential clients)
    /// * `redirect_uri` - Redirect URI registered with the provider
    pub fn new(
        config: ProviderConfig,
        client_id: String,
        client_secret: Option<String>,
        redirect_uri: String,
    ) -> Result<Self, TokenError> {
        if !config.supports_pkce {
            tracing::warn!(
                "Provider {} does not advertise PKCE support, but attempting anyway",
                config.id
            );
        }

        Ok(Self {
            config,
            client_id,
            client_secret,
            redirect_uri,
            verifier: Arc::new(Mutex::new(None)),
        })
    }

    /// Build an authorization URL for the user to visit.
    ///
    /// This generates a new PKCE code verifier and challenge, and constructs
    /// the authorization URL with the challenge and a CSRF state token.
    ///
    /// # Arguments
    ///
    /// * `scopes` - OAuth scopes to request
    ///
    /// # Returns
    ///
    /// A tuple of (authorization URL, CSRF state token). The state token should
    /// be verified when receiving the redirect to prevent CSRF attacks.
    pub fn build_authorization_url(&self, scopes: Vec<String>) -> (String, String) {
        let client = create_oauth_client(
            &self.config,
            &self.client_id,
            self.client_secret.as_ref(),
            Some(&self.redirect_uri),
        )
        .expect("OAuth client configuration should be valid");

        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        // Store verifier for later use
        *self.verifier.lock().unwrap() = Some(pkce_verifier);

        // Build authorization URL
        let mut auth_request = client
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);

        // Add scopes
        for scope in scopes {
            auth_request = auth_request.add_scope(Scope::new(scope));
        }

        let (url, csrf_state) = auth_request.url();

        (url.to_string(), csrf_state.secret().to_string())
    }

    /// Exchange an authorization code for tokens.
    ///
    /// This exchanges the authorization code received from the redirect for
    /// an access token (and optionally a refresh token) using the PKCE verifier.
    ///
    /// # Arguments
    ///
    /// * `code` - Authorization code from the redirect
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The PKCE verifier is not available (authorization URL not generated)
    /// - The token exchange fails
    /// - Network errors occur
    pub async fn exchange_code(&self, code: impl Into<String>) -> Result<TokenSet, TokenError> {
        let verifier = self.verifier.lock().unwrap().take()
            .ok_or_else(|| TokenError::OAuthError {
                message: "PKCE verifier not found. Call build_authorization_url first.".to_string(),
            })?;

        let client = create_oauth_client(
            &self.config,
            &self.client_id,
            self.client_secret.as_ref(),
            Some(&self.redirect_uri),
        )?;

        let token_result = client
            .exchange_code(AuthorizationCode::new(code.into()))
            .set_pkce_verifier(verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| TokenError::OAuthError {
                message: format!("token exchange failed: {}", e),
            })?;

        // Extract token information
        let access_token = token_result.access_token().secret().to_string();
        let expires_in = token_result.expires_in();
        let scopes = token_result
            .scopes()
            .map(|s| s.iter().map(|scope| scope.to_string()).collect())
            .unwrap_or_default();

        let mut token = Token::new(access_token)
            .with_scopes(scopes);

        // Set expiration if provided
        if let Some(duration) = expires_in {
            let expires_at = chrono::Utc::now() + chrono::Duration::from_std(duration)
                .map_err(|e| TokenError::OAuthError {
                    message: format!("invalid expiration duration: {}", e),
                })?;
            token = token.with_expiry(expires_at);
        }

        let mut token_set = TokenSet::new(token);

        // Add refresh token if provided
        if let Some(refresh_token) = token_result.refresh_token() {
            token_set = token_set.with_refresh_token(refresh_token.secret());
        }

        Ok(token_set)
    }

    /// Start a local HTTP server to listen for the OAuth callback.
    ///
    /// This is a convenience method that starts a simple HTTP server on the
    /// specified port to receive the authorization code. The server will
    /// automatically shut down after receiving the callback.
    ///
    /// # Arguments
    ///
    /// * `port` - Port to listen on (must match the redirect URI)
    /// * `csrf_state` - Expected CSRF state token for validation
    ///
    /// # Returns
    ///
    /// The authorization code received from the callback, or an error.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "oauth")]
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # use sigilforge_core::oauth::pkce::PkceFlow;
    /// # use sigilforge_core::provider::ProviderRegistry;
    /// # let registry = ProviderRegistry::with_defaults();
    /// # let github = registry.get("github").unwrap();
    /// # let flow = PkceFlow::new(
    /// #     github.clone(),
    /// #     "client-id".to_string(),
    /// #     None,
    /// #     "http://localhost:8080/callback".to_string(),
    /// # )?;
    /// let (auth_url, csrf_state) = flow.build_authorization_url(vec![]);
    ///
    /// println!("Visit: {}", auth_url);
    /// let code = flow.listen_for_callback(8080, &csrf_state).await?;
    /// let token_set = flow.exchange_code(code).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn listen_for_callback(
        &self,
        port: u16,
        expected_state: &str,
    ) -> Result<String, TokenError> {
        use tokio::net::TcpListener;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| TokenError::OAuthError {
                message: format!("failed to bind to {}: {}", addr, e),
            })?;

        tracing::info!("Listening for OAuth callback on {}", addr);

        loop {
            let (mut socket, _) = listener.accept()
                .await
                .map_err(|e| TokenError::OAuthError {
                    message: format!("failed to accept connection: {}", e),
                })?;

            let mut buffer = [0; 4096];
            let n = socket.read(&mut buffer)
                .await
                .map_err(|e| TokenError::OAuthError {
                    message: format!("failed to read request: {}", e),
                })?;

            let request = String::from_utf8_lossy(&buffer[..n]);

            // Parse the request line
            if let Some(first_line) = request.lines().next() {
                if let Some(path) = first_line.split_whitespace().nth(1) {
                    // Parse query parameters
                    if let Some(query) = path.split('?').nth(1) {
                        let mut code = None;
                        let mut state = None;
                        let mut error = None;

                        for param in query.split('&') {
                            let parts: Vec<&str> = param.splitn(2, '=').collect();
                            if parts.len() == 2 {
                                match parts[0] {
                                    "code" => code = Some(parts[1].to_string()),
                                    "state" => state = Some(parts[1].to_string()),
                                    "error" => error = Some(parts[1].to_string()),
                                    _ => {}
                                }
                            }
                        }

                        // Check for OAuth error
                        if let Some(err) = error {
                            let response = b"HTTP/1.1 200 OK\r\n\r\n\
                                <html><body><h1>Authentication Failed</h1>\
                                <p>The OAuth provider returned an error.</p></body></html>";
                            let _ = socket.write_all(response).await;

                            return Err(TokenError::OAuthError {
                                message: format!("OAuth provider returned error: {}", err),
                            });
                        }

                        // Verify state
                        if let Some(received_state) = &state {
                            if received_state != expected_state {
                                let response = b"HTTP/1.1 200 OK\r\n\r\n\
                                    <html><body><h1>Authentication Failed</h1>\
                                    <p>Invalid state parameter (CSRF protection).</p></body></html>";
                                let _ = socket.write_all(response).await;

                                return Err(TokenError::OAuthError {
                                    message: "state parameter mismatch".to_string(),
                                });
                            }
                        }

                        // Return the code
                        if let Some(auth_code) = code {
                            let response = b"HTTP/1.1 200 OK\r\n\r\n\
                                <html><body><h1>Authentication Successful!</h1>\
                                <p>You can close this window and return to your application.</p></body></html>";
                            let _ = socket.write_all(response).await;

                            return Ok(auth_code);
                        }
                    }
                }
            }

            // If we got here, something was wrong with the request
            let response = b"HTTP/1.1 400 Bad Request\r\n\r\n\
                <html><body><h1>Bad Request</h1></body></html>";
            let _ = socket.write_all(response).await;
        }
    }
}

#[cfg(all(test, feature = "oauth"))]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_flow_new() {
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

        let flow = PkceFlow::new(
            config,
            "client-id".to_string(),
            None,
            "http://localhost:8080/callback".to_string(),
        );

        assert!(flow.is_ok());
    }

    #[test]
    fn test_build_authorization_url() {
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

        let flow = PkceFlow::new(
            config,
            "client-id".to_string(),
            None,
            "http://localhost:8080/callback".to_string(),
        )
        .unwrap();

        let (url, state) = flow.build_authorization_url(vec!["read".to_string()]);

        assert!(url.contains("https://example.com/auth"));
        assert!(url.contains("client_id=client-id"));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(!state.is_empty());
    }
}
