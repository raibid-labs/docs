use scarab_protocol::{Cell, SharedState, GRID_HEIGHT, GRID_WIDTH};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
/// VTE (Virtual Terminal Emulator) Parser Integration
///
/// This module implements the VTE parser to handle ANSI escape sequences
/// and update the SharedState grid with proper terminal emulation.
///
/// Features:
/// - Parse ANSI/VT100 escape sequences
/// - Handle cursor positioning and scrolling
/// - Support colors and text attributes
/// - UTF-8 multibyte character handling
/// - Scrollback buffer (10k lines)
use vte::{Parser, Perform};

/// Maximum scrollback buffer size (10,000 lines)
const SCROLLBACK_SIZE: usize = 10_000;

/// Default colors
const DEFAULT_FG: u32 = 0xFFCCCCCC; // Light gray
const DEFAULT_BG: u32 = 0xFF000000; // Black

/// Text attribute flags
pub const FLAG_BOLD: u8 = 1 << 0;
pub const FLAG_ITALIC: u8 = 1 << 1;
pub const FLAG_UNDERLINE: u8 = 1 << 2;
pub const FLAG_INVERSE: u8 = 1 << 3;
pub const FLAG_DIM: u8 = 1 << 4;

/// Current text attributes for rendering
#[derive(Clone, Copy, Debug)]
struct TextAttributes {
    fg: u32,
    bg: u32,
    flags: u8,
}

impl Default for TextAttributes {
    fn default() -> Self {
        Self {
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            flags: 0,
        }
    }
}

/// Terminal state manager that implements the VTE Perform trait
pub struct TerminalState {
    /// Pointer to shared memory state
    shared_ptr: *mut SharedState,
    /// VTE parser instance
    parser: Parser,
    /// Current cursor position (0-indexed)
    cursor_x: u16,
    cursor_y: u16,
    /// Current text attributes
    attrs: TextAttributes,
    /// Scrollback buffer (stores lines that scrolled off the top)
    scrollback: VecDeque<Vec<Cell>>,
    /// Sequence counter for atomic updates
    sequence_counter: Arc<AtomicU64>,
    /// Saved cursor position (for DECSC/DECRC)
    saved_cursor: (u16, u16),
    saved_attrs: TextAttributes,
}

impl TerminalState {
    /// Create a new terminal state manager
    pub fn new(shared_ptr: *mut SharedState, sequence_counter: Arc<AtomicU64>) -> Self {
        Self {
            shared_ptr,
            parser: Parser::new(),
            cursor_x: 0,
            cursor_y: 0,
            attrs: TextAttributes::default(),
            scrollback: VecDeque::with_capacity(SCROLLBACK_SIZE),
            sequence_counter,
            saved_cursor: (0, 0),
            saved_attrs: TextAttributes::default(),
        }
    }

    /// Process PTY output through the VTE parser
    pub fn process_output(&mut self, data: &[u8]) {
        // Take ownership of the parser temporarily to satisfy borrow checker
        let mut parser = std::mem::replace(&mut self.parser, vte::Parser::new());

        for byte in data {
            parser.advance(self, *byte);
        }

        // Restore the parser
        self.parser = parser;
        self.mark_dirty();
    }

    /// Mark shared state as dirty and increment sequence number
    fn mark_dirty(&mut self) {
        unsafe {
            let state = &mut *self.shared_ptr;
            state.dirty_flag = 1;
            state.cursor_x = self.cursor_x;
            state.cursor_y = self.cursor_y;
            let new_seq = self.sequence_counter.fetch_add(1, Ordering::SeqCst) + 1;
            state.sequence_number = new_seq;
        }
    }

