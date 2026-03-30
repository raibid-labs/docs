//! Pane Management Plugin for Scarab Terminal
//!
//! Provides split pane management with separate PTY sessions per pane.
//! Works in conjunction with scarab-tabs for full workspace management.

use async_trait::async_trait;
use parking_lot::Mutex;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use scarab_plugin_api::{
    types::{ModalItem, RemoteCommand},
    Action, Plugin, PluginContext, PluginMetadata, Result,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Pane split direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

/// Pane layout information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneLayout {
    pub id: u64,
    pub parent_id: Option<u64>,
    pub split_direction: Option<SplitDirection>,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub is_focused: bool,
}

impl PaneLayout {
    fn new(id: u64, x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            id,
            parent_id: None,
            split_direction: None,
            x,
            y,
            width,
            height,
            is_focused: true,
        }
    }
}

/// Pane metadata and PTY handle
#[derive(Debug)]
pub struct Pane {
    pub layout: PaneLayout,
    pub session_id: Option<String>,
    pub working_dir: Option<String>,
    pub created_at: u64,
    // Note: In production, this would hold PTY master/slave handles
    // For now, we just track metadata
}

impl Pane {
    fn new(layout: PaneLayout) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            layout,
            session_id: None,
            working_dir: None,
            created_at: now,
        }
    }
}

/// Internal plugin state
struct PluginState {
    panes: HashMap<u64, Pane>,
    active_pane_id: u64,
    next_pane_id: u64,
    terminal_size: (u16, u16), // cols, rows
}

impl PluginState {
    fn new(cols: u16, rows: u16) -> Self {
        let mut state = Self {
            panes: HashMap::new(),
            active_pane_id: 0,
            next_pane_id: 1,
            terminal_size: (cols, rows),
        };

        // Create initial pane
        let layout = PaneLayout::new(0, 0, 0, cols, rows);
        state.panes.insert(0, Pane::new(layout));
        state
    }

    fn split_pane(&mut self, pane_id: u64, direction: SplitDirection) -> Option<u64> {
        let pane = self.panes.get(&pane_id)?;
        let layout = pane.layout.clone();

        let new_id = self.next_pane_id;
        self.next_pane_id += 1;

        let (layout1, layout2) = match direction {
            SplitDirection::Horizontal => {
                // Split horizontally (top/bottom)
                let height1 = layout.height / 2;
                let height2 = layout.height - height1;

                let mut l1 = layout.clone();
                l1.height = height1;

                let mut l2 = PaneLayout::new(
                    new_id,
                    layout.x,
                    layout.y + height1,
                    layout.width,
                    height2,
                );
                l2.parent_id = Some(pane_id);
                l2.split_direction = Some(direction);
                l2.is_focused = true;

                (l1, l2)
            }
            SplitDirection::Vertical => {
                // Split vertically (left/right)
                let width1 = layout.width / 2;
                let width2 = layout.width - width1;

                let mut l1 = layout.clone();
                l1.width = width1;

                let mut l2 = PaneLayout::new(
                    new_id,
                    layout.x + width1,
                    layout.y,
                    width2,
                    layout.height,
                );
                l2.parent_id = Some(pane_id);
                l2.split_direction = Some(direction);
                l2.is_focused = true;

                (l1, l2)
            }
        };

        // Update existing pane
        if let Some(pane) = self.panes.get_mut(&pane_id) {
            pane.layout = layout1;
            pane.layout.is_focused = false;
        }

        // Create new pane
        self.panes.insert(new_id, Pane::new(layout2));
        self.active_pane_id = new_id;

        log::info!(
            "Split pane {} {:?}, created pane {}",
            pane_id,
            direction,
            new_id
        );

        Some(new_id)
    }

