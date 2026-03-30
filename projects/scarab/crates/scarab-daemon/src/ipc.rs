use crate::plugin_manager::PluginManager;
use crate::session::{handle_session_command, SessionManager};
use anyhow::{Context, Result};
use portable_pty::PtySize;
use scarab_protocol::{ControlMessage, DaemonMessage, PluginInspectorInfo, MAX_CLIENTS, MAX_MESSAGE_SIZE, SOCKET_PATH};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::OwnedWriteHalf;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, Mutex, RwLock};

/// Helper for defer logic (since we don't have a crate for it)
macro_rules! defer {
    ( $($code:tt)* ) => {
        struct Defer<F: FnOnce()>(Option<F>);
        impl<F: FnOnce()> Drop for Defer<F> {
            fn drop(&mut self) {
                if let Some(f) = self.0.take() {
                    f();
                }
            }
        }
        let _defer = Defer(Some(|| { $($code)* }));
    }
}

/// Handle to send commands to PTY
/// Using channels for thread-safe communication
#[derive(Clone)]
pub struct PtyHandle {
    input_tx: mpsc::UnboundedSender<Vec<u8>>,
    resize_tx: mpsc::Sender<PtySize>,
}

impl PtyHandle {
    pub fn new(input_tx: mpsc::UnboundedSender<Vec<u8>>, resize_tx: mpsc::Sender<PtySize>) -> Self {
        Self {
            input_tx,
            resize_tx,
        }
    }

    pub async fn write_input(&self, data: &[u8]) -> Result<()> {
        self.input_tx
            .send(data.to_vec())
            .context("Failed to send input to PTY channel")?;
        Ok(())
    }

    pub async fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.resize_tx
            .send(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .await
            .context("Failed to send resize event to PTY")?;
        Ok(())
    }
}

/// Thread-safe handle for sending messages to a specific client
#[derive(Clone)]
pub struct ClientSender {
    sink: Arc<Mutex<OwnedWriteHalf>>,
}

impl ClientSender {
    pub fn new(sink: OwnedWriteHalf) -> Self {
        Self {
            sink: Arc::new(Mutex::new(sink)),
        }
    }

    pub async fn send(&self, msg: DaemonMessage) -> Result<()> {
        let bytes =
            rkyv::to_bytes::<_, MAX_MESSAGE_SIZE>(&msg).context("Failed to serialize message")?;
        let len = bytes.len() as u32;

        let mut sink = self.sink.lock().await;
        sink.write_u32(len).await?;
        sink.write_all(&bytes).await?;
        sink.flush().await?;
        Ok(())
    }
}

/// Registry of active client connections
#[derive(Clone)]
pub struct ClientRegistry {
    clients: Arc<RwLock<HashMap<u64, ClientSender>>>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, id: u64, sender: ClientSender) {
        let mut map = self.clients.write().await;
        map.insert(id, sender);
    }

    pub async fn unregister(&self, id: u64) {
        let mut map = self.clients.write().await;
        map.remove(&id);
    }

    pub async fn send(&self, id: u64, msg: DaemonMessage) -> Result<()> {
        let map = self.clients.read().await;
        if let Some(sender) = map.get(&id) {
            sender.send(msg).await?;
            Ok(())
        } else {
            anyhow::bail!("Client {} not found", id);
        }
    }

    pub async fn broadcast(&self, msg: DaemonMessage) {
        let map = self.clients.read().await;
        for (id, sender) in map.iter() {
            if let Err(e) = sender.send(msg.clone()).await {
                eprintln!("Failed to broadcast to client {}: {}", id, e);
            }
        }
    }
}

/// IPC server managing multiple client connections
pub struct IpcServer {
    listener: UnixListener,
    pty_handle: PtyHandle,
    session_manager: Arc<SessionManager>,
    plugin_manager: Arc<Mutex<PluginManager>>,
    client_registry: ClientRegistry,
    client_counter: Arc<RwLock<u64>>,
}

impl IpcServer {
    /// Create new IPC server, removing stale socket if exists
    pub async fn new(
        pty_handle: PtyHandle,
        session_manager: Arc<SessionManager>,
        client_registry: ClientRegistry,
        plugin_manager: Arc<Mutex<PluginManager>>,
    ) -> Result<Self> {
        // Remove existing socket if present
        if Path::new(SOCKET_PATH).exists() {
            std::fs::remove_file(SOCKET_PATH).context("Failed to remove stale socket")?;
        }

        let listener = UnixListener::bind(SOCKET_PATH).context("Failed to bind Unix socket")?;

        // Set socket permissions to 700 (owner only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(SOCKET_PATH, std::fs::Permissions::from_mode(0o700))
                .context("Failed to set socket permissions")?;
        }

