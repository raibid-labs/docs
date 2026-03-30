//! Mouse Support Plugin for Scarab Terminal
//!
//! Provides comprehensive mouse interaction support including:
//! - Click to position cursor
//! - Drag to select text
//! - Right-click context menu
//! - Scroll wheel navigation
//! - URL/file opening with Ctrl+Click
//! - Mouse mode detection (normal vs application mode)
//!
//! Integrates with clipboard for selection operations.

pub mod bevy_plugin;
pub mod click_handler;
pub mod context_menu;
pub mod mode;
pub mod selection;
pub mod types;

pub use bevy_plugin::MousePlugin as BevyMousePlugin;
pub use types::{ClickType, MouseButton, MouseEvent, MouseMode, Position};

use async_trait::async_trait;
use parking_lot::Mutex;
use scarab_plugin_api::{
    types::ModalItem,
    Action, Plugin, PluginContext, PluginMetadata, Result,
};
use std::sync::Arc;

/// Main mouse support plugin
pub struct MousePlugin {
    metadata: PluginMetadata,
    state: Arc<Mutex<MouseState>>,
}

/// Internal plugin state
pub struct MouseState {
    /// Current mouse mode
    pub mode: MouseMode,
    /// Current selection if any
    pub selection: Option<selection::Selection>,
    /// Last click position and time for double/triple-click detection
    pub last_click: Option<(Position, std::time::Instant, ClickType)>,
    /// Whether context menu is currently shown
    pub context_menu_visible: bool,
    /// Detected URLs and file paths in visible area
    pub clickable_items: Vec<ClickableItem>,
}

#[derive(Debug, Clone)]
pub struct ClickableItem {
    pub kind: ClickableKind,
    pub text: String,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickableKind {
    Url,
    FilePath,
}

impl Default for MouseState {
    fn default() -> Self {
        Self {
            mode: MouseMode::Normal,
            selection: None,
            last_click: None,
            context_menu_visible: false,
            clickable_items: Vec::new(),
        }
    }
}

impl MousePlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "scarab-mouse",
                "0.1.0",
                "Comprehensive mouse support with context menus and smart selection",
                "Scarab Team",
            )
            .with_emoji("🖱️")
            .with_color("#00BFFF")
            .with_catchphrase("Point, click, interact"),
            state: Arc::new(Mutex::new(MouseState::default())),
        }
    }

    /// Get shared state reference for Bevy plugin
    pub fn state(&self) -> Arc<Mutex<MouseState>> {
        Arc::clone(&self.state)
    }

    /// Handle mouse mode escape sequences
    fn handle_mode_change(&self, data: &[u8]) -> bool {
        let mut state = self.state.lock();

        // Detect ANSI mouse mode sequences
        // CSI ? 1000 h - Enable X10 mouse reporting
        // CSI ? 1002 h - Enable button-event tracking
        // CSI ? 1003 h - Enable any-event tracking
        // CSI ? 1006 h - Enable SGR mouse mode
        // CSI ? 1000 l - Disable mouse reporting

        if data.len() >= 6 {
            if let Some(seq) = std::str::from_utf8(data).ok() {
                if seq.contains("\x1b[?1000h") || seq.contains("\x1b[?1002h") || seq.contains("\x1b[?1003h") {
                    state.mode = MouseMode::Application;
                    log::debug!("Mouse mode changed to Application");
                    return true;
                } else if seq.contains("\x1b[?1000l") {
                    state.mode = MouseMode::Normal;
                    log::debug!("Mouse mode changed to Normal");
                    return true;
                }
            }
        }

        false
    }
}