    fn close_pane(&mut self, pane_id: u64) -> Option<Pane> {
        // Don't close the last pane
        if self.panes.len() <= 1 {
            return None;
        }

        let pane = self.panes.remove(&pane_id)?;

        // If closing the active pane, switch to another
        if self.active_pane_id == pane_id {
            self.active_pane_id = *self.panes.keys().next().unwrap();
            if let Some(active) = self.panes.get_mut(&self.active_pane_id) {
                active.layout.is_focused = true;
            }
        }

        // TODO: Resize adjacent panes to fill the space
        self.recalculate_layout();

        Some(pane)
    }

    fn focus_pane(&mut self, pane_id: u64) -> bool {
        if !self.panes.contains_key(&pane_id) {
            return false;
        }

        // Unfocus all panes
        for pane in self.panes.values_mut() {
            pane.layout.is_focused = false;
        }

        // Focus the target pane
        if let Some(pane) = self.panes.get_mut(&pane_id) {
            pane.layout.is_focused = true;
            self.active_pane_id = pane_id;
            true
        } else {
            false
        }
    }

    fn navigate(&mut self, direction: Direction) -> bool {
        let current_layout = &self.panes.get(&self.active_pane_id).unwrap().layout;
        let (cx, cy) = (current_layout.x, current_layout.y);

        let mut candidates: Vec<(u64, u32)> = Vec::new();

        for (id, pane) in &self.panes {
            if *id == self.active_pane_id {
                continue;
            }

            let (px, py) = (pane.layout.x, pane.layout.y);

            let is_valid = match direction {
                Direction::Up => py < cy,
                Direction::Down => py > cy,
                Direction::Left => px < cx,
                Direction::Right => px > cx,
            };

            if is_valid {
                let distance = ((px as i32 - cx as i32).abs() + (py as i32 - cy as i32).abs()) as u32;
                candidates.push((*id, distance));
            }
        }

        if let Some((nearest_id, _)) = candidates.iter().min_by_key(|(_, dist)| dist) {
            self.focus_pane(*nearest_id)
        } else {
            false
        }
    }

    fn resize_pane(&mut self, pane_id: u64, direction: Direction, amount: i16) -> bool {
        // TODO: Implement pane resizing logic
        // This requires updating the layout of the pane and its neighbors
        log::info!("Resize pane {} {:?} by {}", pane_id, direction, amount);
        true
    }

    fn recalculate_layout(&mut self) {
        // TODO: Implement smart layout recalculation
        // For now, we just distribute space equally
        let count = self.panes.len();
        if count == 0 {
            return;
        }

        let (cols, rows) = self.terminal_size;

        // Simple equal distribution (grid layout)
        let cols_per_pane = cols / count as u16;
        let rows_per_pane = rows;

        for (i, (_, pane)) in self.panes.iter_mut().enumerate() {
            pane.layout.x = i as u16 * cols_per_pane;
            pane.layout.y = 0;
            pane.layout.width = cols_per_pane;
            pane.layout.height = rows_per_pane;
        }
    }

