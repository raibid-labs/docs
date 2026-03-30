//! Integration tests for token refresh functionality.
//!
//! These tests verify that the DefaultTokenManager correctly:
//! - Detects expired tokens
//! - Refreshes tokens using refresh tokens
//! - Handles refresh failures gracefully
//! - Persists token sets across operations

#![cfg(feature = "oauth")]

use chrono::{Duration, Utc};
use sigilforge_core::{
    model::{AccountId, ServiceId},
    provider::{ProviderConfig, ProviderRegistry},
    store::{MemoryStore, Secret, SecretStore},
    token::{Token, TokenError, TokenManager, TokenSet},
    token_manager::DefaultTokenManager,
};
use wiremock::{
    matchers::{body_string_contains, method, path},
    Mock, MockServer, ResponseTemplate,
};

/// Helper to create a test provider configuration.
fn create_test_provider(token_url: &str) -> ProviderConfig {
    ProviderConfig {
        id: "test-provider".to_string(),
        name: "Test Provider".to_string(),
        auth_url: "https://example.com/auth".to_string(),
        token_url: token_url.to_string(),
        revoke_url: None,
        default_scopes: vec![],
        supports_pkce: true,
        supports_device_code: false,
    }
}

/// Helper to set up a token manager with a test provider.
async fn setup_manager(
    token_url: &str,
) -> (DefaultTokenManager<MemoryStore>, ServiceId, AccountId) {
    let store = MemoryStore::new();
    let mut registry = ProviderRegistry::new();
    registry.register(create_test_provider(token_url));

    let manager = DefaultTokenManager::new(store, registry);
    let service = ServiceId::new("test-provider");
    let account = AccountId::new("test-account");

    // Store client credentials
    let client_id_key = format!("sigilforge/{}/{}/client_id", service.as_str(), account.as_str());
    let client_secret_key =
        format!("sigilforge/{}/{}/client_secret", service.as_str(), account.as_str());

    manager
        .store
        .set(&client_id_key, &Secret::new("test-client-id"))
        .await
        .unwrap();
    manager
        .store
        .set(&client_secret_key, &Secret::new("test-client-secret"))
        .await
        .unwrap();

    (manager, service, account)
}

#[tokio::test]
async fn test_ensure_access_token_returns_valid_token() {
    let (manager, service, account) = setup_manager("https://unused.example.com").await;

    // Store a valid token
    let token = Token::new("valid-access-token")
        .with_expiry(Utc::now() + Duration::hours(1))
        .with_scopes(vec!["read".to_string()]);

    let token_set = TokenSet::new(token).with_refresh_token("refresh-token");

    manager
        .store_token_set(&service, &account, token_set)
        .await
        .unwrap();

    // Ensure access token should return the cached token
    let result = manager.ensure_access_token(&service, &account).await;

    assert!(result.is_ok());
    let token = result.unwrap();
    assert_eq!(token.access_token.expose(), "valid-access-token");
    assert_eq!(token.scopes, vec!["read"]);
}

#[tokio::test]
async fn test_ensure_access_token_refreshes_expired_token() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock the token refresh endpoint
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=refresh_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "new-access-token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "new-refresh-token",
            "scope": "read write"
        })))
        .mount(&mock_server)
        .await;

    let (manager, service, account) = setup_manager(&format!("{}/token", mock_server.uri())).await;

    // Store an expired token with a refresh token
    let token = Token::new("expired-access-token")
        .with_expiry(Utc::now() - Duration::hours(1)) // Expired
        .with_scopes(vec!["read".to_string()]);

    let token_set = TokenSet::new(token).with_refresh_token("old-refresh-token");

    manager
        .store_token_set(&service, &account, token_set)
        .await
        .unwrap();

    // Ensure access token should trigger a refresh
    let result = manager.ensure_access_token(&service, &account).await;

    assert!(result.is_ok());
    let token = result.unwrap();
    assert_eq!(token.access_token.expose(), "new-access-token");
    assert_eq!(token.scopes, vec!["read", "write"]);

    // Verify the new token was persisted
    let stored = manager.get_token_set(&service, &account).await.unwrap();
    assert!(stored.is_some());
    let stored_set = stored.unwrap();
    assert_eq!(
        stored_set.access_token.access_token.expose(),
        "new-access-token"
    );
    assert!(stored_set.refresh_token.is_some());
    assert_eq!(
        stored_set.refresh_token.unwrap().expose(),
        "new-refresh-token"
    );
}

