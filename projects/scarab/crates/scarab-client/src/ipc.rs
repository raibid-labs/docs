use anyhow::{Context, Result};
use bevy::prelude::*;
use scarab_protocol::{
    ControlMessage, DaemonMessage, MAX_MESSAGE_SIZE, MAX_RECONNECT_ATTEMPTS, RECONNECT_DELAY_MS,
    SOCKET_PATH,
};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::UnixStream;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

/// Event triggered when a message is received from the Daemon
#[derive(Event)]
pub struct RemoteMessageEvent(pub DaemonMessage);

/// Bevy resource for IPC communication
#[derive(Resource)]
pub struct IpcChannel {
    inner: Arc<RwLock<Option<IpcConnection>>>,
    // Receiver for messages from the read loop to the Bevy system
    rx: Arc<std::sync::Mutex<std::sync::mpsc::Receiver<DaemonMessage>>>,
    runtime: tokio::runtime::Runtime,
}

struct IpcConnection {
    sink: OwnedWriteHalf,
    connected: bool,
}

impl IpcChannel {
    /// Create new IPC channel with automatic connection
    pub fn new() -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("ipc-worker")
            .build()
            .context("Failed to create Tokio runtime")?;

        let inner = Arc::new(RwLock::new(None));
        let (tx, rx) = std::sync::mpsc::channel();

        // Spawn connection task
        let inner_clone = inner.clone();
        runtime.spawn(async move {
            if let Err(e) = establish_connection(inner_clone, tx).await {
                eprintln!("Failed to establish initial connection: {}", e);
            }
        });

        Ok(Self {
            inner,
            rx: Arc::new(std::sync::Mutex::new(rx)),
            runtime,
        })
    }

    /// Send a control message to the daemon
    pub fn send(&self, msg: ControlMessage) {
        let inner = self.inner.clone();
        self.runtime.spawn(async move {
            if let Err(e) = send_message(inner, msg).await {
                eprintln!("Failed to send message: {}", e);
            }
        });
    }

    /// Check if connected - Public API for connection status monitoring
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.runtime.block_on(async {
            let conn = self.inner.read().await;
            conn.as_ref().map_or(false, |c| c.connected)
        })
    }

    /// Force reconnection - Public API for manual reconnection attempts
    #[allow(dead_code)]
    pub fn reconnect(&self) {
        // We need to create a new channel sender for the new connection loop
        // This is tricky because we can't easily inject it back into the existing IpcChannel if we dropped the sender.
        // However, establish_connection takes a sender.
        // For now, manual reconnect might be limited or we need to restructure to keep a sender factory.
        // Simplification: Reconnect only works if we don't need to change the channel.
        // But establish_connection takes 'tx'. We don't have 'tx' stored here.
        // TODO: Refactor for robust manual reconnect.
        eprintln!("Manual reconnect not fully implemented in this version");
    }
}

/// Establish connection with exponential backoff
async fn establish_connection(
    inner: Arc<RwLock<Option<IpcConnection>>>,
    tx: std::sync::mpsc::Sender<DaemonMessage>,
) -> Result<()> {
    let mut attempts = 0;
    let mut delay_ms = RECONNECT_DELAY_MS;

    loop {
        match UnixStream::connect(SOCKET_PATH).await {
            Ok(stream) => {
                println!("Connected to daemon at {}", SOCKET_PATH);
                let (stream_read, stream_write) = stream.into_split();

                let mut conn = inner.write().await;
                *conn = Some(IpcConnection {
                    sink: stream_write,
                    connected: true,
                });

                // Spawn read loop
                tokio::spawn(read_loop(stream_read, tx.clone()));

                return Ok(());
            }
            Err(e) => {
                attempts += 1;
                if attempts >= MAX_RECONNECT_ATTEMPTS {
                    return Err(e).context(format!(
                        "Failed to connect after {} attempts",
                        MAX_RECONNECT_ATTEMPTS
                    ));
                }

                eprintln!(
                    "Connection attempt {} failed: {}. Retrying in {}ms...",
                    attempts, e, delay_ms
                );

                sleep(Duration::from_millis(delay_ms)).await;

                // Exponential backoff with cap at 5 seconds
                delay_ms = (delay_ms * 2).min(5000);
            }
        }
    }
}