    /// Write a character at the current cursor position
    fn write_char(&mut self, c: char) {
        if self.cursor_x >= GRID_WIDTH as u16 {
            // Handle line wrapping
            self.cursor_x = 0;
            self.cursor_y += 1;
        }

        if self.cursor_y >= GRID_HEIGHT as u16 {
            // Scroll up
            self.scroll_up(1);
        }

        let index = (self.cursor_y as usize * GRID_WIDTH) + self.cursor_x as usize;

        unsafe {
            let state = &mut *self.shared_ptr;
            if index < state.cells.len() {
                state.cells[index] = Cell {
                    char_codepoint: c as u32,
                    fg: self.attrs.fg,
                    bg: self.attrs.bg,
                    flags: self.attrs.flags,
                    _padding: [0; 3],
                };
            }
        }

        self.cursor_x += 1;
    }

    /// Scroll the screen up by n lines
    fn scroll_up(&mut self, lines: usize) {
        unsafe {
            let state = &mut *self.shared_ptr;

            // Save scrolled lines to scrollback buffer
            for i in 0..lines {
                if i >= GRID_HEIGHT {
                    break;
                }
                let mut line = Vec::with_capacity(GRID_WIDTH);
                for x in 0..GRID_WIDTH {
                    let idx = i * GRID_WIDTH + x;
                    line.push(state.cells[idx]);
                }
                self.scrollback.push_back(line);

                // Limit scrollback buffer size
                if self.scrollback.len() > SCROLLBACK_SIZE {
                    self.scrollback.pop_front();
                }
            }

            // Shift grid content up
            for y in 0..(GRID_HEIGHT - lines) {
                for x in 0..GRID_WIDTH {
                    let src_idx = (y + lines) * GRID_WIDTH + x;
                    let dst_idx = y * GRID_WIDTH + x;
                    state.cells[dst_idx] = state.cells[src_idx];
                }
            }

            // Clear the bottom lines
            for y in (GRID_HEIGHT - lines)..GRID_HEIGHT {
                for x in 0..GRID_WIDTH {
                    let idx = y * GRID_WIDTH + x;
                    state.cells[idx] = Cell {
                        char_codepoint: 0,
                        fg: DEFAULT_FG,
                        bg: DEFAULT_BG,
                        flags: 0,
                        _padding: [0; 3],
                    };
                }
            }

            // Adjust cursor position
            if self.cursor_y >= lines as u16 {
                self.cursor_y -= lines as u16;
            } else {
                self.cursor_y = 0;
            }
        }
    }

    /// Clear the screen
    fn clear_screen(&mut self) {
        unsafe {
            let state = &mut *self.shared_ptr;
            for cell in state.cells.iter_mut() {
                *cell = Cell {
                    char_codepoint: 0,
                    fg: DEFAULT_FG,
                    bg: DEFAULT_BG,
                    flags: 0,
                    _padding: [0; 3],
                };
            }
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    /// Clear from cursor to end of line
    fn clear_to_eol(&mut self) {
        unsafe {
            let state = &mut *self.shared_ptr;
            for x in self.cursor_x as usize..GRID_WIDTH {
                let idx = self.cursor_y as usize * GRID_WIDTH + x;
                state.cells[idx] = Cell {
                    char_codepoint: 0,
                    fg: DEFAULT_FG,
                    bg: DEFAULT_BG,
                    flags: 0,
                    _padding: [0; 3],
                };
            }
        }
    }

    /// Set SGR (Select Graphic Rendition) attributes
    fn set_sgr(&mut self, params: &[i64]) {
        if params.is_empty() {
            // Reset all attributes
            self.attrs = TextAttributes::default();
            return;
        }

        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => self.attrs = TextAttributes::default(),
                1 => self.attrs.flags |= FLAG_BOLD,
                2 => self.attrs.flags |= FLAG_DIM,
                3 => self.attrs.flags |= FLAG_ITALIC,
                4 => self.attrs.flags |= FLAG_UNDERLINE,
                7 => self.attrs.flags |= FLAG_INVERSE,
                22 => self.attrs.flags &= !(FLAG_BOLD | FLAG_DIM),
                23 => self.attrs.flags &= !FLAG_ITALIC,
                24 => self.attrs.flags &= !FLAG_UNDERLINE,
                27 => self.attrs.flags &= !FLAG_INVERSE,

                // Foreground colors (30-37, 90-97)
                30..=37 => self.attrs.fg = ansi_color_to_rgba(params[i] as u8 - 30),
                90..=97 => self.attrs.fg = ansi_bright_color_to_rgba(params[i] as u8 - 90),

                // Background colors (40-47, 100-107)
                40..=47 => self.attrs.bg = ansi_color_to_rgba(params[i] as u8 - 40),
                100..=107 => self.attrs.bg = ansi_bright_color_to_rgba(params[i] as u8 - 100),

                // 256-color mode (38;5;n for fg, 48;5;n for bg)
                38 | 48 => {
                    if i + 2 < params.len() && params[i + 1] == 5 {
                        let color = color_256_to_rgba(params[i + 2] as u8);
                        if params[i] == 38 {
                            self.attrs.fg = color;
                        } else {
                            self.attrs.bg = color;
                        }
                        i += 2;
                    }
                }

                // Default colors
                39 => self.attrs.fg = DEFAULT_FG,
                49 => self.attrs.bg = DEFAULT_BG,

                _ => {} // Ignore unknown codes
            }
            i += 1;
        }
    }
}

