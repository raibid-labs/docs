// Extended selection system for scrollback buffer
// Allows text selection in both live view and scrollback history

use bevy::prelude::*;
use crate::terminal::scrollback::{ScrollbackBuffer, ScrollbackState};
use crate::ui::visual_selection::{SelectionMode, SelectionRegion};
use arboard::Clipboard;

/// Extended selection state that works with scrollback
#[derive(Resource)]
pub struct ScrollbackSelectionState {
    pub active: bool,
    pub mode: SelectionMode,
    pub region: SelectionRegion,
    /// Whether selection is in scrollback (vs live view)
    pub in_scrollback: bool,
    /// Start line in scrollback buffer (if in_scrollback is true)
    pub scrollback_start_line: usize,
    /// End line in scrollback buffer (if in_scrollback is true)
    pub scrollback_end_line: usize,
}

impl Default for ScrollbackSelectionState {
    fn default() -> Self {
        Self {
            active: false,
            mode: SelectionMode::Character,
            region: SelectionRegion::default(),
            in_scrollback: false,
            scrollback_start_line: 0,
            scrollback_end_line: 0,
        }
    }
}

impl ScrollbackSelectionState {
    /// Start selection in scrollback
    pub fn start_scrollback_selection(&mut self, line: usize, col: u16, mode: SelectionMode) {
        self.active = true;
        self.in_scrollback = true;
        self.mode = mode;
        self.scrollback_start_line = line;
        self.scrollback_end_line = line;
        self.region = SelectionRegion::new(col, 0, col, 0);
    }

    /// Update selection endpoint
    pub fn update_scrollback_selection(&mut self, line: usize, col: u16) {
        if !self.active || !self.in_scrollback {
            return;
        }

        self.scrollback_end_line = line;
        self.region.end_x = col;
    }

    /// Clear selection
    pub fn clear(&mut self) {
        self.active = false;
        self.in_scrollback = false;
        self.region = SelectionRegion::default();
    }
}

/// Event for mouse-based selection in scrollback
#[derive(Event)]
pub struct ScrollbackMouseSelection {
    pub start_line: usize,
    pub start_col: u16,
    pub end_line: usize,
    pub end_col: u16,
}

/// System to handle mouse selection in scrollback
fn handle_mouse_selection(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut selection: ResMut<ScrollbackSelectionState>,
    scrollback: Res<ScrollbackBuffer>,
    scrollback_state: Res<ScrollbackState>,
) {
    if !scrollback_state.is_scrolled {
        return; // Only handle mouse selection when in scrollback view
    }

    let window = windows.single();

    // Start selection on mouse press
    if mouse_buttons.just_pressed(MouseButton::Left) {
        if let Some(cursor_pos) = window.cursor_position() {
            // Convert cursor position to scrollback line/column
            // This would need proper coordinate conversion based on renderer
            // For now, this is a placeholder
            let line = 0; // TODO: Convert cursor_pos to scrollback line
            let col = 0; // TODO: Convert cursor_pos to column

            selection.start_scrollback_selection(line, col, SelectionMode::Character);
            debug!("Started scrollback selection at line {}, col {}", line, col);
        }
    }

    // Update selection while dragging
    if mouse_buttons.pressed(MouseButton::Left) && selection.active {
        if let Some(cursor_pos) = window.cursor_position() {
            let line = 0; // TODO: Convert cursor_pos to scrollback line
            let col = 0; // TODO: Convert cursor_pos to column

            selection.update_scrollback_selection(line, col);
        }
    }

    // End selection on release
    if mouse_buttons.just_released(MouseButton::Left) && selection.active {
        debug!("Ended scrollback selection");
    }
}

/// System to copy scrollback selection to clipboard
fn copy_scrollback_selection(
    keys: Res<ButtonInput<KeyCode>>,
    selection: Res<ScrollbackSelectionState>,
    scrollback: Res<ScrollbackBuffer>,
) {
    if !selection.active || !selection.in_scrollback {
        return;
    }

    // Ctrl+C or 'y' to copy
    let should_copy = (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight))
        && keys.just_pressed(KeyCode::KeyC)
        || keys.just_pressed(KeyCode::KeyY);

    if !should_copy {
        return;
    }

    // Extract text from scrollback buffer
    let mut text = String::new();

    let start_line = selection.scrollback_start_line.min(selection.scrollback_end_line);
    let end_line = selection.scrollback_start_line.max(selection.scrollback_end_line);

    for line_idx in start_line..=end_line {
        if let Some(line) = scrollback.get_line(line_idx) {
            let line_text = line.to_string();

            // Apply column selection for first and last lines
            if line_idx == start_line && line_idx == end_line {
                // Single line selection
                let start_col = selection.region.start_x.min(selection.region.end_x) as usize;
                let end_col = (selection.region.start_x.max(selection.region.end_x) as usize + 1)
                    .min(line_text.len());

                if start_col < line_text.len() {
                    text.push_str(&line_text[start_col..end_col]);
                }
            } else if line_idx == start_line {
                // First line of multi-line selection
                let start_col = selection.region.start_x as usize;
                if start_col < line_text.len() {
                    text.push_str(&line_text[start_col..]);
                }
                text.push('\n');
            } else if line_idx == end_line {
                // Last line of multi-line selection
                let end_col = (selection.region.end_x as usize + 1).min(line_text.len());
                text.push_str(&line_text[..end_col]);
            } else {
                // Middle lines - full line
                text.push_str(&line_text);
                text.push('\n');
            }
        }
    }

    // Copy to clipboard
    match Clipboard::new() {
        Ok(mut clipboard) => {
            if let Err(e) = clipboard.set_text(&text) {
                error!("Failed to copy scrollback selection to clipboard: {}", e);
            } else {
                info!(
                    "Copied {} characters from scrollback to clipboard",
                    text.len()
                );
            }
        }
        Err(e) => {
            error!("Failed to initialize clipboard: {}", e);
        }
    }
}

/// Plugin for scrollback selection
pub struct ScrollbackSelectionPlugin;

impl Plugin for ScrollbackSelectionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScrollbackSelectionState::default())
            .add_event::<ScrollbackMouseSelection>()
            .add_systems(
                Update,
                (handle_mouse_selection, copy_scrollback_selection).chain(),
            );

        info!("Scrollback selection plugin initialized");
    }
}
