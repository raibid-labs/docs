use async_trait::async_trait;
use regex::Regex;
use scarab_plugin_api::{
    types::{OverlayStyle, RemoteCommand},
    Action, Plugin, PluginContext, PluginMetadata, Result,
};
use scarab_nav_protocol::v1::UpdateLayout;
use std::sync::{Arc, Mutex};
use tokio::net::UnixListener;
use tokio::io::AsyncReadExt;
use prost::Message;

pub struct NavigationPlugin {
    metadata: PluginMetadata,
    state: Arc<Mutex<PluginState>>,
    url_regex: Regex,
}

#[derive(Default)]
struct PluginState {
    active: bool,
    hints: Vec<Hint>,
    input_buffer: String,
    latest_layout: Option<UpdateLayout>,
}

struct Hint {
    #[allow(dead_code)]
    id: u64,
    label: String,
    action: HintAction,
}

enum HintAction {
    OpenUrl(String),
    Click(u32, u32), // x, y
}

impl NavigationPlugin {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(PluginState::default()));
        let state_clone = state.clone();

        // Start the socket listener in a background task
        // We use a standard path for now. In a real integration, the daemon would 
        // likely pass a unique socket path or FD to the plugin.
        // For this MVP, we'll use /tmp/scarab-nav.sock
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let socket_path = "/tmp/scarab-nav.sock";
                // Remove existing socket if it exists
                let _ = tokio::fs::remove_file(socket_path).await;
                
                match UnixListener::bind(socket_path) {
                    Ok(listener) => {
                        log::info!("Scarab Nav listening on {}", socket_path);
                        loop {
                            match listener.accept().await {
                                Ok((mut stream, _addr)) => {
                                    let state_ref = state_clone.clone();
                                    tokio::spawn(async move {
                                        // Simple framing: u32 length prefix + protobuf body
                                        loop {
                                            let mut len_buf = [0u8; 4];
                                            if stream.read_exact(&mut len_buf).await.is_err() {
                                                break;
                                            }
                                            let len = u32::from_le_bytes(len_buf) as usize;
                                            let mut buf = vec![0u8; len];
                                            if stream.read_exact(&mut buf).await.is_err() {
                                                break;
                                            }
                                            
                                            if let Ok(layout) = UpdateLayout::decode(&buf[..]) {
                                                if let Ok(mut s) = state_ref.lock() {
                                                    s.latest_layout = Some(layout);
                                                }
                                            }
                                        }
                                    });
                                }
                                Err(e) => log::error!("Failed to accept connection: {}", e),
                            }
                        }
                    }
                    Err(e) => log::error!("Failed to bind to {}: {}", socket_path, e),
                }
            });
        });

        Self {
            metadata: PluginMetadata::new(
                "scarab-nav",
                "0.2.0",
                "Keyboard navigation with protocol support",
                "Scarab Team",
            ),
            state,
            url_regex: Regex::new(r"https?://[^\s\)]+").unwrap(),
        }
    }

    fn generate_labels(count: usize) -> Vec<String> {
        let chars: Vec<char> = "asdfghjklqwertyuiopzxcvbnm".chars().collect();
        let mut labels = Vec::new();
        
        if count <= chars.len() {
            for i in 0..count {
                labels.push(chars[i].to_string());
            }
        } else {
            // Two-char labels
            for c1 in &chars {
                for c2 in &chars {
                    labels.push(format!("{}{}", c1, c2));
                    if labels.len() >= count {
                        return labels;
                    }
                }
            }
        }
        labels
    }

    /// Open a URL in the default browser
    fn open_url(url: &str) -> std::result::Result<(), String> {
        #[cfg(target_os = "linux")]
        let cmd = "xdg-open";
        #[cfg(target_os = "macos")]
        let cmd = "open";
        #[cfg(target_os = "windows")]
        let cmd = "start"; // roughly

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        return Err("Unsupported platform".into());

        std::process::Command::new(cmd)
            .arg(url)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[async_trait]
impl Plugin for NavigationPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        let mut state = self.state.lock().unwrap();

        // Trigger: Alt+f (Esc + f) -> [0x1b, b'f']
        // Or Ctrl+f -> [0x06]
        
        if !state.active {
            // Activation check
            if input == [0x1b, b'f'] || input == [0x06] {
                log::info!("Activating Navigation");
                state.active = true;
                state.hints.clear();
                state.input_buffer.clear();

                let mut hints_to_generate = Vec::new();

                // 1. Check Protocol Layout
                if let Some(layout) = &state.latest_layout {
                    for element in &layout.elements {
                        // Simple check: is element on screen?
                        // The layout coords are 0-based relative to window.
                        // We assume window 0,0 maps to terminal 0,0 for this MVP.
                        hints_to_generate.push((
                            element.x as u16, 
                            element.y as u16, 
                            HintAction::Click(element.x + element.width / 2, element.y + element.height / 2)
                        ));
                    }
                }

                // 2. Fallback / Augment with Regex (URL detection)
                let (_cols, rows) = ctx.get_size();
                for y in 0..rows {
                    if let Some(line) = ctx.get_line(y) {
                        for mat in self.url_regex.find_iter(&line) {
                            hints_to_generate.push((
                                mat.start() as u16,
                                y,
                                HintAction::OpenUrl(mat.as_str().to_string())
                            ));
                        }
                    }
                }

                let labels = Self::generate_labels(hints_to_generate.len());
                let mut hint_id = 0;

                for (i, (x, y, action)) in hints_to_generate.into_iter().enumerate() {
                    if i >= labels.len() { break; }
                    
                    let label = labels[i].clone();
                    let id = hint_id;
                    hint_id += 1;

                    state.hints.push(Hint {
                        id,
                        label: label.clone(),
                        action,
                    });

                    ctx.queue_command(RemoteCommand::DrawOverlay {
                        id,
                        x,
                        y,
                        text: label,
                        style: OverlayStyle {
                            fg: 0xFFFFFFFF,
                            bg: 0xFF0000FF, // Red
                            z_index: 100.0,
                        },
                    });
                }

                return Ok(Action::Modify(Vec::new())); // Consume trigger
            }
            
            return Ok(Action::Continue);
        } else {
            // Handling input while active
            
            // Esc -> Cancel
            if input == [0x1b] {
                state.active = false;
                ctx.queue_command(RemoteCommand::ClearOverlays { id: None });
                return Ok(Action::Modify(Vec::new()));
            }

            // Typed char
            if let Ok(s) = std::str::from_utf8(input) {
                // We handle single char for now, or build buffer if multi-char labels
                // Assuming labels are generated alphabetically, we can just append to buffer
                state.input_buffer.push_str(s);
                
                // Check for exact match
                if let Some(hint) = state.hints.iter().find(|h| h.label == state.input_buffer) {
                    // Execute Action
                    match &hint.action {
                        HintAction::OpenUrl(url) => {
                            let _ = Self::open_url(url);
                            ctx.notify_success("URL Opened", url);
                        }
                        HintAction::Click(x, y) => {
                            // Generate xterm mouse click sequence (SGR 1006)
                            // \x1b[<0;x;yM (press) \x1b[<0;x;ym (release)
                            // Coordinates are 1-based
                            let x = x + 1;
                            let y = y + 1;
                            let seq = format!("\x1b[<0;{};{}M\x1b[<0;{};{}m", x, y, x, y);
                            
                            state.active = false;
                            ctx.queue_command(RemoteCommand::ClearOverlays { id: None });
                            return Ok(Action::Modify(seq.into_bytes()));
                        }
                    }

                    state.active = false;
                    ctx.queue_command(RemoteCommand::ClearOverlays { id: None });
                    return Ok(Action::Modify(Vec::new()));
                }
                
                // Partial match check? If buffer doesn't match start of any label, reset or cancel.
                let has_prefix = state.hints.iter().any(|h| h.label.starts_with(&state.input_buffer));
                if !has_prefix {
                    // Invalid input, deactivate and pass through to terminal
                    state.active = false;
                    state.input_buffer.clear();
                    ctx.queue_command(RemoteCommand::ClearOverlays { id: None });
                    return Ok(Action::Continue);
                }
            }

            return Ok(Action::Modify(Vec::new()));
        }
    }
}