        println!("IPC server listening on: {}", SOCKET_PATH);

        Ok(Self {
            listener,
            pty_handle,
            session_manager,
            plugin_manager,
            client_registry,
            client_counter: Arc::new(RwLock::new(0)),
        })
    }

    /// Accept client connections in a loop
    pub async fn accept_loop(self) -> Result<()> {
        let active_clients = Arc::new(RwLock::new(0usize));

        loop {
            match self.listener.accept().await {
                Ok((stream, _addr)) => {
                    let client_count = {
                        let mut count = active_clients.write().await;
                        *count += 1;
                        *count
                    };

                    if client_count > MAX_CLIENTS {
                        eprintln!(
                            "Max clients ({}) reached, rejecting connection",
                            MAX_CLIENTS
                        );
                        let mut count = active_clients.write().await;
                        *count -= 1;
                        continue;
                    }

                    let client_id = {
                        let mut counter = self.client_counter.write().await;
                        *counter += 1;
                        *counter
                    };

                    println!("Client {} connected (active: {})", client_id, client_count);

                    let pty_handle = self.pty_handle.clone();
                    let session_manager = self.session_manager.clone();
                    let client_registry = self.client_registry.clone();
                    let plugin_manager = self.plugin_manager.clone();
                    let active_clients = active_clients.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_client(
                            stream,
                            client_id,
                            pty_handle,
                            session_manager,
                            client_registry,
                            plugin_manager,
                        )
                        .await
                        {
                            eprintln!("Client {} error: {}", client_id, e);
                        }

                        let mut count = active_clients.write().await;
                        *count -= 1;
                        println!("Client {} disconnected (active: {})", client_id, *count);
                    });
                }
                Err(e) => {
                    eprintln!("Failed to accept client: {}", e);
                }
            }
        }
    }
}

/// Handle individual client connection
async fn handle_client(
    stream: UnixStream, // Takes ownership
    client_id: u64,
    pty_handle: PtyHandle,
    session_manager: Arc<SessionManager>,
    client_registry: ClientRegistry,
    plugin_manager: Arc<Mutex<PluginManager>>,
) -> Result<()> {
    let (mut stream_read, stream_write) = stream.into_split();
    let mut buffer = vec![0u8; MAX_MESSAGE_SIZE];

    // Register client for writing
    let sender = ClientSender::new(stream_write);
    client_registry.register(client_id, sender).await;

    // Ensure cleanup on exit
    let registry_clone = client_registry.clone();
    defer! {
        tokio::spawn(async move {
            registry_clone.unregister(client_id).await;
        });
    }

    // Reading loop
    loop {
        // Read message length prefix (4 bytes)
        let len = match stream_read.read_u32().await {
            Ok(len) => len as usize,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // Client disconnected gracefully
                break;
            }
            Err(e) => {
                return Err(e).context("Failed to read message length");
            }
        };

        if len == 0 || len > MAX_MESSAGE_SIZE {
            anyhow::bail!("Invalid message length: {}", len);
        }

        // Read message data
        stream_read
            .read_exact(&mut buffer[..len])
            .await
            .context("Failed to read message data")?;

        // Deserialize with rkyv
        let msg = match rkyv::from_bytes::<ControlMessage>(&buffer[..len]) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to deserialize ControlMessage: {:?}", e);
                anyhow::bail!("Deserialization error");
            }
        };

        // Handle message
        if let Err(e) = handle_message(
            msg,
            &pty_handle,
            &session_manager,
            &plugin_manager,
            &client_registry,
            client_id,
        )
        .await
        {
            eprintln!("Client {} message handling error: {}", client_id, e);
            // Don't disconnect on individual message errors
        }
    }

    Ok(())
}

