/// Comprehensive unit tests for VTE parser integration
///
/// Tests cover:
/// - Basic text printing
/// - ANSI color codes (8, 16, 256 colors)
/// - Cursor movement and positioning
/// - Screen clearing and line clearing
/// - Text attributes (bold, italic, underline, inverse)
/// - Scrollback buffer
/// - UTF-8 multibyte characters
/// - Edge cases and error handling

#[cfg(test)]
mod tests {
    use crate::vte::*;
    use scarab_protocol::{Cell, SharedState, GRID_HEIGHT, GRID_WIDTH};
    use std::sync::atomic::AtomicU64;
    use std::sync::Arc;

    /// Helper to create a test terminal state
    fn create_test_terminal() -> (*mut SharedState, TerminalState, Box<SharedState>) {
        let mut state = Box::new(unsafe { std::mem::zeroed::<SharedState>() });
        let ptr = state.as_mut() as *mut SharedState;
        let sequence_counter = Arc::new(AtomicU64::new(0));
        let terminal = TerminalState::new(ptr, sequence_counter);
        (ptr, terminal, state)
    }

    /// Helper to get cell content at position
    fn get_cell(state: &SharedState, x: usize, y: usize) -> &Cell {
        let idx = y * GRID_WIDTH + x;
        &state.cells[idx]
    }

    #[test]
    fn test_basic_text_printing() {
        let (_, mut terminal, state) = create_test_terminal();

        // Print "Hello"
        terminal.process_output(b"Hello");

        // Verify characters were written
        assert_eq!(get_cell(&state, 0, 0).char_codepoint, 'H' as u32);
        assert_eq!(get_cell(&state, 1, 0).char_codepoint, 'e' as u32);
        assert_eq!(get_cell(&state, 2, 0).char_codepoint, 'l' as u32);
        assert_eq!(get_cell(&state, 3, 0).char_codepoint, 'l' as u32);
        assert_eq!(get_cell(&state, 4, 0).char_codepoint, 'o' as u32);

        // Cursor should be at position 5
        assert_eq!(state.cursor_x, 5);
        assert_eq!(state.cursor_y, 0);
    }

    #[test]
    fn test_newline_and_carriage_return() {
        let (_, mut terminal, state) = create_test_terminal();

        // Print "Line1\nLine2\n"
        terminal.process_output(b"Line1\nLine2\n");

        // Verify first line
        assert_eq!(get_cell(&state, 0, 0).char_codepoint, 'L' as u32);
        assert_eq!(get_cell(&state, 4, 0).char_codepoint, '1' as u32);

        // Verify second line
        assert_eq!(get_cell(&state, 0, 1).char_codepoint, 'L' as u32);
        assert_eq!(get_cell(&state, 4, 1).char_codepoint, '2' as u32);

        // Cursor should be on line 3
        assert_eq!(state.cursor_y, 2);
    }

    #[test]
    fn test_carriage_return() {
        let (_, mut terminal, state) = create_test_terminal();

        // Print "Hello\rWorld"
        terminal.process_output(b"Hello\rWorld");

        // "World" should overwrite "Hello" at the beginning
        assert_eq!(get_cell(&state, 0, 0).char_codepoint, 'W' as u32);
        assert_eq!(get_cell(&state, 1, 0).char_codepoint, 'o' as u32);
        assert_eq!(get_cell(&state, 2, 0).char_codepoint, 'r' as u32);
        assert_eq!(get_cell(&state, 3, 0).char_codepoint, 'l' as u32);
        assert_eq!(get_cell(&state, 4, 0).char_codepoint, 'd' as u32);

        // Cursor should be at position 5
        assert_eq!(state.cursor_x, 5);
    }

    #[test]
    fn test_ansi_colors() {
        let (_, mut terminal, state) = create_test_terminal();

        // Red text
        terminal.process_output(b"\x1b[31mRed\x1b[0m");

        // Check that "Red" has red foreground color
        let red_cell = get_cell(&state, 0, 0);
        assert_eq!(red_cell.char_codepoint, 'R' as u32);
        assert_eq!(red_cell.fg, 0xFFCD0000); // Red color

        // After reset, should use default color
        terminal.process_output(b"Normal");
        let normal_cell = get_cell(&state, 3, 0);
        assert_eq!(normal_cell.fg, 0xFFCCCCCC); // Default foreground
    }

    #[test]
    fn test_bright_colors() {
        let (_, mut terminal, state) = create_test_terminal();

        // Bright red text
        terminal.process_output(b"\x1b[91mBrightRed\x1b[0m");

        // Check bright red color
        let cell = get_cell(&state, 0, 0);
        assert_eq!(cell.fg, 0xFFFF0000); // Bright red
    }

    #[test]
    fn test_256_colors() {
        let (_, mut terminal, state) = create_test_terminal();

        // 256-color mode (color index 196 = bright red)
        terminal.process_output(b"\x1b[38;5;196mColor\x1b[0m");

        // Verify the character was written
        let cell = get_cell(&state, 0, 0);
        assert_eq!(cell.char_codepoint, 'C' as u32);
        // Color should be non-default
        assert_ne!(cell.fg, 0xFFCCCCCC);
    }