    fn update_terminal_size(&mut self, cols: u16, rows: u16) {
        self.terminal_size = (cols, rows);
        self.recalculate_layout();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct PanesPlugin {
    metadata: PluginMetadata,
    state: Arc<Mutex<PluginState>>,
}

impl PanesPlugin {
    pub fn new() -> Self {
        Self::with_size(80, 24)
    }

    pub fn with_size(cols: u16, rows: u16) -> Self {
        Self {
            metadata: PluginMetadata::new(
                "scarab-panes",
                "0.1.0",
                "Split pane management with PTY sessions",
                "Scarab Team",
            )
            .with_emoji("🪟")
            .with_color("#E94B3C")
            .with_catchphrase("Split, resize, conquer"),
            state: Arc::new(Mutex::new(PluginState::new(cols, rows))),
        }
    }

    fn handle_keybinding(&self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        let mut state = self.state.lock();

        // Ctrl+Shift+| (vertical split) - would be custom sequence
        // For now, we'll use simpler bindings for testing

        // Ctrl+Shift+- (horizontal split) - ASCII 0x1F
        if input == [0x1f] {
            let active_id = state.active_pane_id;
            if let Some(new_id) = state.split_pane(active_id, SplitDirection::Horizontal) {
                log::info!("Split pane horizontally, created pane {}", new_id);
                ctx.notify_success(
                    "Split Pane",
                    &format!("Created pane {} (horizontal)", new_id)
                );
                return Ok(Action::Modify(Vec::new()));
            }
        }

        // Ctrl+Shift+\ (vertical split) - ASCII 0x1C
        if input == [0x1c] {
            let active_id = state.active_pane_id;
            if let Some(new_id) = state.split_pane(active_id, SplitDirection::Vertical) {
                log::info!("Split pane vertically, created pane {}", new_id);
                ctx.notify_success(
                    "Split Pane",
                    &format!("Created pane {} (vertical)", new_id)
                );
                return Ok(Action::Modify(Vec::new()));
            }
        }

        Ok(Action::Continue)
    }
}

impl Default for PanesPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for PanesPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn get_commands(&self) -> Vec<ModalItem> {
        vec![
            ModalItem {
                id: "panes.split_horizontal".to_string(),
                label: "Split Pane Horizontally".to_string(),
                description: Some("Split current pane horizontally (Ctrl+Shift+-)".to_string()),
            },
            ModalItem {
                id: "panes.split_vertical".to_string(),
                label: "Split Pane Vertically".to_string(),
                description: Some("Split current pane vertically (Ctrl+Shift+|)".to_string()),
            },
            ModalItem {
                id: "panes.close".to_string(),
                label: "Close Pane".to_string(),
                description: Some("Close current pane (Ctrl+Shift+W)".to_string()),
            },
            ModalItem {
                id: "panes.navigate_up".to_string(),
                label: "Navigate Up".to_string(),
                description: Some("Focus pane above (Ctrl+Shift+Up)".to_string()),
            },
            ModalItem {
                id: "panes.navigate_down".to_string(),
                label: "Navigate Down".to_string(),
                description: Some("Focus pane below (Ctrl+Shift+Down)".to_string()),
            },
            ModalItem {
                id: "panes.navigate_left".to_string(),
                label: "Navigate Left".to_string(),
                description: Some("Focus pane to the left (Ctrl+Shift+Left)".to_string()),
            },
            ModalItem {
                id: "panes.navigate_right".to_string(),
                label: "Navigate Right".to_string(),
                description: Some("Focus pane to the right (Ctrl+Shift+Right)".to_string()),
            },
            ModalItem {
                id: "panes.zoom".to_string(),
                label: "Zoom Pane".to_string(),
                description: Some("Toggle pane zoom (fullscreen)".to_string()),
            },
        ]
    }

    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        self.handle_keybinding(input, ctx)
    }

