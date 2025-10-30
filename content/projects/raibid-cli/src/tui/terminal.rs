//! Terminal setup and cleanup utilities
//!
//! This module handles terminal initialization, configuration, and restoration.

use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal as RatatuiTerminal};
use std::io::{self, Stdout};

/// Minimum terminal dimensions
pub const MIN_WIDTH: u16 = 80;
pub const MIN_HEIGHT: u16 = 24;

/// Type alias for our terminal
pub type Terminal = RatatuiTerminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal for TUI rendering
///
/// This function:
/// - Checks minimum terminal size
/// - Enables raw mode
/// - Enters alternate screen
/// - Creates and returns a configured terminal
pub fn init() -> Result<Terminal> {
    // Check terminal size
    let (width, height) = crossterm::terminal::size().context("Failed to get terminal size")?;

    if width < MIN_WIDTH || height < MIN_HEIGHT {
        anyhow::bail!(
            "Terminal too small. Minimum size: {}x{}, current size: {}x{}",
            MIN_WIDTH,
            MIN_HEIGHT,
            width,
            height
        );
    }

    // Enable raw mode
    enable_raw_mode().context("Failed to enable raw mode")?;

    // Enter alternate screen
    execute!(io::stdout(), EnterAlternateScreen).context("Failed to enter alternate screen")?;

    // Create terminal
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = RatatuiTerminal::new(backend).context("Failed to create terminal")?;

    Ok(terminal)
}

/// Restore the terminal to its original state
///
/// This function:
/// - Leaves alternate screen
/// - Disables raw mode
pub fn restore() -> Result<()> {
    // Leave alternate screen
    execute!(io::stdout(), LeaveAlternateScreen).context("Failed to leave alternate screen")?;

    // Disable raw mode
    disable_raw_mode().context("Failed to disable raw mode")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_size_constants() {
        assert_eq!(MIN_WIDTH, 80);
        assert_eq!(MIN_HEIGHT, 24);
    }
}