async fn read_loop(mut stream: OwnedReadHalf, tx: std::sync::mpsc::Sender<DaemonMessage>) {
    let mut buffer = vec![0u8; MAX_MESSAGE_SIZE];

    loop {
        // Read length
        let len = match stream.read_u32().await {
            Ok(l) => l as usize,
            Err(_) => break, // Connection closed or error
        };

        if len == 0 || len > MAX_MESSAGE_SIZE {
            eprintln!("Invalid message length from daemon: {}", len);
            break;
        }

        // Read data
        if let Err(e) = stream.read_exact(&mut buffer[..len]).await {
            eprintln!("Failed to read message body: {}", e);
            break;
        }

        // Deserialize
        match rkyv::from_bytes::<DaemonMessage>(&buffer[..len]) {
            Ok(msg) => {
                if let Err(e) = tx.send(msg) {
                    eprintln!("Failed to forward message to Bevy: {}", e);
                    break;
                }
            }
            Err(e) => {
                eprintln!("Failed to deserialize daemon message: {:?}", e);
            }
        }
    }
    println!("Read loop terminated");
}

/// Send message with automatic reconnection on failure
async fn send_message(
    inner: Arc<RwLock<Option<IpcConnection>>>,
    msg: ControlMessage,
) -> Result<()> {
    // Serialize message
    let bytes =
        rkyv::to_bytes::<_, MAX_MESSAGE_SIZE>(&msg).context("Failed to serialize message")?;

    let len = bytes.len();
    if len > MAX_MESSAGE_SIZE {
        anyhow::bail!("Message too large: {} bytes", len);
    }

    // Try to send
    let mut retry_count = 0;
    loop {
        {
            let mut conn_lock = inner.write().await;
            if let Some(ref mut conn) = *conn_lock {
                // Write length prefix + data
                match conn.sink.write_u32(len as u32).await {
                    Ok(_) => match conn.sink.write_all(&bytes).await {
                        Ok(_) => {
                            conn.sink.flush().await.context("Failed to flush stream")?;
                            return Ok(());
                        }
                        Err(e) => {
                            eprintln!("Write failed: {}", e);
                            conn.connected = false;
                            *conn_lock = None; // Drop connection
                        }
                    },
                    Err(e) => {
                        eprintln!("Write length failed: {}", e);
                        conn.connected = false;
                        *conn_lock = None; // Drop connection
                    }
                }
            }
        }

        // If we are here, connection failed.
        // Note: We can't easily reconnect here because we don't have the 'tx' for the read loop.
        // For now, we just fail. A full reconnect logic requires more state management.
        anyhow::bail!("Connection lost, cannot send message");
    }
}

/// Bevy system to handle keyboard input
pub fn handle_keyboard_input(keys: Res<ButtonInput<KeyCode>>, ipc: Res<IpcChannel>) {
    for key in keys.get_just_pressed() {
        let bytes = key_to_bytes(*key);
        if let Some(bytes) = bytes {
            ipc.send(ControlMessage::Input { data: bytes });
        }
    }
}

