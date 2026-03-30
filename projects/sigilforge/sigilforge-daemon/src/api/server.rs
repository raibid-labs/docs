//! JSON-RPC server implementation with Unix socket support.

use super::handlers::{ApiState, SigilforgeApiImpl, SigilforgeApiServer};
use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// Handle to a running RPC server
pub struct ServerHandle {
    shutdown: Arc<Mutex<Option<tokio::sync::mpsc::Sender<()>>>>,
    join_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
}

/// Start the JSON-RPC server on a Unix socket.
///
/// # Parameters
///
/// - `socket_path`: Path to the Unix socket file
/// - `state`: API state shared across handlers
///
/// # Returns
///
/// A handle to the running server that can be used to stop it.
pub async fn start_server(socket_path: &Path, state: ApiState) -> Result<ServerHandle> {
    // Remove existing socket if present
    if socket_path.exists() {
        warn!("Removing existing socket at {:?}", socket_path);
        std::fs::remove_file(socket_path)
            .with_context(|| format!("Failed to remove existing socket at {:?}", socket_path))?;
    }

    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create socket directory {:?}", parent))?;
    }

    info!("Starting JSON-RPC server on {:?}", socket_path);

    // Create Unix listener
    let listener = UnixListener::bind(socket_path)
        .with_context(|| format!("Failed to bind Unix socket at {:?}", socket_path))?;

    // Create the RPC API implementation
    let api = Arc::new(SigilforgeApiImpl::new(state));

    // Create a cancellation token
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);
    let handle_tx = tx.clone();

    // Spawn server task
    let server_task: JoinHandle<()> = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = rx.recv() => {
                    debug!("Server shutdown signal received");
                    break;
                }
                result = listener.accept() => {
                    match result {
                        Ok((stream, _addr)) => {
                            let api = api.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, api).await {
                                    warn!("Connection handler error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            warn!("Failed to accept connection: {}", e);
                        }
                    }
                }
            }
        }
    });

    info!("JSON-RPC server started and listening");

    // Create a server handle
    let handle = ServerHandle {
        shutdown: Arc::new(Mutex::new(Some(handle_tx))),
        join_handle: Arc::new(Mutex::new(Some(server_task))),
    };

    Ok(handle)
}

/// Handle a single connection
async fn handle_connection(
    mut stream: UnixStream,
    api: Arc<SigilforgeApiImpl>,
) -> Result<()> {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;

        if n == 0 {
            // Connection closed
            break;
        }

        debug!("Received request: {}", line.trim());

        // Parse JSON-RPC request
        let request: serde_json::Value = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                let error_response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": format!("Parse error: {}", e)
                    },
                    "id": null
                });
                writer.write_all(error_response.to_string().as_bytes()).await?;
                writer.write_all(b"\n").await?;
                writer.flush().await?;
                continue;
            }
        };

        // Process request and send response
        let response = process_request(request, &api).await;
        writer.write_all(response.to_string().as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}

/// Process a JSON-RPC request
async fn process_request(
    request: serde_json::Value,
    api: &Arc<SigilforgeApiImpl>,
) -> serde_json::Value {
    use jsonrpsee::types::ErrorObject;

    let id = request.get("id").cloned().unwrap_or(serde_json::Value::Null);
    let method = match request.get("method").and_then(|m| m.as_str()) {
        Some(m) => m,
        None => {
            return serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32600,
                    "message": "Invalid Request: missing method"
                },
                "id": id
            });
        }
    };

    let params = request.get("params").cloned().unwrap_or(serde_json::Value::Array(vec![]));

    // Call the appropriate method
    let result = match method {
        "get_token" => {
            let params_array = params.as_array();
            if let Some(arr) = params_array {
                if arr.len() >= 2 {
                    if let (Some(service), Some(account)) = (arr[0].as_str(), arr[1].as_str()) {
                        match api.get_token(service.to_string(), account.to_string()).await {
                            Ok(resp) => Ok(serde_json::to_value(resp).unwrap()),
                            Err(e) => Err(e),
                        }
                    } else {
                        Err(ErrorObject::owned(-32602, "Invalid params", None::<()>))
                    }
                } else {
                    Err(ErrorObject::owned(-32602, "Invalid params", None::<()>))
                }
            } else {
                Err(ErrorObject::owned(-32602, "Invalid params", None::<()>))
            }
        }
        "list_accounts" => {
            let service = params.as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            match api.list_accounts(service).await {
                Ok(resp) => Ok(serde_json::to_value(resp).unwrap()),
                Err(e) => Err(e),
            }
        }
        "add_account" => {
            let params_array = params.as_array();
            if let Some(arr) = params_array {
                if arr.len() >= 3 {
                    if let (Some(service), Some(account), Some(scopes)) =
                        (arr[0].as_str(), arr[1].as_str(), arr[2].as_array()) {
                        let scopes_vec: Vec<String> = scopes
                            .iter()
                            .filter_map(|s| s.as_str().map(|s| s.to_string()))
                            .collect();
                        match api.add_account(service.to_string(), account.to_string(), scopes_vec).await {
                            Ok(resp) => Ok(serde_json::to_value(resp).unwrap()),
                            Err(e) => Err(e),
                        }
                    } else {
                        Err(ErrorObject::owned(-32602, "Invalid params", None::<()>))
                    }
                } else {
                    Err(ErrorObject::owned(-32602, "Invalid params", None::<()>))
                }
            } else {
                Err(ErrorObject::owned(-32602, "Invalid params", None::<()>))
            }
        }
        "resolve" => {
            let reference = params.as_array()
                .and_then(|arr| arr.first())
                .and_then(|v| v.as_str());
            if let Some(ref_str) = reference {
                match api.resolve(ref_str.to_string()).await {
                    Ok(resp) => Ok(serde_json::to_value(resp).unwrap()),
                    Err(e) => Err(e),
                }
            } else {
                Err(ErrorObject::owned(-32602, "Invalid params", None::<()>))
            }
        }
        _ => Err(ErrorObject::owned(-32601, "Method not found", None::<()>)),
    };

    match result {
        Ok(value) => serde_json::json!({
            "jsonrpc": "2.0",
            "result": value,
            "id": id
        }),
        Err(error) => serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": error.code(),
                "message": error.message()
            },
            "id": id
        }),
    }
}

impl ServerHandle {
    /// Stop the server
    pub async fn stop(&self) -> Result<()> {
        if let Some(tx) = self.shutdown.lock().await.take() {
            let _ = tx.send(()).await;
        }

        if let Some(handle) = self.join_handle.lock().await.take() {
            // If the task panicked, surface the error
            handle.await?;
        }

        Ok(())
    }

    /// Wait for the server to stop
    pub async fn stopped(&self) {
        // No-op: stop() already awaits the join handle.
    }
}

/// Configuration for the JSON-RPC server.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Path to the Unix socket
    pub socket_path: std::path::PathBuf,
}

impl ServerConfig {
    /// Create a new server configuration.
    #[allow(dead_code)]
    pub fn new(socket_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }
}