    async fn on_remote_command(&mut self, id: &str, ctx: &PluginContext) -> Result<()> {
        let mut state = self.state.lock();

        match id {
            "panes.split_horizontal" => {
                let active_id = state.active_pane_id;
                if let Some(new_id) = state.split_pane(active_id, SplitDirection::Horizontal) {
                    log::info!("Command: Split pane horizontally, created {}", new_id);
                    ctx.notify_success("Split Pane", &format!("Created pane {}", new_id));
                }
            }
            "panes.split_vertical" => {
                let active_id = state.active_pane_id;
                if let Some(new_id) = state.split_pane(active_id, SplitDirection::Vertical) {
                    log::info!("Command: Split pane vertically, created {}", new_id);
                    ctx.notify_success("Split Pane", &format!("Created pane {}", new_id));
                }
            }
            "panes.close" => {
                let active_id = state.active_pane_id;
                if state.panes.len() > 1 {
                    if let Some(closed) = state.close_pane(active_id) {
                        log::info!("Command: Closed pane {}", active_id);
                        ctx.notify_info("Pane Closed", &format!("Closed pane {}", active_id));
                    }
                } else {
                    ctx.notify_warning("Cannot Close", "Cannot close the last pane");
                }
            }
            "panes.navigate_up" => {
                if state.navigate(Direction::Up) {
                    ctx.notify_info("Navigate", "Focused pane above");
                }
            }
            "panes.navigate_down" => {
                if state.navigate(Direction::Down) {
                    ctx.notify_info("Navigate", "Focused pane below");
                }
            }
            "panes.navigate_left" => {
                if state.navigate(Direction::Left) {
                    ctx.notify_info("Navigate", "Focused pane to the left");
                }
            }
            "panes.navigate_right" => {
                if state.navigate(Direction::Right) {
                    ctx.notify_info("Navigate", "Focused pane to the right");
                }
            }
            "panes.zoom" => {
                log::info!("Command: Toggle pane zoom (not yet implemented)");
                ctx.notify_info("Zoom", "Feature coming soon");
            }
            _ => {}
        }

        Ok(())
    }

    async fn on_resize(&mut self, cols: u16, rows: u16, ctx: &PluginContext) -> Result<()> {
        let mut state = self.state.lock();
        state.update_terminal_size(cols, rows);
        log::info!("Panes plugin: Terminal resized to {}x{}, recalculated layout", cols, rows);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_horizontal() {
        let mut state = PluginState::new(80, 24);
        assert_eq!(state.panes.len(), 1);

        let new_id = state.split_pane(0, SplitDirection::Horizontal);
        assert!(new_id.is_some());
        assert_eq!(state.panes.len(), 2);

        let pane0 = &state.panes[&0];
        let pane1 = &state.panes[&new_id.unwrap()];

        assert_eq!(pane0.layout.height, 12);
        assert_eq!(pane1.layout.height, 12);
        assert_eq!(pane1.layout.y, 12);
    }

    #[test]
    fn test_split_vertical() {
        let mut state = PluginState::new(80, 24);
        let new_id = state.split_pane(0, SplitDirection::Vertical);

        assert!(new_id.is_some());
        assert_eq!(state.panes.len(), 2);

        let pane0 = &state.panes[&0];
        let pane1 = &state.panes[&new_id.unwrap()];

        assert_eq!(pane0.layout.width, 40);
        assert_eq!(pane1.layout.width, 40);
        assert_eq!(pane1.layout.x, 40);
    }

    #[test]
    fn test_close_pane() {
        let mut state = PluginState::new(80, 24);
        let new_id = state.split_pane(0, SplitDirection::Horizontal).unwrap();

        assert_eq!(state.panes.len(), 2);

        let closed = state.close_pane(new_id);
        assert!(closed.is_some());
        assert_eq!(state.panes.len(), 1);
    }

    #[test]
    fn test_cannot_close_last_pane() {
        let mut state = PluginState::new(80, 24);
        let closed = state.close_pane(0);
        assert!(closed.is_none());
        assert_eq!(state.panes.len(), 1);
    }

    #[test]
    fn test_focus_pane() {
        let mut state = PluginState::new(80, 24);
        let new_id = state.split_pane(0, SplitDirection::Horizontal).unwrap();

        assert_eq!(state.active_pane_id, new_id);
        assert!(state.panes[&new_id].layout.is_focused);
        assert!(!state.panes[&0].layout.is_focused);

        state.focus_pane(0);
        assert_eq!(state.active_pane_id, 0);
        assert!(state.panes[&0].layout.is_focused);
        assert!(!state.panes[&new_id].layout.is_focused);
    }

    #[test]
    fn test_resize_updates_layout() {
        let mut state = PluginState::new(80, 24);
        state.split_pane(0, SplitDirection::Horizontal);

        state.update_terminal_size(160, 48);
        assert_eq!(state.terminal_size, (160, 48));
    }
}