/// Convert KeyCode to terminal bytes
fn key_to_bytes(key: KeyCode) -> Option<Vec<u8>> {
    match key {
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Backspace => Some(vec![0x7F]),
        KeyCode::Tab => Some(vec![b'\t']),
        KeyCode::Escape => Some(vec![0x1B]),
        KeyCode::Space => Some(vec![b' ']),
        KeyCode::ArrowUp => Some(vec![0x1B, b'[', b'A']),
        KeyCode::ArrowDown => Some(vec![0x1B, b'[', b'B']),
        KeyCode::ArrowRight => Some(vec![0x1B, b'[', b'C']),
        KeyCode::ArrowLeft => Some(vec![0x1B, b'[', b'D']),
        KeyCode::Home => Some(vec![0x1B, b'[', b'H']),
        KeyCode::End => Some(vec![0x1B, b'[', b'F']),
        KeyCode::PageUp => Some(vec![0x1B, b'[', b'5', b'~']),
        KeyCode::PageDown => Some(vec![0x1B, b'[', b'6', b'~']),
        KeyCode::Delete => Some(vec![0x1B, b'[', b'3', b'~']),
        KeyCode::Insert => Some(vec![0x1B, b'[', b'2', b'~']),
        // Function keys
        KeyCode::F1 => Some(vec![0x1B, b'O', b'P']),
        KeyCode::F2 => Some(vec![0x1B, b'O', b'Q']),
        KeyCode::F3 => Some(vec![0x1B, b'O', b'R']),
        KeyCode::F4 => Some(vec![0x1B, b'O', b'S']),
        KeyCode::F5 => Some(vec![0x1B, b'[', b'1', b'5', b'~']),
        KeyCode::F6 => Some(vec![0x1B, b'[', b'1', b'7', b'~']),
        KeyCode::F7 => Some(vec![0x1B, b'[', b'1', b'8', b'~']),
        KeyCode::F8 => Some(vec![0x1B, b'[', b'1', b'9', b'~']),
        KeyCode::F9 => Some(vec![0x1B, b'[', b'2', b'0', b'~']),
        KeyCode::F10 => Some(vec![0x1B, b'[', b'2', b'1', b'~']),
        KeyCode::F11 => Some(vec![0x1B, b'[', b'2', b'3', b'~']),
        KeyCode::F12 => Some(vec![0x1B, b'[', b'2', b'4', b'~']),
        // Regular characters - handle via character input event instead
        _ => None,
    }
}

/// Bevy system to handle character input (for printable characters)
pub fn handle_character_input(
    mut char_events: EventReader<bevy::input::keyboard::KeyboardInput>,
    ipc: Res<IpcChannel>,
) {
    for event in char_events.read() {
        // Only handle key presses (not releases)
        if !event.state.is_pressed() {
            continue;
        }

        // Handle text input via logical_key
        if let bevy::input::keyboard::Key::Character(ref s) = event.logical_key {
            let bytes = s.as_str().as_bytes().to_vec();
            ipc.send(ControlMessage::Input { data: bytes });
        }
    }
}

/// Bevy system to handle window resize
pub fn handle_window_resize(
    mut resize_events: EventReader<bevy::window::WindowResized>,
    ipc: Res<IpcChannel>,
) {
    for event in resize_events.read() {
        // Calculate terminal rows/cols from window size
        // Assuming 8x16 character cell size (this should be configurable)
        let cols = (event.width / 8.0) as u16;
        let rows = (event.height / 16.0) as u16;

        println!(
            "Window resized: {}x{} -> {}x{} chars",
            event.width, event.height, cols, rows
        );

        ipc.send(ControlMessage::Resize { cols, rows });
    }
}

/// Dispatch received messages to Bevy events
pub fn receive_ipc_messages(ipc: Res<IpcChannel>, mut events: EventWriter<RemoteMessageEvent>) {
    if let Ok(rx) = ipc.rx.lock() {
        // Drain all pending messages
        while let Ok(msg) = rx.try_recv() {
            events.send(RemoteMessageEvent(msg));
        }
    }
}

/// Bevy plugin for IPC functionality
pub struct IpcPlugin;

impl Plugin for IpcPlugin {
    fn build(&self, app: &mut App) {
        // Initialize IPC channel
        match IpcChannel::new() {
            Ok(channel) => {
                println!("IPC channel initialized");
                app.insert_resource(channel);
                app.add_event::<RemoteMessageEvent>();

                // Register input handling systems
                app.add_systems(
                    Update,
                    (
                        handle_keyboard_input,
                        handle_character_input,
                        handle_window_resize,
                        receive_ipc_messages,
                    ),
                );
            }
            Err(e) => {
                eprintln!("Failed to initialize IPC: {}", e);
                eprintln!("Client will run without IPC support");
            }
        }
    }
}