    #[test]
    fn test_background_colors() {
        let (_, mut terminal, state) = create_test_terminal();

        // Blue background
        terminal.process_output(b"\x1b[44mBlue BG\x1b[0m");

        // Check background color
        let cell = get_cell(&state, 0, 0);
        assert_eq!(cell.bg, 0xFF0000EE); // Blue background
    }

    #[test]
    fn test_text_attributes() {
        let (_, mut terminal, state) = create_test_terminal();

        // Bold text
        terminal.process_output(b"\x1b[1mBold\x1b[0m");
        let bold_cell = get_cell(&state, 0, 0);
        assert_eq!(bold_cell.flags & FLAG_BOLD, FLAG_BOLD);

        // Italic text
        terminal.process_output(b" \x1b[3mItalic\x1b[0m");
        let italic_cell = get_cell(&state, 5, 0);
        assert_eq!(italic_cell.flags & FLAG_ITALIC, FLAG_ITALIC);

        // Underline text
        terminal.process_output(b" \x1b[4mUnder\x1b[0m");
        let under_cell = get_cell(&state, 12, 0);
        assert_eq!(under_cell.flags & FLAG_UNDERLINE, FLAG_UNDERLINE);
    }

    #[test]
    fn test_cursor_movement() {
        let (_, mut terminal, state) = create_test_terminal();

        // Move cursor to position (5, 3)
        terminal.process_output(b"\x1b[4;6H");

        assert_eq!(state.cursor_x, 5);
        assert_eq!(state.cursor_y, 3);

        // Write at that position
        terminal.process_output(b"X");
        let cell = get_cell(&state, 5, 3);
        assert_eq!(cell.char_codepoint, 'X' as u32);
    }

    #[test]
    fn test_cursor_up_down_left_right() {
        let (_, mut terminal, state) = create_test_terminal();

        // Start at (0, 0), move down 5 times
        terminal.process_output(b"\x1b[5B");
        assert_eq!(state.cursor_y, 5);

        // Move right 10 times
        terminal.process_output(b"\x1b[10C");
        assert_eq!(state.cursor_x, 10);

        // Move up 2 times
        terminal.process_output(b"\x1b[2A");
        assert_eq!(state.cursor_y, 3);

        // Move left 5 times
        terminal.process_output(b"\x1b[5D");
        assert_eq!(state.cursor_x, 5);
    }

