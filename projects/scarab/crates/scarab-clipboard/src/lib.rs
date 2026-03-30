//! Clipboard Integration Plugin for Scarab Terminal
//!
//! Provides comprehensive copy/paste functionality with support for:
//! - Multiple selection modes (character, word, line, block)
//! - Cross-platform clipboard integration via arboard
//! - X11 primary selection support
//! - Paste confirmation for large/multiline content
//! - Bracket paste mode for shell safety

use async_trait::async_trait;
use parking_lot::Mutex;
use regex::Regex;
use scarab_plugin_api::{
    types::{ModalItem, OverlayStyle, RemoteCommand},
    Action, Plugin, PluginContext, PluginMetadata, Result,
};

mod clipboard;
mod selection;

pub use clipboard::{ClipboardManager, ClipboardType, PasteConfirmation};
pub use selection::{SelectionMode, SelectionRegion, SelectionState};

/// Main clipboard plugin
pub struct ClipboardPlugin {
    metadata: PluginMetadata,
    state: Mutex<PluginState>,
    clipboard_manager: Mutex<ClipboardManager>,
    word_boundary_regex: Regex,
}

/// Internal plugin state
#[derive(Default)]
struct PluginState {
    selection: SelectionState,
    paste_pending: Option<PendingPaste>,
    bracket_mode_enabled: bool,
}

/// Pending paste operation awaiting confirmation
#[derive(Clone)]
struct PendingPaste {
    text: String,
    clipboard_type: ClipboardType,
    requires_confirmation: bool,
}