impl Perform for TerminalState {
    fn print(&mut self, c: char) {
        self.write_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => {
                // Backspace
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            0x09 => {
                // Tab (move to next tab stop, typically 8 spaces)
                self.cursor_x = ((self.cursor_x / 8) + 1) * 8;
                if self.cursor_x >= GRID_WIDTH as u16 {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                }
            }
            0x0A => {
                // Line Feed
                self.cursor_y += 1;
                if self.cursor_y >= GRID_HEIGHT as u16 {
                    self.scroll_up(1);
                }
            }
            0x0D => {
                // Carriage Return
                self.cursor_x = 0;
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
    }

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let params: Vec<i64> = params.iter().map(|p| p[0] as i64).collect();

        match action {
            'A' => {
                // Cursor Up
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                self.cursor_y = self.cursor_y.saturating_sub(n);
            }
            'B' => {
                // Cursor Down
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                self.cursor_y = (self.cursor_y + n).min(GRID_HEIGHT as u16 - 1);
            }
            'C' => {
                // Cursor Forward
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                self.cursor_x = (self.cursor_x + n).min(GRID_WIDTH as u16 - 1);
            }
            'D' => {
                // Cursor Back
                let n = params.get(0).copied().unwrap_or(1).max(1) as u16;
                self.cursor_x = self.cursor_x.saturating_sub(n);
            }
            'H' | 'f' => {
                // Cursor Position
                let row = params.get(0).copied().unwrap_or(1).max(1) as u16 - 1;
                let col = params.get(1).copied().unwrap_or(1).max(1) as u16 - 1;
                self.cursor_y = row.min(GRID_HEIGHT as u16 - 1);
                self.cursor_x = col.min(GRID_WIDTH as u16 - 1);
            }
            'J' => {
                // Erase in Display
                let n = params.get(0).copied().unwrap_or(0);
                match n {
                    0 => {
                        // Clear from cursor to end of screen
                        self.clear_to_eol();
                        for y in (self.cursor_y as usize + 1)..GRID_HEIGHT {
                            for x in 0..GRID_WIDTH {
                                unsafe {
                                    let state = &mut *self.shared_ptr;
                                    let idx = y * GRID_WIDTH + x;
                                    state.cells[idx] = Cell::default();
                                }
                            }
                        }
                    }
                    1 => {
                        // Clear from cursor to beginning of screen
                        for y in 0..self.cursor_y as usize {
                            for x in 0..GRID_WIDTH {
                                unsafe {
                                    let state = &mut *self.shared_ptr;
                                    let idx = y * GRID_WIDTH + x;
                                    state.cells[idx] = Cell::default();
                                }
                            }
                        }
                    }
                    2 => {
                        // Clear entire screen
                        self.clear_screen();
                    }
                    _ => {}
                }
            }
            'K' => {
                // Erase in Line
                let n = params.get(0).copied().unwrap_or(0);
                match n {
                    0 => self.clear_to_eol(),
                    1 => {
                        // Clear from beginning of line to cursor
                        unsafe {
                            let state = &mut *self.shared_ptr;
                            for x in 0..=self.cursor_x as usize {
                                let idx = self.cursor_y as usize * GRID_WIDTH + x;
                                state.cells[idx] = Cell::default();
                            }
                        }
                    }
                    2 => {
                        // Clear entire line
                        unsafe {
                            let state = &mut *self.shared_ptr;
                            for x in 0..GRID_WIDTH {
                                let idx = self.cursor_y as usize * GRID_WIDTH + x;
                                state.cells[idx] = Cell::default();
                            }
                        }
                    }
                    _ => {}
                }
            }
            'm' => {
                // SGR (Select Graphic Rendition)
                self.set_sgr(&params);
            }
            's' => {
                // Save cursor position (DECSC)
                self.saved_cursor = (self.cursor_x, self.cursor_y);
                self.saved_attrs = self.attrs;
            }
            'u' => {
                // Restore cursor position (DECRC)
                self.cursor_x = self.saved_cursor.0;
                self.cursor_y = self.saved_cursor.1;
                self.attrs = self.saved_attrs;
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

/// Convert ANSI color index (0-7) to RGBA
fn ansi_color_to_rgba(index: u8) -> u32 {
    match index {
        0 => 0xFF000000, // Black
        1 => 0xFFCD0000, // Red
        2 => 0xFF00CD00, // Green
        3 => 0xFFCDCD00, // Yellow
        4 => 0xFF0000EE, // Blue
        5 => 0xFFCD00CD, // Magenta
        6 => 0xFF00CDCD, // Cyan
        7 => 0xFFE5E5E5, // White
        _ => DEFAULT_FG,
    }
}

/// Convert ANSI bright color index (0-7) to RGBA
fn ansi_bright_color_to_rgba(index: u8) -> u32 {
    match index {
        0 => 0xFF7F7F7F, // Bright Black (Gray)
        1 => 0xFFFF0000, // Bright Red
        2 => 0xFF00FF00, // Bright Green
        3 => 0xFFFFFF00, // Bright Yellow
        4 => 0xFF5C5CFF, // Bright Blue
        5 => 0xFFFF00FF, // Bright Magenta
        6 => 0xFF00FFFF, // Bright Cyan
        7 => 0xFFFFFFFF, // Bright White
        _ => DEFAULT_FG,
    }
}

/// Convert 256-color palette index to RGBA
fn color_256_to_rgba(index: u8) -> u32 {
    match index {
        // 0-15: Standard ANSI colors
        0..=7 => ansi_color_to_rgba(index),
        8..=15 => ansi_bright_color_to_rgba(index - 8),

        // 16-231: 6x6x6 color cube
        16..=231 => {
            let idx = index - 16;
            let r = (idx / 36) * 51;
            let g = ((idx % 36) / 6) * 51;
            let b = (idx % 6) * 51;
            0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
        }

        // 232-255: Grayscale
        232..=255 => {
            let gray = 8 + (index - 232) * 10;
            0xFF000000 | ((gray as u32) << 16) | ((gray as u32) << 8) | (gray as u32)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_color_conversion() {
        assert_eq!(ansi_color_to_rgba(0), 0xFF000000); // Black
        assert_eq!(ansi_color_to_rgba(1), 0xFFCD0000); // Red
        assert_eq!(ansi_color_to_rgba(7), 0xFFE5E5E5); // White
    }

    #[test]
    fn test_256_color_conversion() {
        // Test standard colors
        assert_eq!(color_256_to_rgba(0), ansi_color_to_rgba(0));

        // Test grayscale
        let gray = color_256_to_rgba(232);
        assert!(gray != 0);
    }
}