    #[test]
    fn test_clear_screen() {
        let (_, mut terminal, state) = create_test_terminal();

        // Fill screen with text
        terminal.process_output(b"Hello World");

        // Clear screen
        terminal.process_output(b"\x1b[2J");

        // All cells should be empty
        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let cell = get_cell(&state, x, y);
                assert_eq!(cell.char_codepoint, 0);
            }
        }

        // Cursor should be at origin
        assert_eq!(state.cursor_x, 0);
        assert_eq!(state.cursor_y, 0);
    }

    #[test]
    fn test_clear_line() {
        let (_, mut terminal, state) = create_test_terminal();

        // Write a line
        terminal.process_output(b"Hello World");

        // Move cursor back
        terminal.process_output(b"\x1b[6D");

        // Clear to end of line
        terminal.process_output(b"\x1b[K");

        // "Hello " should remain, "World" should be cleared
        assert_eq!(get_cell(&state, 0, 0).char_codepoint, 'H' as u32);
        assert_eq!(get_cell(&state, 5, 0).char_codepoint, ' ' as u32);
        assert_eq!(get_cell(&state, 6, 0).char_codepoint, 0);
    }

    #[test]
    fn test_save_restore_cursor() {
        let (_, mut terminal, state) = create_test_terminal();

        // Move to (10, 5)
        terminal.process_output(b"\x1b[6;11H");

        // Save cursor position
        terminal.process_output(b"\x1b[s");

        // Move elsewhere
        terminal.process_output(b"\x1b[1;1H");
        assert_eq!(state.cursor_x, 0);
        assert_eq!(state.cursor_y, 0);

        // Restore cursor
        terminal.process_output(b"\x1b[u");
        assert_eq!(state.cursor_x, 10);
        assert_eq!(state.cursor_y, 5);
    }

    #[test]
    fn test_line_wrapping() {
        let (_, mut terminal, state) = create_test_terminal();

        // Create a string longer than screen width
        let long_text = "A".repeat(GRID_WIDTH + 10);
        terminal.process_output(long_text.as_bytes());

        // First line should be full
        for x in 0..GRID_WIDTH {
            assert_eq!(get_cell(&state, x, 0).char_codepoint, 'A' as u32);
        }

        // Text should wrap to second line
        for x in 0..10 {
            assert_eq!(get_cell(&state, x, 1).char_codepoint, 'A' as u32);
        }
    }

    #[test]
    fn test_scrolling() {
        let (_, mut terminal, state) = create_test_terminal();

        // Fill screen with numbered lines
        for i in 0..GRID_HEIGHT + 5 {
            terminal.process_output(format!("Line {}\n", i).as_bytes());
        }

        // First line should contain "Line 5" (lines 0-4 scrolled off)
        assert_eq!(get_cell(&state, 0, 0).char_codepoint, 'L' as u32);

        // Cursor should be near bottom
        assert!(state.cursor_y >= GRID_HEIGHT as u16 - 10);
    }

    #[test]
    fn test_utf8_characters() {
        let (_, mut terminal, state) = create_test_terminal();

        // Test various UTF-8 characters
        terminal.process_output("Hello 世界! 🚀".as_bytes());

        // Verify ASCII characters
        assert_eq!(get_cell(&state, 0, 0).char_codepoint, 'H' as u32);

        // Verify UTF-8 characters were stored
        // Note: The actual positions may vary due to character width handling
        let mut has_chinese = false;
        for x in 0..20 {
            let codepoint = get_cell(&state, x, 0).char_codepoint;
            if codepoint == '世' as u32 || codepoint == '界' as u32 {
                has_chinese = true;
                break;
            }
        }
        assert!(has_chinese, "Should contain Chinese characters");
    }

    #[test]
    fn test_tab_character() {
        let (_, mut terminal, state) = create_test_terminal();

        // Tab should advance to next tab stop (every 8 columns)
        terminal.process_output(b"A\tB");

        assert_eq!(get_cell(&state, 0, 0).char_codepoint, 'A' as u32);
        assert_eq!(get_cell(&state, 8, 0).char_codepoint, 'B' as u32);
    }

    #[test]
    fn test_backspace() {
        let (_, mut terminal, state) = create_test_terminal();

        terminal.process_output(b"Hello\x08\x08World");

        // "World" should overwrite last 2 chars of "Hello"
        assert_eq!(get_cell(&state, 0, 0).char_codepoint, 'H' as u32);
        assert_eq!(get_cell(&state, 1, 0).char_codepoint, 'e' as u32);
        assert_eq!(get_cell(&state, 2, 0).char_codepoint, 'l' as u32);
        assert_eq!(get_cell(&state, 3, 0).char_codepoint, 'W' as u32);
    }

    #[test]
    fn test_combined_attributes() {
        let (_, mut terminal, state) = create_test_terminal();

        // Bold + Red + Underline
        terminal.process_output(b"\x1b[1;31;4mBoldRedUnder\x1b[0m");

        let cell = get_cell(&state, 0, 0);
        assert_eq!(cell.flags & FLAG_BOLD, FLAG_BOLD);
        assert_eq!(cell.flags & FLAG_UNDERLINE, FLAG_UNDERLINE);
        assert_eq!(cell.fg, 0xFFCD0000); // Red
    }

    #[test]
    fn test_inverse_attribute() {
        let (_, mut terminal, state) = create_test_terminal();

        // Inverse text (swap fg/bg)
        terminal.process_output(b"\x1b[7mInverse\x1b[0m");

        let cell = get_cell(&state, 0, 0);
        assert_eq!(cell.flags & FLAG_INVERSE, FLAG_INVERSE);
    }

    #[test]
    fn test_partial_escape_sequences() {
        let (_, mut terminal, state) = create_test_terminal();

        // Send escape sequence in chunks
        terminal.process_output(b"\x1b");
        terminal.process_output(b"[");
        terminal.process_output(b"31");
        terminal.process_output(b"m");
        terminal.process_output(b"Red");

        let cell = get_cell(&state, 0, 0);
        assert_eq!(cell.char_codepoint, 'R' as u32);
        assert_eq!(cell.fg, 0xFFCD0000); // Red
    }

    #[test]
    fn test_dirty_flag_and_sequence() {
        let (_, mut terminal, state) = create_test_terminal();

        let initial_seq = state.sequence_number;

        // Process some output
        terminal.process_output(b"Test");

        // Dirty flag should be set
        assert_eq!(state.dirty_flag, 1);

        // Sequence number should have incremented
        assert!(state.sequence_number > initial_seq);
    }

    #[test]
    fn test_cursor_bounds_checking() {
        let (_, mut terminal, state) = create_test_terminal();

        // Try to move cursor beyond screen bounds
        terminal.process_output(format!("\x1b[1000;1000H").as_bytes());

        // Cursor should be clamped to screen size
        assert!(state.cursor_x < GRID_WIDTH as u16);
        assert!(state.cursor_y < GRID_HEIGHT as u16);
    }

    #[test]
    fn test_large_output_performance() {
        let (_, mut terminal, _state) = create_test_terminal();

        // Generate large output (simulate scrolling)
        let large_text = "A".repeat(100_000);

        // This should not panic or cause performance issues
        terminal.process_output(large_text.as_bytes());

        // Test completed successfully if we reach here
    }
}