#[tokio::test]
async fn test_ensure_access_token_refresh_fails() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock the token refresh endpoint to return an error
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": "invalid_grant",
            "error_description": "The refresh token is invalid or expired"
        })))
        .mount(&mock_server)
        .await;

    let (manager, service, account) = setup_manager(&format!("{}/token", mock_server.uri())).await;

    // Store an expired token with a refresh token
    let token = Token::new("expired-access-token")
        .with_expiry(Utc::now() - Duration::hours(1));

    let token_set = TokenSet::new(token).with_refresh_token("invalid-refresh-token");

    manager
        .store_token_set(&service, &account, token_set)
        .await
        .unwrap();

    // Ensure access token should fail to refresh
    let result = manager.ensure_access_token(&service, &account).await;

    assert!(result.is_err());
    match result {
        Err(TokenError::Expired { .. }) => {}
        _ => panic!("Expected TokenError::Expired"),
    }
}

#[tokio::test]
async fn test_ensure_access_token_no_refresh_token() {
    let (manager, service, account) = setup_manager("https://unused.example.com").await;

    // Store an expired token WITHOUT a refresh token
    let token = Token::new("expired-access-token")
        .with_expiry(Utc::now() - Duration::hours(1));

    let token_set = TokenSet::new(token); // No refresh token

    manager
        .store_token_set(&service, &account, token_set)
        .await
        .unwrap();

    // Ensure access token should fail (no way to refresh)
    let result = manager.ensure_access_token(&service, &account).await;

    assert!(result.is_err());
    match result {
        Err(TokenError::Expired { message }) => {
            assert!(message.contains("no refresh token"));
        }
        _ => panic!("Expected TokenError::Expired with 'no refresh token'"),
    }
}

#[tokio::test]
async fn test_ensure_access_token_not_found() {
    let (manager, service, account) = setup_manager("https://unused.example.com").await;

    // Don't store any token
    let result = manager.ensure_access_token(&service, &account).await;

    assert!(result.is_err());
    match result {
        Err(TokenError::NotFound { .. }) => {}
        _ => panic!("Expected TokenError::NotFound"),
    }
}

#[tokio::test]
async fn test_token_set_persistence() {
    let (manager, service, account) = setup_manager("https://unused.example.com").await;

    // Create and store a complete token set
    let token = Token::new("access-token-123")
        .with_expiry(Utc::now() + Duration::hours(2))
        .with_scopes(vec!["read".to_string(), "write".to_string()]);

    let token_set = TokenSet::new(token).with_refresh_token("refresh-token-456");

    manager
        .store_token_set(&service, &account, token_set)
        .await
        .unwrap();

    // Retrieve and verify
    let retrieved = manager
        .get_token_set(&service, &account)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        retrieved.access_token.access_token.expose(),
        "access-token-123"
    );
    assert_eq!(retrieved.access_token.scopes, vec!["read", "write"]);
    assert!(retrieved.access_token.expires_at.is_some());
    assert!(retrieved.refresh_token.is_some());
    assert_eq!(
        retrieved.refresh_token.unwrap().expose(),
        "refresh-token-456"
    );
}