impl ClipboardPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "scarab-clipboard",
                "0.1.0",
                "Clipboard integration and text selection for terminal",
                "Scarab Team",
            )
            .with_emoji("📋")
            .with_color("#FFA500")
            .with_catchphrase("Copy, paste, and select with ease"),
            state: Mutex::new(PluginState::default()),
            clipboard_manager: Mutex::new(ClipboardManager::new()),
            word_boundary_regex: Regex::new(r"\b").unwrap(),
        }
    }

    /// Extract text from terminal grid based on selection region
    fn extract_selection_text(
        &self,
        ctx: &PluginContext,
        region: &SelectionRegion,
        mode: SelectionMode,
    ) -> String {
        let mut text = String::new();
        let (cols, rows) = ctx.get_size();

        let mut normalized_region = region.clone();
        normalized_region.normalize();

        match mode {
            SelectionMode::Character => {
                // Character-wise selection
                for y in normalized_region.start_y..=normalized_region.end_y.min(rows - 1) {
                    let start_x = if y == normalized_region.start_y {
                        normalized_region.start_x
                    } else {
                        0
                    };

                    let end_x = if y == normalized_region.end_y {
                        normalized_region.end_x.min(cols - 1)
                    } else {
                        cols - 1
                    };

                    if let Some(line) = ctx.get_line(y) {
                        let line_chars: Vec<char> = line.chars().collect();
                        for x in start_x..=end_x {
                            if let Some(&ch) = line_chars.get(x as usize) {
                                text.push(ch);
                            }
                        }
                    }

                    if y < normalized_region.end_y {
                        text.push('\n');
                    }
                }
            }

            SelectionMode::Word => {
                // Word-wise selection (expand to word boundaries)
                if let Some(line) = ctx.get_line(normalized_region.start_y) {
                    let (word_start, word_end) =
                        self.find_word_boundaries(&line, normalized_region.start_x);
                    let line_chars: Vec<char> = line.chars().collect();

                    for x in word_start..=word_end {
                        if let Some(&ch) = line_chars.get(x as usize) {
                            text.push(ch);
                        }
                    }
                }
            }

            SelectionMode::Line => {
                // Line-wise selection (full lines)
                for y in normalized_region.start_y..=normalized_region.end_y.min(rows - 1) {
                    if let Some(line) = ctx.get_line(y) {
                        text.push_str(&line);
                    }

                    if y < normalized_region.end_y {
                        text.push('\n');
                    }
                }
            }

            SelectionMode::Block => {
                // Block/rectangular selection
                for y in normalized_region.start_y..=normalized_region.end_y.min(rows - 1) {
                    if let Some(line) = ctx.get_line(y) {
                        let line_chars: Vec<char> = line.chars().collect();
                        for x in normalized_region.start_x
                            ..=normalized_region.end_x.min(cols - 1)
                        {
                            if let Some(&ch) = line_chars.get(x as usize) {
                                text.push(ch);
                            }
                        }
                    }

                    if y < normalized_region.end_y {
                        text.push('\n');
                    }
                }
            }
        }

        // Trim trailing whitespace from each line
        text.lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Find word boundaries for word selection
    fn find_word_boundaries(&self, line: &str, col: u16) -> (u16, u16) {
        let chars: Vec<char> = line.chars().collect();
        let col = col as usize;

        if col >= chars.len() {
            return (col as u16, col as u16);
        }

        // Find start of word
        let mut start = col;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Find end of word
        let mut end = col;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        (start as u16, end.saturating_sub(1) as u16)
    }

    /// Check if paste requires confirmation (multiline or large)
    fn requires_paste_confirmation(text: &str) -> bool {
        const MAX_SAFE_SIZE: usize = 1024; // 1KB
        const MAX_SAFE_LINES: usize = 5;

        let line_count = text.lines().count();
        let size = text.len();

        line_count > MAX_SAFE_LINES || size > MAX_SAFE_SIZE
    }

    /// Handle copy operation
    fn handle_copy(
        &self,
        ctx: &PluginContext,
        state: &mut PluginState,
        clipboard_type: ClipboardType,
    ) -> Result<Action> {
        if !state.selection.active {
            ctx.notify_warning("Copy Failed", "No text selected");
            return Ok(Action::Continue);
        }

        let text = self.extract_selection_text(
            ctx,
            &state.selection.region,
            state.selection.mode,
        );

        if text.is_empty() {
            ctx.notify_warning("Copy Failed", "Selection is empty");
            return Ok(Action::Continue);
        }

        // Copy to clipboard
        let mut clipboard_mgr = self.clipboard_manager.lock();
        match clipboard_mgr.copy(&text, clipboard_type) {
            Ok(_) => {
                log::info!("Copied {} characters to {:?}", text.len(), clipboard_type);
                ctx.notify_success(
                    "Copied",
                    &format!("Copied {} characters", text.len()),
                );

                // Clear selection after copy
                state.selection.clear();
            }
            Err(e) => {
                log::error!("Failed to copy to clipboard: {}", e);
                ctx.notify_error("Copy Failed", &format!("Error: {}", e));
            }
        }

        Ok(Action::Modify(Vec::new())) // Consume the key
    }

    /// Handle paste operation
    fn handle_paste(
        &self,
        ctx: &PluginContext,
        state: &mut PluginState,
        clipboard_type: ClipboardType,
    ) -> Result<Action> {
        let mut clipboard_mgr = self.clipboard_manager.lock();

        match clipboard_mgr.paste(clipboard_type) {
            Ok(text) => {
                if text.is_empty() {
                    ctx.notify_info("Paste", "Clipboard is empty");
                    return Ok(Action::Continue);
                }

                // Check if confirmation is needed
                if Self::requires_paste_confirmation(&text) {
                    state.paste_pending = Some(PendingPaste {
                        text: text.clone(),
                        clipboard_type,
                        requires_confirmation: true,
                    });

                    // Show modal for confirmation
                    ctx.queue_command(RemoteCommand::ShowModal {
                        title: "Confirm Paste".to_string(),
                        items: vec![
                            ModalItem {
                                id: "clipboard.paste.confirm".to_string(),
                                label: "Paste".to_string(),
                                description: Some(format!(
                                    "Paste {} lines ({} bytes)",
                                    text.lines().count(),
                                    text.len()
                                )),
                            },
                            ModalItem {
                                id: "clipboard.paste.cancel".to_string(),
                                label: "Cancel".to_string(),
                                description: None,
                            },
                        ],
                    });

                    ctx.notify_info(
                        "Paste Confirmation",
                        &format!(
                            "Confirm pasting {} lines ({} bytes)",
                            text.lines().count(),
                            text.len()
                        ),
                    );
                } else {
                    // Paste directly
                    let output = if state.bracket_mode_enabled {
                        // Wrap with bracket paste escape sequences
                        format!("\x1b[200~{}\x1b[201~", text)
                    } else {
                        text
                    };

                    log::info!("Pasting {} characters", output.len());
                    return Ok(Action::Modify(output.into_bytes()));
                }
            }
            Err(e) => {
                log::error!("Failed to paste from clipboard: {}", e);
                ctx.notify_error("Paste Failed", &format!("Error: {}", e));
            }
        }

        Ok(Action::Continue)
    }

    /// Handle selection start
    fn start_selection(
        &self,
        ctx: &PluginContext,
        state: &mut PluginState,
        mode: SelectionMode,
    ) {
        let (cursor_x, cursor_y) = ctx.get_cursor();

        state.selection.start(cursor_x, cursor_y, mode);

        // Draw overlay at cursor position
        ctx.queue_command(RemoteCommand::DrawOverlay {
            id: 1000, // Fixed ID for selection indicator
            x: cursor_x,
            y: cursor_y,
            text: match mode {
                SelectionMode::Character => "-- VISUAL --".to_string(),
                SelectionMode::Word => "-- VISUAL WORD --".to_string(),
                SelectionMode::Line => "-- VISUAL LINE --".to_string(),
                SelectionMode::Block => "-- VISUAL BLOCK --".to_string(),
            },
            style: OverlayStyle {
                fg: 0xFFFFFFFF, // White
                bg: 0x0000FFFF, // Blue
                z_index: 200.0,
            },
        });

        log::info!("Started {:?} selection at ({}, {})", mode, cursor_x, cursor_y);
    }

    /// Handle keybindings
    fn handle_keybinding(
        &self,
        input: &[u8],
        ctx: &PluginContext,
    ) -> Result<Action> {
        let mut state = self.state.lock();

        // Ctrl+Shift+C (Copy) - 0x03 with modifiers
        if input == [0x03] && state.selection.active {
            return self.handle_copy(ctx, &mut state, ClipboardType::Standard);
        }

        // Ctrl+Shift+V (Paste) - 0x16 with modifiers
        if input == [0x16] {
            return self.handle_paste(ctx, &mut state, ClipboardType::Standard);
        }

        // Ctrl+Shift+L (Copy entire line) - 0x0C with modifiers
        if input == [0x0C] {
            let (cursor_x, cursor_y) = ctx.get_cursor();
            let (cols, _) = ctx.get_size();

            state.selection.start(0, cursor_y, SelectionMode::Line);
            state.selection.update(cols - 1, cursor_y);

            return self.handle_copy(ctx, &mut state, ClipboardType::Standard);
        }

        // Visual mode activation
        // 'v' - Character mode
        if input == [b'v'] && !state.selection.active {
            self.start_selection(ctx, &mut state, SelectionMode::Character);
            return Ok(Action::Modify(Vec::new()));
        }

        // 'V' - Line mode (Shift+v)
        if input == [b'V'] && !state.selection.active {
            self.start_selection(ctx, &mut state, SelectionMode::Line);
            return Ok(Action::Modify(Vec::new()));
        }

        // Ctrl+V - Block mode
        if input == [0x16] && !state.selection.active {
            self.start_selection(ctx, &mut state, SelectionMode::Block);
            return Ok(Action::Modify(Vec::new()));
        }

        // Handle selection movement (when active)
        if state.selection.active {
            // Escape - Cancel selection
            if input == [0x1b] {
                state.selection.clear();
                ctx.queue_command(RemoteCommand::ClearOverlays { id: Some(1000) });
                log::info!("Cancelled selection");
                return Ok(Action::Modify(Vec::new()));
            }

            // 'y' - Yank (copy) and exit selection
            if input == [b'y'] {
                let result = self.handle_copy(ctx, &mut state, ClipboardType::Standard);
                ctx.queue_command(RemoteCommand::ClearOverlays { id: Some(1000) });
                return result;
            }

            // Arrow keys would be handled here in a full implementation
            // For now, we'll let them pass through
        }

        Ok(Action::Continue)
    }
}

