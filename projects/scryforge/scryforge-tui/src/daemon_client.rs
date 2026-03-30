//! Client for communicating with the scryforge-daemon.
//!
//! This module provides an async client for fetching streams and items
//! from the daemon via JSON-RPC over HTTP.

use anyhow::{Context, Result};
use fusabi_streams_core::{Item, Stream};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::rpc_params;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

/// Messages sent from the UI thread to the async client thread.
#[derive(Debug, Clone)]
pub enum Command {
    /// Fetch all available streams
    FetchStreams,
    /// Fetch items for a specific stream
    FetchItems(String),
    /// Shutdown the client
    Shutdown,
}

/// Messages sent from the async client thread to the UI thread.
#[derive(Debug, Clone)]
pub enum Message {
    /// Streams were loaded successfully
    StreamsLoaded(Vec<Stream>),
    /// Items were loaded successfully
    ItemsLoaded(Vec<Item>),
    /// An error occurred
    Error(String),
    /// Client is ready
    Ready,
    /// Client disconnected
    Disconnected,
}

/// Client for communicating with the scryforge-daemon.
pub struct DaemonClient {
    client: HttpClient,
}

impl DaemonClient {
    /// Connect to the daemon via HTTP.
    pub async fn connect(url: &str) -> Result<Self> {
        info!("Connecting to daemon at {}", url);

        // Build HTTP client
        let client = HttpClientBuilder::default()
            .build(url)
            .context("Failed to build HTTP client")?;

        info!("Connected to daemon successfully");

        Ok(Self { client })
    }

    /// List all available streams.
    pub async fn list_streams(&self) -> Result<Vec<Stream>> {
        debug!("Fetching streams from daemon");

        let streams: Vec<Stream> = self
            .client
            .request("streams.list", rpc_params![])
            .await
            .context("Failed to fetch streams")?;

        debug!("Fetched {} streams", streams.len());
        Ok(streams)
    }

    /// List items for a specific stream.
    pub async fn list_items(&self, stream_id: &str) -> Result<Vec<Item>> {
        debug!("Fetching items for stream: {}", stream_id);

        let items: Vec<Item> = self
            .client
            .request("items.list", rpc_params![stream_id])
            .await
            .context("Failed to fetch items")?;

        debug!("Fetched {} items for stream {}", items.len(), stream_id);
        Ok(items)
    }
}

/// Spawn the daemon client task.
///
/// This function spawns a background task that handles communication with the daemon.
/// It receives commands from the UI thread via `cmd_rx` and sends responses back
/// via `msg_tx`.
///
/// # Arguments
///
/// * `url` - URL of the daemon's HTTP endpoint (e.g., "http://127.0.0.1:3030")
/// * `cmd_rx` - Receiver for commands from the UI thread
/// * `msg_tx` - Sender for messages to the UI thread
///
/// # Returns
///
/// A join handle for the spawned task
pub fn spawn_client_task(
    url: String,
    mut cmd_rx: mpsc::UnboundedReceiver<Command>,
    msg_tx: mpsc::UnboundedSender<Message>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // Try to connect to the daemon
        let client = match DaemonClient::connect(&url).await {
            Ok(client) => {
                let _ = msg_tx.send(Message::Ready);
                client
            }
            Err(e) => {
                error!("Failed to connect to daemon: {}", e);
                let _ = msg_tx.send(Message::Error(format!("Failed to connect to daemon: {}", e)));
                let _ = msg_tx.send(Message::Disconnected);
                return;
            }
        };

        // Process commands
        while let Some(cmd) = cmd_rx.recv().await {
            match cmd {
                Command::FetchStreams => {
                    match client.list_streams().await {
                        Ok(streams) => {
                            let _ = msg_tx.send(Message::StreamsLoaded(streams));
                        }
                        Err(e) => {
                            error!("Failed to fetch streams: {}", e);
                            let _ = msg_tx.send(Message::Error(format!("Failed to fetch streams: {}", e)));
                        }
                    }
                }
                Command::FetchItems(stream_id) => {
                    match client.list_items(&stream_id).await {
                        Ok(items) => {
                            let _ = msg_tx.send(Message::ItemsLoaded(items));
                        }
                        Err(e) => {
                            error!("Failed to fetch items: {}", e);
                            let _ = msg_tx.send(Message::Error(format!("Failed to fetch items: {}", e)));
                        }
                    }
                }
                Command::Shutdown => {
                    info!("Shutting down daemon client");
                    break;
                }
            }
        }

        let _ = msg_tx.send(Message::Disconnected);
    })
}

/// Get the default daemon URL.
pub fn get_daemon_url() -> String {
    "http://127.0.0.1:3030".to_string()
}
