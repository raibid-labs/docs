use prost::Message;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};

pub use scarab_nav_protocol::v1;
use scarab_nav_protocol::v1::{ElementType, InteractiveElement, UpdateLayout};

#[derive(Debug, thiserror::Error)]
pub enum NavError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Protobuf Error: {0}")]
    Prost(#[from] prost::EncodeError),
    #[error("Channel Closed")]
    ChannelClosed,
}

/// A client for the Scarab Navigation Protocol.
/// It handles connection management and message buffering in a background task.
#[derive(Clone)]
pub struct NavClient {
    sender: mpsc::UnboundedSender<UpdateLayout>,
}

impl Default for NavClient {
    fn default() -> Self {
        Self::new()
    }
}

impl NavClient {
    /// Create a new NavClient.
    /// It attempts to connect to the socket at `SCARAB_NAV_SOCKET`.
    /// If the env var is not set, it logs a warning and returns a dummy client (no-op).
    pub fn new() -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<UpdateLayout>();

        let socket_path = std::env::var("SCARAB_NAV_SOCKET").map(PathBuf::from).ok();

        if let Some(path) = socket_path {
            tokio::spawn(async move {
                let mut stream = match UnixStream::connect(&path).await {
                    Ok(s) => s,
                    Err(e) => {
                        warn!(
                            "Failed to connect to Scarab Nav Socket at {:?}: {}",
                            path, e
                        );
                        // In a real implementation, we might want a retry loop here
                        return;
                    }
                };

                debug!("Connected to Scarab Nav Socket at {:?}", path);

                while let Some(msg) = rx.recv().await {
                    let mut buf = Vec::new();
                    // Simple length-prefixed framing? Or just raw protobuf?
                    // For a Unix socket, we usually need some framing if we send multiple messages.
                    // Let's use a simple u32 length prefix (Little Endian).

                    if let Err(e) = msg.encode(&mut buf) {
                        error!("Failed to encode UpdateLayout: {}", e);
                        continue;
                    }

                    let len = buf.len() as u32;
                    let len_bytes = len.to_le_bytes();

                    if let Err(e) = stream.write_all(&len_bytes).await {
                        error!("Failed to write message length: {}", e);
                        break; // Connection likely broken
                    }

                    if let Err(e) = stream.write_all(&buf).await {
                        error!("Failed to write message body: {}", e);
                        break;
                    }
                }
            });
        } else {
            debug!("SCARAB_NAV_SOCKET not set. Navigation updates will be ignored.");
            // Consume the channel to prevent memory leak if the user keeps sending
            tokio::spawn(async move { while (rx.recv().await).is_some() {} });
        }

        Self { sender: tx }
    }

    pub fn update(&self, layout: UpdateLayout) {
        let _ = self.sender.send(layout);
    }
}

#[cfg(feature = "ratatui")]
pub mod ratatui_helper {
    use super::*;
    // We don't import ratatui::layout::Rect to avoid version conflicts.
    // Users pass coordinates explicitly.

    /// A helper to build UpdateLayout messages.
    pub struct NavRecorder {
        elements: Vec<InteractiveElement>,
    }

    impl Default for NavRecorder {
        fn default() -> Self {
            Self::new()
        }
    }

    impl NavRecorder {
        pub fn new() -> Self {
            Self {
                elements: Vec::new(),
            }
        }

        /// Register an interactive element with its position and type.
        ///
        /// # Arguments
        /// * `x`, `y` - Top-left position of the element
        /// * `width`, `height` - Size of the element
        /// * `element_type` - Type of interactive element
        /// * `description` - Human-readable description
        /// * `key_hint` - Optional keyboard shortcut hint
        #[allow(clippy::too_many_arguments)]
        pub fn register(
            &mut self,
            x: u16,
            y: u16,
            width: u16,
            height: u16,
            element_type: ElementType,
            description: impl Into<String>,
            key_hint: Option<String>,
        ) {
            self.elements.push(InteractiveElement {
                id: uuid::Uuid::new_v4().to_string(), // Random ID for now
                x: x as u32,
                y: y as u32,
                width: width as u32,
                height: height as u32,
                r#type: element_type as i32,
                description: description.into(),
                key_hint: key_hint.unwrap_or_default(),
            });
        }

        pub fn finish(self) -> UpdateLayout {
            UpdateLayout {
                window_id: std::process::id().to_string(), // Use PID as window ID for simple correlation
                elements: self.elements,
            }
        }
    }
}
