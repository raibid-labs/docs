//! Integration tests for the daemon RPC API.
//!
//! These tests verify that the JSON-RPC server works correctly over Unix sockets
//! and that basic RPC operations succeed.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::{sleep, Duration};

use sigilforge_core::account_store::AccountStore;

/// Response containing a fresh access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetTokenResponse {
    token: String,
    expires_at: Option<String>,
}

/// Information about a configured account.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountInfo {
    service: String,
    account: String,
    scopes: Vec<String>,
    created_at: String,
    last_used: Option<String>,
}

/// Response containing a list of accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListAccountsResponse {
    accounts: Vec<AccountInfo>,
}

/// Response after adding an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AddAccountResponse {
    message: String,
}

/// Detect whether the sandbox allows binding Unix sockets. Skip tests if not.
fn can_bind_unix_socket() -> bool {
    let path = std::env::temp_dir().join("sigilforge-socket-permission-check.sock");
    let _ = fs::remove_file(&path);
    let result = std::os::unix::net::UnixListener::bind(&path);
    let ok = result.is_ok();
    let _ = fs::remove_file(&path);
    ok
}

/// Helper function to send an RPC request and receive a response.
async fn send_rpc_request<T: for<'de> Deserialize<'de>>(
    stream: &mut UnixStream,
    method: &str,
    params: serde_json::Value,
    id: u64,
) -> Result<T, Box<dyn std::error::Error>> {
    let request = json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": id,
    });

    let request_str = serde_json::to_string(&request)?;
    stream.write_all(request_str.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut response_str = String::new();
    reader.read_line(&mut response_str).await?;

    let response: serde_json::Value = serde_json::from_str(&response_str)?;

    if let Some(error) = response.get("error") {
        return Err(format!("RPC error: {}", error).into());
    }

    let result = response
        .get("result")
        .ok_or("No result in response")?;

    Ok(serde_json::from_value(result.clone())?)
}

#[tokio::test]
async fn test_add_and_list_accounts() {
    if !can_bind_unix_socket() {
        eprintln!("Skipping test_add_and_list_accounts: Unix sockets not permitted in sandbox");
        return;
    }

    // Use a unique socket path for this test
    let socket_path = PathBuf::from("/tmp/sigilforge-test-add-list.sock");

    // Remove socket if it exists
    let _ = std::fs::remove_file(&socket_path);

    // Start the server in the background with a temp account store
    let socket_path_clone = socket_path.clone();
    let store_path = std::env::temp_dir().join("sigilforge-test-accounts-add-list.json");
    let store = AccountStore::load_from_path(store_path).unwrap();
    tokio::spawn(async move {
        use sigilforge_daemon::api::{start_server, ApiState};
        let state = ApiState::with_store(store);
        if let Err(e) = start_server(&socket_path_clone, state).await {
            eprintln!("Server start failed: {}", e);
        }
    });

    // Give the server time to start and begin accepting connections
    sleep(Duration::from_millis(1000)).await;

    // Verify socket exists
    if !socket_path.exists() {
        panic!("Socket file was not created at {:?}", socket_path);
    }

    // Connect to the server
    let mut stream = UnixStream::connect(&socket_path)
        .await
        .expect("Failed to connect to daemon");

    // Test 1: List accounts (should be empty initially)
    let list_response: ListAccountsResponse =
        send_rpc_request(&mut stream, "list_accounts", json!([null]), 1)
            .await
            .expect("list_accounts failed");

    assert_eq!(list_response.accounts.len(), 0);

    // Test 2: Add an account
    let add_response: AddAccountResponse = send_rpc_request(
        &mut stream,
        "add_account",
        json!(["spotify", "personal", ["user-read-email", "playlist-read-private"]]),
        2,
    )
    .await
    .expect("add_account failed");

    assert!(add_response.message.contains("added successfully"));

    // Test 3: List accounts again (should have one account now)
    let list_response: ListAccountsResponse =
        send_rpc_request(&mut stream, "list_accounts", json!([null]), 3)
            .await
            .expect("list_accounts failed");

    assert_eq!(list_response.accounts.len(), 1);
    assert_eq!(list_response.accounts[0].service, "spotify");
    assert_eq!(list_response.accounts[0].account, "personal");
    assert_eq!(list_response.accounts[0].scopes.len(), 2);

    // Test 4: Add another account
    let _add_response: AddAccountResponse = send_rpc_request(
        &mut stream,
        "add_account",
        json!(["github", "work", ["repo", "user"]]),
        4,
    )
    .await
    .expect("add_account failed");

    // Test 5: List accounts with filter
    let list_response: ListAccountsResponse =
        send_rpc_request(&mut stream, "list_accounts", json!(["spotify"]), 5)
            .await
            .expect("list_accounts failed");

    assert_eq!(list_response.accounts.len(), 1);
    assert_eq!(list_response.accounts[0].service, "spotify");

    // Cleanup
    let _ = std::fs::remove_file(&socket_path);
}