/// Process a control message
async fn handle_message(
    msg: ControlMessage,
    pty_handle: &PtyHandle,
    session_manager: &Arc<SessionManager>,
    plugin_manager: &Arc<Mutex<PluginManager>>,
    client_registry: &ClientRegistry,
    client_id: u64,
) -> Result<()> {
    // Try to handle as session command first
    if let Ok(Some(response)) =
        handle_session_command(msg.clone(), session_manager, client_id).await
    {
        log::info!("Session command response: {:?}", response);
        // Send response back to client
        client_registry
            .send(client_id, DaemonMessage::Session(response))
            .await?;
        return Ok(());
    }

    // Handle non-session commands
    match msg {
        ControlMessage::Resize { cols, rows } => {
            log::info!("Client {} resize: {}x{}", client_id, cols, rows);
            pty_handle.resize(cols, rows).await?;
        }
        ControlMessage::Input { data } => {
            // Validate input size to prevent abuse
            if data.len() > MAX_MESSAGE_SIZE {
                anyhow::bail!("Input data too large: {} bytes", data.len());
            }
            pty_handle.write_input(&data).await?;
        }
        ControlMessage::LoadPlugin { path } => {
            log::info!("Client {} loading plugin: {}", client_id, path);

            let path_str = path.to_string();
            let plugin_manager = plugin_manager.clone();

            // Load plugin asynchronously
            match std::path::PathBuf::from(&path_str).canonicalize() {
                Ok(abs_path) => {
                    let mut pm = plugin_manager.lock().await;

                    // Create minimal plugin config
                    let config = scarab_plugin_api::PluginConfig {
                        name: abs_path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        path: abs_path.clone(),
                        enabled: true,
                        config: Default::default(),
                    };

                    let plugin_name = config.name.clone(); // Clone name before moving config

                    // Load the plugin
                    match pm.load_plugin_from_config(config).await {
                        Ok(_) => {
                            log::info!("Successfully loaded plugin from: {:?}", abs_path);

                            // Send updated plugin list to all clients
                            let plugins = pm.list_plugins();
                            let plugin_infos: Vec<PluginInspectorInfo> = plugins
                                .into_iter()
                                .map(|p| PluginInspectorInfo {
                                    name: p.name.clone().into(),
                                    version: p.version.clone().into(),
                                    description: p.description.clone().into(),
                                    author: p.author.clone().into(),
                                    homepage: p.homepage.clone().map(|s| s.into()),
                                    api_version: p.api_version.clone().into(),
                                    min_scarab_version: p.min_scarab_version.clone().into(),
                                    enabled: p.enabled,
                                    failure_count: p.failure_count,
                                })
                                .collect();

                            client_registry
                                .broadcast(DaemonMessage::PluginList { plugins: plugin_infos })
                                .await;
                        }
                        Err(e) => {
                            log::error!("Failed to load plugin: {}", e);
                            client_registry
                                .send(
                                    client_id,
                                    DaemonMessage::PluginError {
                                        name: plugin_name.into(),
                                        error: format!("Failed to load plugin: {}", e).into(),
                                    },
                                )
                                .await?;
                        }
                    }
                }
                Err(e) => {
                    log::error!("Invalid plugin path '{}': {}", path_str, e);
                    client_registry
                        .send(
                            client_id,
                            DaemonMessage::PluginError {
                                name: path_str.clone().into(),
                                error: format!("Invalid path: {}", e).into(),
                            },
                        )
                        .await?;
                }
            }
        }
        ControlMessage::Ping { timestamp } => {
            log::debug!("Client {} ping: {}", client_id, timestamp);
            // Could respond with pong if bidirectional communication is needed
        }
        ControlMessage::Disconnect { client_id: id } => {
            log::info!("Client {} requesting disconnect", id);
            // Client will disconnect when this function returns
        }
        ControlMessage::CommandSelected { id } => {
            log::info!("Client {} selected command: {}", client_id, id);
            let mut pm = plugin_manager.lock().await;
            if let Err(e) = pm.dispatch_remote_command(&id).await {
                log::error!("Failed to dispatch remote command: {}", e);
            }
        }
        ControlMessage::PluginListRequest => {
            log::info!("Client {} requesting plugin list", client_id);

            let pm = plugin_manager.lock().await;
            let plugins = pm.list_plugins();

            // Convert to protocol-compatible format
            let plugin_infos: Vec<PluginInspectorInfo> = plugins
                .into_iter()
                .map(|p| PluginInspectorInfo {
                    name: p.name.clone().into(),
                    version: p.version.clone().into(),
                    description: p.description.clone().into(),
                    author: p.author.clone().into(),
                    homepage: p.homepage.clone().map(|s| s.into()),
                    api_version: p.api_version.clone().into(),
                    min_scarab_version: p.min_scarab_version.clone().into(),
                    enabled: p.enabled,
                    failure_count: p.failure_count,
                })
                .collect();

            log::debug!("Sending plugin list with {} plugins", plugin_infos.len());

            // Send response to requesting client
            client_registry
                .send(client_id, DaemonMessage::PluginList { plugins: plugin_infos })
                .await?;
        }
        ControlMessage::PluginEnable { name } => {
            log::info!("Client {} enabling plugin: {}", client_id, name);

            let mut pm = plugin_manager.lock().await;

            // Find the plugin by name
            if let Some(plugin) = pm.plugins.iter_mut().find(|p| p.plugin.metadata().name == name.as_str()) {
                if plugin.enabled {
                    log::warn!("Plugin '{}' is already enabled", name);
                } else {
                    plugin.enabled = true;
                    plugin.failure_count = 0; // Reset failures on enable
                    log::info!("Plugin '{}' enabled", name);

                    // Refresh commands list since plugin is now active
                    pm.refresh_commands();

                    // Notify all clients of status change
                    client_registry
                        .broadcast(DaemonMessage::PluginStatusChanged {
                            name: name.clone(),
                            enabled: true,
                        })
                        .await;
                }
            } else {
                log::error!("Plugin '{}' not found", name);
                client_registry
                    .send(
                        client_id,
                        DaemonMessage::PluginError {
                            name: name.clone(),
                            error: "Plugin not found".to_string().into(),
                        },
                    )
                    .await?;
            }
        }
        ControlMessage::PluginDisable { name } => {
            log::info!("Client {} disabling plugin: {}", client_id, name);

            let mut pm = plugin_manager.lock().await;

            // Find the plugin by name
            if let Some(plugin) = pm.plugins.iter_mut().find(|p| p.plugin.metadata().name == name.as_str()) {
                if !plugin.enabled {
                    log::warn!("Plugin '{}' is already disabled", name);
                } else {
                    plugin.enabled = false;
                    log::info!("Plugin '{}' disabled", name);

                    // Refresh commands list since plugin is now inactive
                    pm.refresh_commands();

                    // Notify all clients of status change
                    client_registry
                        .broadcast(DaemonMessage::PluginStatusChanged {
                            name: name.clone(),
                            enabled: false,
                        })
                        .await;
                }
            } else {
                log::error!("Plugin '{}' not found", name);
                client_registry
                    .send(
                        client_id,
                        DaemonMessage::PluginError {
                            name: name.clone(),
                            error: "Plugin not found".to_string().into(),
                        },
                    )
                    .await?;
            }
        }
        ControlMessage::PluginReload { name } => {
            log::info!("Client {} reloading plugin: {}", client_id, name);

            let plugin_manager = plugin_manager.clone();
            let mut pm = plugin_manager.lock().await;

            // Find the plugin and get its config path
            let plugin_config = pm.plugins.iter().find(|p| p.plugin.metadata().name == name.as_str())
                .map(|p| p.config.clone());

            if let Some(config) = plugin_config {
                // Unload the plugin
                let plugin_idx = pm.plugins.iter().position(|p| p.plugin.metadata().name == name.as_str());

                if let Some(idx) = plugin_idx {
                    let mut managed = pm.plugins.remove(idx); // Make it mutable

                    log::debug!("Unloading plugin '{}'", name);

                    // Call on_unload with timeout
                    if let Err(e) = tokio::time::timeout(
                        pm.hook_timeout,
                        managed.plugin.on_unload()
                    ).await {
                        log::warn!("Plugin '{}' unload timed out: {:?}", name, e);
                    }

                    // Reload the plugin
                    log::debug!("Reloading plugin '{}' from {:?}", name, config.path);

                    match pm.load_plugin_from_config(config.clone()).await {
                        Ok(_) => {
                            log::info!("Successfully reloaded plugin '{}'", name);

                            // Refresh commands
                            pm.refresh_commands();

                            // Send updated plugin list to all clients
                            let plugins = pm.list_plugins();
                            let plugin_infos: Vec<PluginInspectorInfo> = plugins
                                .into_iter()
                                .map(|p| PluginInspectorInfo {
                                    name: p.name.clone().into(),
                                    version: p.version.clone().into(),
                                    description: p.description.clone().into(),
                                    author: p.author.clone().into(),
                                    homepage: p.homepage.clone().map(|s| s.into()),
                                    api_version: p.api_version.clone().into(),
                                    min_scarab_version: p.min_scarab_version.clone().into(),
                                    enabled: p.enabled,
                                    failure_count: p.failure_count,
                                })
                                .collect();

                            client_registry
                                .broadcast(DaemonMessage::PluginList { plugins: plugin_infos })
                                .await;

                            // Notify status change
                            client_registry
                                .broadcast(DaemonMessage::PluginStatusChanged {
                                    name: name.clone(),
                                    enabled: true,
                                })
                                .await;
                        }
                        Err(e) => {
                            log::error!("Failed to reload plugin '{}': {}", name, e);
                            client_registry
                                .send(
                                    client_id,
                                    DaemonMessage::PluginError {
                                        name: name.clone(),
                                        error: format!("Reload failed: {}", e).into(),
                                    },
                                )
                                .await?;
                        }
                    }
                }
            } else {
                log::error!("Plugin '{}' not found", name);
                client_registry
                    .send(
                        client_id,
                        DaemonMessage::PluginError {
                            name: name.clone(),
                            error: "Plugin not found".to_string().into(),
                        },
                    )
                    .await?;
            }
        }
        // Session commands and internal messages - already handled elsewhere
        ControlMessage::SessionCreate { .. }
        | ControlMessage::SessionDelete { .. }
        | ControlMessage::SessionList
        | ControlMessage::SessionAttach { .. }
        | ControlMessage::SessionDetach { .. }
        | ControlMessage::SessionRename { .. } => {
            // Already handled by handle_session_command
        }
        // Tab management - delegate to plugins
        ControlMessage::TabCreate { title } => {
            log::info!("Client {} creating tab: {:?}", client_id, title);
            let mut pm = plugin_manager.lock().await;
            if let Err(e) = pm.dispatch_remote_command("tabs.new").await {
                log::error!("Failed to create tab: {}", e);
            }
        }
        ControlMessage::TabClose { tab_id } => {
            log::info!("Client {} closing tab {}", client_id, tab_id);
            // TODO: Implement tab closing via plugin
        }
        ControlMessage::TabSwitch { tab_id } => {
            log::info!("Client {} switching to tab {}", client_id, tab_id);
            // TODO: Implement tab switching via plugin
        }
        ControlMessage::TabRename { tab_id, new_title } => {
            log::info!("Client {} renaming tab {} to '{}'", client_id, tab_id, new_title);
            // TODO: Implement tab renaming via plugin
        }
        ControlMessage::TabList => {
            log::info!("Client {} requesting tab list", client_id);
            // TODO: Implement tab list via plugin
        }

        // Pane management - delegate to plugins
        ControlMessage::PaneSplit { pane_id, direction } => {
            log::info!("Client {} splitting pane {} {:?}", client_id, pane_id, direction);
            let mut pm = plugin_manager.lock().await;
            let command = match direction {
                scarab_protocol::SplitDirection::Horizontal => "panes.split_horizontal",
                scarab_protocol::SplitDirection::Vertical => "panes.split_vertical",
            };
            if let Err(e) = pm.dispatch_remote_command(command).await {
                log::error!("Failed to split pane: {}", e);
            }
        }
        ControlMessage::PaneClose { pane_id } => {
            log::info!("Client {} closing pane {}", client_id, pane_id);
            // TODO: Implement pane closing via plugin
        }
        ControlMessage::PaneFocus { pane_id } => {
            log::info!("Client {} focusing pane {}", client_id, pane_id);
            // TODO: Implement pane focusing via plugin
        }
        ControlMessage::PaneResize { pane_id, width, height } => {
            log::info!("Client {} resizing pane {} to {}x{}", client_id, pane_id, width, height);
            // TODO: Implement pane resizing via plugin
        }
        ControlMessage::PluginLog { .. }
        | ControlMessage::PluginNotify { .. } => {
            // These are internal messages sent BY plugins, not received FROM clients
            log::warn!("Received internal-only message from client {}", client_id);
        }
    }

    Ok(())
}

/// Cleanup socket on server shutdown
impl Drop for IpcServer {
    fn drop(&mut self) {
        if Path::new(SOCKET_PATH).exists() {
            let _ = std::fs::remove_file(SOCKET_PATH);
            println!("Cleaned up socket: {}", SOCKET_PATH);
        }
    }
}