impl Default for MousePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for MousePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn get_commands(&self) -> Vec<ModalItem> {
        vec![
            ModalItem {
                id: "mouse.copy".to_string(),
                label: "Copy Selection".to_string(),
                description: Some("Copy selected text to clipboard".to_string()),
            },
            ModalItem {
                id: "mouse.paste".to_string(),
                label: "Paste".to_string(),
                description: Some("Paste from clipboard".to_string()),
            },
            ModalItem {
                id: "mouse.select_all".to_string(),
                label: "Select All".to_string(),
                description: Some("Select all text in terminal".to_string()),
            },
            ModalItem {
                id: "mouse.clear_selection".to_string(),
                label: "Clear Selection".to_string(),
                description: Some("Clear current text selection".to_string()),
            },
            ModalItem {
                id: "mouse.toggle_mode".to_string(),
                label: "Toggle Mouse Mode".to_string(),
                description: Some("Switch between Normal and Application mode".to_string()),
            },
        ]
    }

    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        log::info!("Mouse plugin loaded");
        ctx.notify_success("Mouse Support", "Mouse interaction enabled");
        Ok(())
    }

    async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
        // Scan for mouse mode change sequences
        if self.handle_mode_change(line.as_bytes()) {
            let state = self.state.lock();
            let mode_str = match state.mode {
                MouseMode::Normal => "Normal (Scarab)",
                MouseMode::Application => "Application (Program)",
            };

            ctx.notify_info("Mouse Mode", &format!("Switched to: {}", mode_str));
        }

        Ok(Action::Continue)
    }

    async fn on_input(&mut self, input: &[u8], _ctx: &PluginContext) -> Result<Action> {
        // Check for mode change sequences in input too
        self.handle_mode_change(input);
        Ok(Action::Continue)
    }

    async fn on_remote_command(&mut self, id: &str, ctx: &PluginContext) -> Result<()> {
        let mut state = self.state.lock();

        match id {
            "mouse.copy" => {
                if let Some(selection) = &state.selection {
                    // TODO: Integrate with clipboard plugin
                    log::info!("Copy selection: {:?}", selection);
                    ctx.notify_success("Copied", "Selection copied to clipboard");
                } else {
                    ctx.notify_warning("No Selection", "Nothing to copy");
                }
            }
            "mouse.paste" => {
                // TODO: Integrate with clipboard plugin
                log::info!("Paste requested");
                ctx.notify_info("Paste", "Pasting from clipboard");
            }
            "mouse.select_all" => {
                let (cols, rows) = ctx.get_size();
                state.selection = Some(selection::Selection {
                    start: Position { x: 0, y: 0 },
                    end: Position { x: cols - 1, y: rows - 1 },
                    kind: selection::SelectionKind::Block,
                });
                log::info!("Select all");
                ctx.notify_info("Selected", "All text selected");
            }
            "mouse.clear_selection" => {
                state.selection = None;
                log::info!("Clear selection");
                ctx.notify_info("Cleared", "Selection cleared");
            }
            "mouse.toggle_mode" => {
                state.mode = match state.mode {
                    MouseMode::Normal => MouseMode::Application,
                    MouseMode::Application => MouseMode::Normal,
                };

                let mode_str = match state.mode {
                    MouseMode::Normal => "Normal (Scarab handles mouse)",
                    MouseMode::Application => "Application (Program handles mouse)",
                };

                log::info!("Toggled mouse mode to: {:?}", state.mode);
                ctx.notify_info("Mouse Mode", mode_str);
            }
            _ => {}
        }

        Ok(())
    }

    async fn on_resize(&mut self, cols: u16, rows: u16, _ctx: &PluginContext) -> Result<()> {
        let mut state = self.state.lock();

        // Clear selection on resize as coordinates may be invalid
        if state.selection.is_some() {
            state.selection = None;
            log::debug!("Cleared selection due to terminal resize to {}x{}", cols, rows);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = MousePlugin::new();
        assert_eq!(plugin.metadata().name, "scarab-mouse");
        assert_eq!(plugin.metadata().emoji, Some("🖱️".to_string()));
    }

    #[test]
    fn test_mouse_mode_default() {
        let plugin = MousePlugin::new();
        let state = plugin.state.lock();
        assert_eq!(state.mode, MouseMode::Normal);
    }

    #[test]
    fn test_mouse_mode_detection() {
        let plugin = MousePlugin::new();

        // Enable mouse reporting
        assert!(plugin.handle_mode_change(b"\x1b[?1000h"));
        {
            let state = plugin.state.lock();
            assert_eq!(state.mode, MouseMode::Application);
        }

        // Disable mouse reporting
        assert!(plugin.handle_mode_change(b"\x1b[?1000l"));
        {
            let state = plugin.state.lock();
            assert_eq!(state.mode, MouseMode::Normal);
        }
    }

    #[test]
    fn test_commands() {
        let plugin = MousePlugin::new();
        let commands = plugin.get_commands();
        assert!(commands.len() >= 4);
        assert!(commands.iter().any(|c| c.id == "mouse.copy"));
        assert!(commands.iter().any(|c| c.id == "mouse.paste"));
    }
}
