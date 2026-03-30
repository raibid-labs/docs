//! Contract tests for Sigilforge client.
//!
//! These tests verify the client's behavior with both real and mocked
//! Sigilforge daemon responses.

use scryforge_sigilforge_client::{MockTokenFetcher, SigilforgeClient, SigilforgeError, TokenFetcher};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

// ============================================================================
// Mock Server Helpers
// ============================================================================

/// Start a mock Sigilforge daemon that responds to get_token requests.
async fn start_mock_daemon(
    socket_path: PathBuf,
    tokens: HashMap<(String, String), String>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let listener = UnixListener::bind(&socket_path).expect("Failed to bind socket");

        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    let tokens = tokens.clone();
                    tokio::spawn(async move {
                        if let Err(e) = handle_mock_connection(&mut stream, tokens).await {
                            eprintln!("Mock daemon error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                    break;
                }
            }
        }
    })
}

/// Handle a single connection to the mock daemon.
async fn handle_mock_connection(
    stream: &mut UnixStream,
    tokens: HashMap<(String, String), String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(stream);
    let mut request_line = String::new();

    reader.read_line(&mut request_line).await?;

    let request: serde_json::Value = serde_json::from_str(&request_line)?;
    let method = request["method"].as_str().unwrap_or("");
    let empty_params = vec![];
    let params = request["params"].as_array().unwrap_or(&empty_params);
    let id = request["id"].clone();

    let response = match method {
        "get_token" => {
            let service = params
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let account = params
                .get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            if let Some(token) = tokens.get(&(service.clone(), account.clone())) {
                json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "token": token,
                        "expires_at": "2025-12-07T00:00:00Z"
                    },
                    "id": id
                })
            } else {
                json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32602,
                        "message": format!("Account {}/{} not found", service, account)
                    },
                    "id": id
                })
            }
        }
        "resolve" => {
            let reference = params.get(0).and_then(|v| v.as_str()).unwrap_or("");
            json!({
                "jsonrpc": "2.0",
                "result": {
                    "value": format!("resolved_{}", reference)
                },
                "id": id
            })
        }
        _ => json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32601,
                "message": "Method not found"
            },
            "id": id
        }),
    };

    let response_str = serde_json::to_string(&response)?;
    reader.get_mut().write_all(response_str.as_bytes()).await?;
    reader.get_mut().write_all(b"\n").await?;
    reader.get_mut().flush().await?;

    Ok(())
}

// ============================================================================
// Contract Tests
// ============================================================================

#[tokio::test]
async fn test_client_unavailable_returns_error() {
    let client = SigilforgeClient::new(PathBuf::from("/nonexistent/socket.sock"));

    assert!(!client.is_available());

    let result = client.get_token("spotify", "personal").await;
    assert!(matches!(result, Err(SigilforgeError::Unavailable(_))));
}

#[tokio::test]
async fn test_client_fetch_token_success() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");

    let mut tokens = HashMap::new();
    tokens.insert(
        ("spotify".to_string(), "personal".to_string()),
        "test_access_token_123".to_string(),
    );

    let _server = start_mock_daemon(socket_path.clone(), tokens).await;

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = SigilforgeClient::new(socket_path);
    assert!(client.is_available());

    let token = client.get_token("spotify", "personal").await.unwrap();
    assert_eq!(token, "test_access_token_123");
}

#[tokio::test]
async fn test_client_token_not_found() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");

    let tokens = HashMap::new();
    let _server = start_mock_daemon(socket_path.clone(), tokens).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = SigilforgeClient::new(socket_path);

    let result = client.get_token("github", "work").await;
    assert!(matches!(result, Err(SigilforgeError::TokenNotFound { .. })));
}

#[tokio::test]
async fn test_client_resolve_reference() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");

    let tokens = HashMap::new();
    let _server = start_mock_daemon(socket_path.clone(), tokens).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = SigilforgeClient::new(socket_path);

    let value = client
        .resolve("auth://spotify/personal/token")
        .await
        .unwrap();
    assert_eq!(value, "resolved_auth://spotify/personal/token");
}

#[tokio::test]
async fn test_token_fetcher_trait_impl() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");

    let mut tokens = HashMap::new();
    tokens.insert(
        ("spotify".to_string(), "personal".to_string()),
        "trait_test_token".to_string(),
    );

    let _server = start_mock_daemon(socket_path.clone(), tokens).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = SigilforgeClient::new(socket_path);
    let fetcher: &dyn TokenFetcher = &client;

    let token = fetcher.fetch_token("spotify", "personal").await.unwrap();
    assert_eq!(token, "trait_test_token");
}

#[tokio::test]
async fn test_mock_token_fetcher_success() {
    let mut tokens = HashMap::new();
    tokens.insert(
        ("spotify".to_string(), "personal".to_string()),
        "mock_token_abc".to_string(),
    );
    tokens.insert(
        ("github".to_string(), "work".to_string()),
        "mock_token_xyz".to_string(),
    );

    let fetcher = MockTokenFetcher::new(tokens);

    let token1 = fetcher.fetch_token("spotify", "personal").await.unwrap();
    assert_eq!(token1, "mock_token_abc");

    let token2 = fetcher.fetch_token("github", "work").await.unwrap();
    assert_eq!(token2, "mock_token_xyz");
}

#[tokio::test]
async fn test_mock_token_fetcher_not_found() {
    let fetcher = MockTokenFetcher::empty();

    let result = fetcher.fetch_token("spotify", "personal").await;
    assert!(matches!(result, Err(SigilforgeError::TokenNotFound { .. })));
}

#[tokio::test]
async fn test_mock_token_fetcher_builder() {
    let fetcher = MockTokenFetcher::empty()
        .with_token(
            "spotify".to_string(),
            "personal".to_string(),
            "builder_token_1".to_string(),
        )
        .with_token(
            "github".to_string(),
            "work".to_string(),
            "builder_token_2".to_string(),
        );

    let token1 = fetcher.fetch_token("spotify", "personal").await.unwrap();
    assert_eq!(token1, "builder_token_1");

    let token2 = fetcher.fetch_token("github", "work").await.unwrap();
    assert_eq!(token2, "builder_token_2");
}

#[tokio::test]
async fn test_multiple_requests() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");

    let mut tokens = HashMap::new();
    tokens.insert(
        ("spotify".to_string(), "personal".to_string()),
        "token_1".to_string(),
    );
    tokens.insert(
        ("github".to_string(), "work".to_string()),
        "token_2".to_string(),
    );

    let _server = start_mock_daemon(socket_path.clone(), tokens).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = SigilforgeClient::new(socket_path);

    let token1 = client.get_token("spotify", "personal").await.unwrap();
    assert_eq!(token1, "token_1");

    let token2 = client.get_token("github", "work").await.unwrap();
    assert_eq!(token2, "token_2");

    // Verify we can fetch the same token again
    let token1_again = client.get_token("spotify", "personal").await.unwrap();
    assert_eq!(token1_again, "token_1");
}

#[tokio::test]
async fn test_graceful_error_when_daemon_stops() {
    let temp_dir = TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("test.sock");

    let tokens = HashMap::new();
    let server = start_mock_daemon(socket_path.clone(), tokens).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let client = SigilforgeClient::new(socket_path.clone());
    assert!(client.is_available());

    // Stop the server
    server.abort();
    std::fs::remove_file(&socket_path).ok();

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Client should detect daemon is unavailable
    assert!(!client.is_available());

    let result = client.get_token("spotify", "personal").await;
    assert!(matches!(result, Err(SigilforgeError::Unavailable(_))));
}
