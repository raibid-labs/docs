//! Tab Management Plugin for Scarab Terminal
//!
//! Provides tab creation, switching, reordering, and persistence.
//! Works in conjunction with scarab-panes for full workspace management.

use async_trait::async_trait;
use parking_lot::Mutex;
use scarab_plugin_api::{
    types::{ModalItem, RemoteCommand},
    Action, Plugin, PluginContext, PluginMetadata, Result,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tab metadata and state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: u64,
    pub title: String,
    pub session_id: Option<String>,
    pub working_dir: Option<String>,
    pub active_pane_id: Option<u64>,
    pub created_at: u64,
    pub last_active: u64,
}

impl Tab {
    fn new(id: u64, title: impl Into<String>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id,
            title: title.into(),
            session_id: None,
            working_dir: None,
            active_pane_id: None,
            created_at: now,
            last_active: now,
        }
    }
}

/// Internal plugin state
#[derive(Default)]
struct PluginState {
    tabs: Vec<Tab>,
    active_tab_index: usize,
    next_tab_id: u64,
}

impl PluginState {
    fn new() -> Self {
        let mut state = Self::default();
        // Create default tab
        state.tabs.push(Tab::new(0, "Terminal 1"));
        state.next_tab_id = 1;
        state
    }

    fn create_tab(&mut self, title: Option<String>) -> &Tab {
        let id = self.next_tab_id;
        self.next_tab_id += 1;

        let title = title.unwrap_or_else(|| format!("Terminal {}", self.tabs.len() + 1));
        self.tabs.push(Tab::new(id, title));

        // Switch to newly created tab
        self.active_tab_index = self.tabs.len() - 1;

        &self.tabs[self.active_tab_index]
    }

    fn close_tab(&mut self, index: usize) -> Option<Tab> {
        if self.tabs.len() <= 1 {
            // Don't close the last tab
            return None;
        }

        if index >= self.tabs.len() {
            return None;
        }

        let tab = self.tabs.remove(index);

        // Adjust active index if needed
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        } else if index < self.active_tab_index {
            self.active_tab_index -= 1;
        }

        Some(tab)
    }

    fn switch_to_tab(&mut self, index: usize) -> bool {
        if index < self.tabs.len() {
            self.active_tab_index = index;
            true
        } else {
            false
        }
    }

    fn next_tab(&mut self) {
        self.active_tab_index = (self.active_tab_index + 1) % self.tabs.len();
    }

    fn prev_tab(&mut self) {
        if self.active_tab_index == 0 {
            self.active_tab_index = self.tabs.len() - 1;
        } else {
            self.active_tab_index -= 1;
        }
    }

    fn move_tab(&mut self, from: usize, to: usize) -> bool {
        if from >= self.tabs.len() || to >= self.tabs.len() {
            return false;
        }

        let tab = self.tabs.remove(from);
        self.tabs.insert(to, tab);

        // Update active index
        if self.active_tab_index == from {
            self.active_tab_index = to;
        } else if from < self.active_tab_index && to >= self.active_tab_index {
            self.active_tab_index -= 1;
        } else if from > self.active_tab_index && to <= self.active_tab_index {
            self.active_tab_index += 1;
        }

        true
    }

    fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab_index]
    }

    fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab_index]
    }
}

pub struct TabsPlugin {
    metadata: PluginMetadata,
    state: Mutex<PluginState>,
}