#[tokio::test]
async fn test_revoke_tokens_removes_all_credentials() {
    let (manager, service, account) = setup_manager("https://unused.example.com").await;

    // Store a token set
    let token = Token::new("access-token").with_expiry(Utc::now() + Duration::hours(1));
    let token_set = TokenSet::new(token).with_refresh_token("refresh-token");

    manager
        .store_token_set(&service, &account, token_set)
        .await
        .unwrap();

    // Verify it exists
    assert!(manager
        .get_token_set(&service, &account)
        .await
        .unwrap()
        .is_some());

    // Revoke tokens
    manager.revoke_tokens(&service, &account).await.unwrap();

    // Verify all tokens are gone
    assert!(manager
        .get_token_set(&service, &account)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_introspect_token_active() {
    let (manager, service, account) = setup_manager("https://unused.example.com").await;

    // Store an active token
    let token = Token::new("active-token")
        .with_expiry(Utc::now() + Duration::hours(1))
        .with_scopes(vec!["read".to_string()]);

    let token_set = TokenSet::new(token);

    manager
        .store_token_set(&service, &account, token_set)
        .await
        .unwrap();

    // Introspect
    let info = manager.introspect_token(&service, &account).await.unwrap();

    assert!(info.active);
    assert_eq!(info.scopes, vec!["read"]);
    assert!(info.expires_at.is_some());
}

#[tokio::test]
async fn test_introspect_token_expired() {
    let (manager, service, account) = setup_manager("https://unused.example.com").await;

    // Store an expired token
    let token = Token::new("expired-token")
        .with_expiry(Utc::now() - Duration::hours(1))
        .with_scopes(vec!["read".to_string()]);

    let token_set = TokenSet::new(token);

    manager
        .store_token_set(&service, &account, token_set)
        .await
        .unwrap();

    // Introspect
    let info = manager.introspect_token(&service, &account).await.unwrap();

    assert!(!info.active);
}

#[tokio::test]
async fn test_expiry_buffer_detection() {
    let (manager, service, account) = setup_manager("https://unused.example.com").await;

    // Store a token that expires in 3 minutes (within the 5-minute buffer)
    let token = Token::new("soon-to-expire").with_expiry(Utc::now() + Duration::minutes(3));

    let token_set = TokenSet::new(token);

    manager
        .store_token_set(&service, &account, token_set)
        .await
        .unwrap();

    // Introspect should report it as inactive (within buffer)
    let info = manager.introspect_token(&service, &account).await.unwrap();

    assert!(!info.active, "Token within expiry buffer should be inactive");
}

#[tokio::test]
async fn test_multiple_accounts_same_service() {
    let (manager, service, _) = setup_manager("https://unused.example.com").await;

    let account1 = AccountId::new("account-1");
    let account2 = AccountId::new("account-2");

    // Store different client credentials for each account
    for account in [&account1, &account2] {
        let client_id_key = format!("sigilforge/{}/{}/client_id", service.as_str(), account.as_str());
        let client_secret_key =
            format!("sigilforge/{}/{}/client_secret", service.as_str(), account.as_str());

        manager
            .store
            .set(&client_id_key, &Secret::new("test-client-id"))
            .await
            .unwrap();
        manager
            .store
            .set(&client_secret_key, &Secret::new("test-client-secret"))
            .await
            .unwrap();
    }

    // Store tokens for both accounts
    let token1 = Token::new("token-for-account-1");
    manager
        .store_token_set(&service, &account1, TokenSet::new(token1))
        .await
        .unwrap();

    let token2 = Token::new("token-for-account-2");
    manager
        .store_token_set(&service, &account2, TokenSet::new(token2))
        .await
        .unwrap();

    // Retrieve and verify they're separate
    let retrieved1 = manager
        .get_token_set(&service, &account1)
        .await
        .unwrap()
        .unwrap();
    let retrieved2 = manager
        .get_token_set(&service, &account2)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        retrieved1.access_token.access_token.expose(),
        "token-for-account-1"
    );
    assert_eq!(
        retrieved2.access_token.access_token.expose(),
        "token-for-account-2"
    );
}