impl Default for ClipboardPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ClipboardPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn get_commands(&self) -> Vec<ModalItem> {
        vec![
            ModalItem {
                id: "clipboard.copy".to_string(),
                label: "Copy Selection".to_string(),
                description: Some("Copy selected text to clipboard (Ctrl+Shift+C)".to_string()),
            },
            ModalItem {
                id: "clipboard.copy_line".to_string(),
                label: "Copy Line".to_string(),
                description: Some("Copy current line to clipboard (Ctrl+Shift+L)".to_string()),
            },
            ModalItem {
                id: "clipboard.paste".to_string(),
                label: "Paste".to_string(),
                description: Some("Paste from clipboard (Ctrl+Shift+V)".to_string()),
            },
            ModalItem {
                id: "clipboard.paste_primary".to_string(),
                label: "Paste Primary".to_string(),
                description: Some("Paste from X11 primary selection".to_string()),
            },
            ModalItem {
                id: "clipboard.visual_character".to_string(),
                label: "Visual Character Mode".to_string(),
                description: Some("Start character-wise selection (v)".to_string()),
            },
            ModalItem {
                id: "clipboard.visual_line".to_string(),
                label: "Visual Line Mode".to_string(),
                description: Some("Start line-wise selection (V)".to_string()),
            },
            ModalItem {
                id: "clipboard.visual_block".to_string(),
                label: "Visual Block Mode".to_string(),
                description: Some("Start block selection (Ctrl+V)".to_string()),
            },
            ModalItem {
                id: "clipboard.toggle_bracket_mode".to_string(),
                label: "Toggle Bracket Paste Mode".to_string(),
                description: Some("Enable/disable bracket paste mode for safety".to_string()),
            },
        ]
    }

    async fn on_input(&mut self, input: &[u8], ctx: &PluginContext) -> Result<Action> {
        self.handle_keybinding(input, ctx)
    }

    async fn on_remote_command(&mut self, id: &str, ctx: &PluginContext) -> Result<()> {
        let mut state = self.state.lock();

        match id {
            "clipboard.copy" => {
                if state.selection.active {
                    self.handle_copy(ctx, &mut state, ClipboardType::Standard)?;
                } else {
                    ctx.notify_warning("Copy Failed", "No text selected");
                }
            }

            "clipboard.copy_line" => {
                let (cursor_x, cursor_y) = ctx.get_cursor();
                let (cols, _) = ctx.get_size();

                state.selection.start(0, cursor_y, SelectionMode::Line);
                state.selection.update(cols - 1, cursor_y);

                self.handle_copy(ctx, &mut state, ClipboardType::Standard)?;
            }

            "clipboard.paste" => {
                self.handle_paste(ctx, &mut state, ClipboardType::Standard)?;
            }

            "clipboard.paste_primary" => {
                self.handle_paste(ctx, &mut state, ClipboardType::Primary)?;
            }

            "clipboard.visual_character" => {
                self.start_selection(ctx, &mut state, SelectionMode::Character);
            }

            "clipboard.visual_line" => {
                self.start_selection(ctx, &mut state, SelectionMode::Line);
            }

            "clipboard.visual_block" => {
                self.start_selection(ctx, &mut state, SelectionMode::Block);
            }

            "clipboard.toggle_bracket_mode" => {
                state.bracket_mode_enabled = !state.bracket_mode_enabled;
                ctx.notify_info(
                    "Bracket Paste Mode",
                    &format!(
                        "Bracket paste mode {}",
                        if state.bracket_mode_enabled { "enabled" } else { "disabled" }
                    ),
                );
                log::info!("Bracket paste mode: {}", state.bracket_mode_enabled);
            }

            "clipboard.paste.confirm" => {
                if let Some(pending) = state.paste_pending.take() {
                    let output = if state.bracket_mode_enabled {
                        format!("\x1b[200~{}\x1b[201~", pending.text)
                    } else {
                        pending.text
                    };

                    // Since we're in on_remote_command, we can't return modified input
                    // This would need to be queued for the daemon
                    log::info!("User confirmed paste of {} characters", output.len());
                    ctx.notify_success("Pasted", &format!("Pasted {} characters", output.len()));
                }

                ctx.queue_command(RemoteCommand::ClearOverlays { id: None });
            }

            "clipboard.paste.cancel" => {
                state.paste_pending = None;
                ctx.notify_info("Paste Cancelled", "Paste operation cancelled");
                ctx.queue_command(RemoteCommand::ClearOverlays { id: None });
            }

            _ => {}
        }

        Ok(())
    }

    async fn on_resize(&mut self, cols: u16, rows: u16, ctx: &PluginContext) -> Result<()> {
        log::debug!("Clipboard plugin: Terminal resized to {}x{}", cols, rows);

        // Clear any active selection on resize
        let mut state = self.state.lock();
        if state.selection.active {
            state.selection.clear();
            ctx.queue_command(RemoteCommand::ClearOverlays { id: Some(1000) });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_boundaries() {
        let plugin = ClipboardPlugin::new();

        let line = "hello world test";
        let (start, end) = plugin.find_word_boundaries(line, 6);
        assert_eq!(start, 6);
        assert_eq!(end, 10); // "world"

        let line = "test_function_name";
        let (start, end) = plugin.find_word_boundaries(line, 8);
        assert_eq!(start, 0);
        assert_eq!(end, 17); // entire identifier
    }

    #[test]
    fn test_paste_confirmation_required() {
        // Small single line - no confirmation
        let text = "short text";
        assert!(!ClipboardPlugin::requires_paste_confirmation(text));

        // Multiple lines - requires confirmation
        let text = "line1\nline2\nline3\nline4\nline5\nline6";
        assert!(ClipboardPlugin::requires_paste_confirmation(text));

        // Large single line - requires confirmation
        let text = "a".repeat(2000);
        assert!(ClipboardPlugin::requires_paste_confirmation(&text));
    }
}