impl TabsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "scarab-tabs",
                "0.1.0",
                "Tab management and workspace organization",
                "Scarab Team",
            )
            .with_emoji("📑")
            .with_color("#4A90E2")
            .with_catchphrase("Organize your terminal like a pro"),
            state: Mutex::new(PluginState::new()),
        }
    }

    fn handle_keybinding(&self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        let mut state = self.state.lock();

        // Ctrl+Shift+T (new tab) - ASCII 0x14
        if input == [0x14] {
            let tab = state.create_tab(None);
            log::info!("Created new tab: {} (ID: {})", tab.title, tab.id);

            ctx.notify_success(
                "New Tab",
                &format!("Created tab: {}", tab.title)
            );

            // Queue command to create session in daemon
            ctx.queue_command(RemoteCommand::PluginNotify {
                title: "Tab Created".to_string(),
                body: format!("Tab #{}: {}", tab.id, tab.title),
                level: scarab_plugin_api::context::NotifyLevel::Success,
            });

            return Ok(Action::Modify(Vec::new()));
        }

        // Ctrl+Tab (next tab) - ASCII 0x09 with special handling
        if input == [0x09] && state.tabs.len() > 1 {
            state.next_tab();
            let tab = state.active_tab();
            log::info!("Switched to next tab: {}", tab.title);

            ctx.notify_info("Tab Switch", &format!("Active: {}", tab.title));
            return Ok(Action::Modify(Vec::new()));
        }

        // Ctrl+Shift+W (close tab) - ASCII 0x17
        if input == [0x17] {
            if state.tabs.len() > 1 {
                let index = state.active_tab_index;
                if let Some(closed_tab) = state.close_tab(index) {
                    log::info!("Closed tab: {} (ID: {})", closed_tab.title, closed_tab.id);

                    ctx.notify_info(
                        "Tab Closed",
                        &format!("Closed: {}", closed_tab.title)
                    );

                    return Ok(Action::Modify(Vec::new()));
                }
            } else {
                ctx.notify_warning("Cannot Close", "Cannot close the last tab");
            }
            return Ok(Action::Modify(Vec::new()));
        }

        // Ctrl+1-9 (switch to tab by number) - ASCII 0x31-0x39 with Ctrl modifier
        if input.len() == 1 {
            let byte = input[0];
            // Check for Ctrl+1 through Ctrl+9
            if (1..=9).contains(&byte) {
                let tab_index = (byte - 1) as usize;
                if state.switch_to_tab(tab_index) {
                    let tab = state.active_tab();
                    log::info!("Switched to tab {}: {}", tab_index + 1, tab.title);

                    ctx.notify_info(
                        "Tab Switch",
                        &format!("Tab {}: {}", tab_index + 1, tab.title)
                    );

                    return Ok(Action::Modify(Vec::new()));
                }
            }
        }

        Ok(Action::Continue)
    }
}