#[tokio::test]
async fn test_get_token() {
    if !can_bind_unix_socket() {
        eprintln!("Skipping test_get_token: Unix sockets not permitted in sandbox");
        return;
    }

    // Use a unique socket path for this test
    let socket_path = PathBuf::from("/tmp/sigilforge-test-token.sock");

    // Remove socket if it exists
    let _ = std::fs::remove_file(&socket_path);

    // Start the server in the background with a temp account store
    let socket_path_clone = socket_path.clone();
    let store_path = std::env::temp_dir().join("sigilforge-test-accounts-token.json");
    let store = AccountStore::load_from_path(store_path).unwrap();
    tokio::spawn(async move {
        use sigilforge_daemon::api::{start_server, ApiState};
        let state = ApiState::with_store(store);
        if let Err(e) = start_server(&socket_path_clone, state).await {
            eprintln!("Server start failed: {}", e);
        }
    });

    // Give the server time to start and begin accepting connections
    sleep(Duration::from_millis(1000)).await;

    // Verify socket exists
    if !socket_path.exists() {
        panic!("Socket file was not created at {:?}", socket_path);
    }

    // Connect to the server
    let mut stream = UnixStream::connect(&socket_path)
        .await
        .expect("Failed to connect to daemon");

    // Add an account first
    let _: AddAccountResponse = send_rpc_request(
        &mut stream,
        "add_account",
        json!(["spotify", "personal", ["user-read-email"]]),
        1,
    )
    .await
    .expect("add_account failed");

    // Get a token for the account
    let token_response: GetTokenResponse =
        send_rpc_request(&mut stream, "get_token", json!(["spotify", "personal"]), 2)
            .await
            .expect("get_token failed");

    assert!(!token_response.token.is_empty());
    assert!(token_response.expires_at.is_some());

    // Cleanup
    let _ = std::fs::remove_file(&socket_path);
}

#[tokio::test]
async fn test_resolve() {
    if !can_bind_unix_socket() {
        eprintln!("Skipping test_resolve: Unix sockets not permitted in sandbox");
        return;
    }

    // Use a unique socket path for this test
    let socket_path = PathBuf::from("/tmp/sigilforge-test-resolve.sock");

    // Remove socket if it exists
    let _ = std::fs::remove_file(&socket_path);

    // Start the server in the background with a temp account store
    let socket_path_clone = socket_path.clone();
    let store_path = std::env::temp_dir().join("sigilforge-test-accounts-resolve.json");
    let store = AccountStore::load_from_path(store_path).unwrap();
    tokio::spawn(async move {
        use sigilforge_daemon::api::{start_server, ApiState};
        let state = ApiState::with_store(store);
        if let Err(e) = start_server(&socket_path_clone, state).await {
            eprintln!("Server start failed: {}", e);
        }
    });

    // Give the server time to start and begin accepting connections
    sleep(Duration::from_millis(1000)).await;

    // Verify socket exists
    if !socket_path.exists() {
        panic!("Socket file was not created at {:?}", socket_path);
    }

    // Connect to the server
    let mut stream = UnixStream::connect(&socket_path)
        .await
        .expect("Failed to connect to daemon");

    // Add an account first
    let _: AddAccountResponse = send_rpc_request(
        &mut stream,
        "add_account",
        json!(["spotify", "personal", ["user-read-email"]]),
        1,
    )
    .await
    .expect("add_account failed");

    // Resolve a credential reference
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct ResolveResponse {
        value: String,
    }

    let resolve_response: ResolveResponse = send_rpc_request(
        &mut stream,
        "resolve",
        json!(["auth://spotify/personal/token"]),
        2,
    )
    .await
    .expect("resolve failed");

    assert!(!resolve_response.value.is_empty());

    // Cleanup
    let _ = std::fs::remove_file(&socket_path);
}

#[tokio::test]
async fn test_error_handling() {
    if !can_bind_unix_socket() {
        eprintln!("Skipping test_error_handling: Unix sockets not permitted in sandbox");
        return;
    }

    // Use a unique socket path for this test
    let socket_path = PathBuf::from("/tmp/sigilforge-test-errors.sock");

    // Remove socket if it exists
    let _ = std::fs::remove_file(&socket_path);

    // Start the server in the background
    let socket_path_clone = socket_path.clone();
    tokio::spawn(async move {
        use sigilforge_daemon::api::{start_server, ApiState};
        if let Ok(state) = ApiState::new() {
            if let Err(e) = start_server(&socket_path_clone, state).await {
                eprintln!("Server start failed: {}", e);
            }
        } else {
            eprintln!("ApiState init failed");
        }
    });

    // Give the server time to start and begin accepting connections
    sleep(Duration::from_millis(1000)).await;

    // Verify socket exists
    if !socket_path.exists() {
        panic!("Socket file was not created at {:?}", socket_path);
    }

    // Connect to the server
    let mut stream = UnixStream::connect(&socket_path)
        .await
        .expect("Failed to connect to daemon");

    // Try to get a token for a non-existent account
    let result: Result<GetTokenResponse, _> =
        send_rpc_request(&mut stream, "get_token", json!(["nonexistent", "account"]), 1).await;

    assert!(result.is_err());

    // Try to add a duplicate account
    let _: AddAccountResponse = send_rpc_request(
        &mut stream,
        "add_account",
        json!(["spotify", "personal", ["user-read-email"]]),
        2,
    )
    .await
    .expect("add_account failed");

    let result: Result<AddAccountResponse, _> = send_rpc_request(
        &mut stream,
        "add_account",
        json!(["spotify", "personal", ["user-read-email"]]),
        3,
    )
    .await;

    assert!(result.is_err());

    // Cleanup
    let _ = std::fs::remove_file(&socket_path);
}