impl Default for TabsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for TabsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn get_commands(&self) -> Vec<ModalItem> {
        vec![
            ModalItem {
                id: "tabs.new".to_string(),
                label: "New Tab".to_string(),
                description: Some("Create a new tab (Ctrl+Shift+T)".to_string()),
            },
            ModalItem {
                id: "tabs.close".to_string(),
                label: "Close Tab".to_string(),
                description: Some("Close current tab (Ctrl+Shift+W)".to_string()),
            },
            ModalItem {
                id: "tabs.next".to_string(),
                label: "Next Tab".to_string(),
                description: Some("Switch to next tab (Ctrl+Tab)".to_string()),
            },
            ModalItem {
                id: "tabs.prev".to_string(),
                label: "Previous Tab".to_string(),
                description: Some("Switch to previous tab (Ctrl+Shift+Tab)".to_string()),
            },
            ModalItem {
                id: "tabs.list".to_string(),
                label: "List Tabs".to_string(),
                description: Some("Show all open tabs".to_string()),
            },
            ModalItem {
                id: "tabs.rename".to_string(),
                label: "Rename Tab".to_string(),
                description: Some("Rename current tab".to_string()),
            },
        ]
    }

    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        self.handle_keybinding(input, ctx)
    }

    async fn on_remote_command(&mut self, id: &str, ctx: &PluginContext) -> Result<()> {
        let mut state = self.state.lock();

        match id {
            "tabs.new" => {
                let tab = state.create_tab(None);
                log::info!("Command: Created new tab: {}", tab.title);
                ctx.notify_success("New Tab", &format!("Created: {}", tab.title));
            }
            "tabs.close" => {
                if state.tabs.len() > 1 {
                    let index = state.active_tab_index;
                    if let Some(closed_tab) = state.close_tab(index) {
                        log::info!("Command: Closed tab: {}", closed_tab.title);
                        ctx.notify_info("Tab Closed", &format!("Closed: {}", closed_tab.title));
                    }
                } else {
                    ctx.notify_warning("Cannot Close", "Cannot close the last tab");
                }
            }
            "tabs.next" => {
                state.next_tab();
                let tab = state.active_tab();
                log::info!("Command: Switched to next tab: {}", tab.title);
                ctx.notify_info("Tab Switch", &format!("Active: {}", tab.title));
            }
            "tabs.prev" => {
                state.prev_tab();
                let tab = state.active_tab();
                log::info!("Command: Switched to previous tab: {}", tab.title);
                ctx.notify_info("Tab Switch", &format!("Active: {}", tab.title));
            }
            "tabs.list" => {
                let tabs_info: Vec<String> = state
                    .tabs
                    .iter()
                    .enumerate()
                    .map(|(i, tab)| {
                        let marker = if i == state.active_tab_index { "● " } else { "○ " };
                        format!("{}{}: {}", marker, i + 1, tab.title)
                    })
                    .collect();

                log::info!("Command: Listing tabs");
                ctx.notify_info(
                    "Open Tabs",
                    &tabs_info.join("\n")
                );
            }
            "tabs.rename" => {
                log::info!("Command: Rename tab (not yet implemented)");
                ctx.notify_info("Rename Tab", "Feature coming soon");
            }
            _ => {}
        }

        Ok(())
    }

    async fn on_resize(&mut self, cols: u16, rows: u16, ctx: &PluginContext) -> Result<()> {
        log::debug!("Tabs plugin: Terminal resized to {}x{}", cols, rows);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tab() {
        let mut state = PluginState::new();
        assert_eq!(state.tabs.len(), 1);

        state.create_tab(Some("Test Tab".to_string()));
        assert_eq!(state.tabs.len(), 2);
        assert_eq!(state.tabs[1].title, "Test Tab");
        assert_eq!(state.active_tab_index, 1);
    }

    #[test]
    fn test_close_tab() {
        let mut state = PluginState::new();
        state.create_tab(Some("Tab 2".to_string()));
        state.create_tab(Some("Tab 3".to_string()));

        assert_eq!(state.tabs.len(), 3);

        let closed = state.close_tab(1);
        assert!(closed.is_some());
        assert_eq!(state.tabs.len(), 2);
    }

    #[test]
    fn test_cannot_close_last_tab() {
        let mut state = PluginState::new();
        assert_eq!(state.tabs.len(), 1);

        let closed = state.close_tab(0);
        assert!(closed.is_none());
        assert_eq!(state.tabs.len(), 1);
    }

    #[test]
    fn test_switch_tab() {
        let mut state = PluginState::new();
        state.create_tab(Some("Tab 2".to_string()));
        state.create_tab(Some("Tab 3".to_string()));

        assert_eq!(state.active_tab_index, 2);

        state.switch_to_tab(0);
        assert_eq!(state.active_tab_index, 0);

        state.switch_to_tab(1);
        assert_eq!(state.active_tab_index, 1);
    }

    #[test]
    fn test_next_prev_tab() {
        let mut state = PluginState::new();
        state.create_tab(Some("Tab 2".to_string()));
        state.create_tab(Some("Tab 3".to_string()));

        state.switch_to_tab(0);
        assert_eq!(state.active_tab_index, 0);

        state.next_tab();
        assert_eq!(state.active_tab_index, 1);

        state.next_tab();
        assert_eq!(state.active_tab_index, 2);

        state.next_tab(); // Wraps around
        assert_eq!(state.active_tab_index, 0);

        state.prev_tab(); // Wraps around
        assert_eq!(state.active_tab_index, 2);

        state.prev_tab();
        assert_eq!(state.active_tab_index, 1);
    }

    #[test]
    fn test_move_tab() {
        let mut state = PluginState::new();
        state.create_tab(Some("Tab 2".to_string()));
        state.create_tab(Some("Tab 3".to_string()));

        state.switch_to_tab(0);

        // Move tab 0 to position 2
        state.move_tab(0, 2);
        assert_eq!(state.tabs[2].title, "Terminal 1");
        assert_eq!(state.active_tab_index, 2);
    }
}
