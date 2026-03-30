//! # fusabi-tui-core
//!
//! Core TUI infrastructure for Ratatui-based Fusabi applications.
//!
//! This crate provides the foundational building blocks for terminal user interfaces
//! in the Fusabi ecosystem:
//!
//! - Event loop abstraction
//! - State management primitives
//! - Input handling framework
//! - Async command dispatch
//!
//! ## Usage
//!
//! This crate is designed to be used alongside `fusabi-tui-widgets` for building
//! complete TUI applications. The typical pattern is:
//!
//! 1. Define your application state implementing [`AppState`]
//! 2. Create a [`TerminalWrapper`]
//! 3. Handle events with [`poll_event`] in your event loop

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Stdout};
use std::time::Duration;

// ============================================================================
// Terminal Management
// ============================================================================

/// Wrapper around the terminal with setup/teardown.
pub struct TerminalWrapper {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TerminalWrapper {
    /// Initialize the terminal for TUI rendering.
    pub fn new() -> Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    /// Get a mutable reference to the underlying terminal.
    pub fn terminal(&mut self) -> &mut Terminal<CrosstermBackend<Stdout>> {
        &mut self.terminal
    }

    /// Restore terminal to normal state.
    pub fn restore(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for TerminalWrapper {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

// ============================================================================
// Input Events
// ============================================================================

/// Application-level input events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppEvent {
    /// A key was pressed
    Key(KeyEvent),
    /// Terminal was resized
    Resize(u16, u16),
    /// Tick for periodic updates
    Tick,
    /// Quit the application
    Quit,
}

/// Poll for input events with a timeout.
pub fn poll_event(timeout: Duration) -> Result<Option<AppEvent>> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => {
                // Handle Ctrl+C as quit
                if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                    return Ok(Some(AppEvent::Quit));
                }
                Ok(Some(AppEvent::Key(key)))
            }
            Event::Resize(w, h) => Ok(Some(AppEvent::Resize(w, h))),
            _ => Ok(None),
        }
    } else {
        Ok(Some(AppEvent::Tick))
    }
}

// ============================================================================
// Application State Trait
// ============================================================================

/// Trait for application state management.
///
/// Implement this trait for your application's main state struct.
/// The event loop will call these methods to update and render your app.
pub trait AppState {
    /// Handle an input event and return whether the app should continue running.
    fn handle_event(&mut self, event: AppEvent) -> bool;

    /// Check if the application should quit.
    fn should_quit(&self) -> bool;
}

// ============================================================================
// Focus and Navigation
// ============================================================================

/// Represents which pane/component has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FocusedPane {
    #[default]
    StreamList,
    ItemList,
    Preview,
    Omnibar,
}

impl FocusedPane {
    /// Move focus to the next pane (left to right).
    pub fn next(self) -> Self {
        match self {
            Self::StreamList => Self::ItemList,
            Self::ItemList => Self::Preview,
            Self::Preview => Self::StreamList,
            Self::Omnibar => Self::StreamList,
        }
    }

    /// Move focus to the previous pane (right to left).
    pub fn prev(self) -> Self {
        match self {
            Self::StreamList => Self::Preview,
            Self::ItemList => Self::StreamList,
            Self::Preview => Self::ItemList,
            Self::Omnibar => Self::StreamList,
        }
    }
}

// ============================================================================
// List State Helper
// ============================================================================

/// Simple list state for tracking selection in lists.
#[derive(Debug, Clone, Default)]
pub struct ListState {
    /// Currently selected index
    pub selected: Option<usize>,
    /// Total number of items
    pub len: usize,
    /// Scroll offset for virtual scrolling
    pub offset: usize,
}

impl ListState {
    pub fn new(len: usize) -> Self {
        Self {
            selected: if len > 0 { Some(0) } else { None },
            len,
            offset: 0,
        }
    }

    pub fn select_next(&mut self) {
        if self.len == 0 {
            return;
        }
        self.selected = Some(match self.selected {
            Some(i) if i + 1 < self.len => i + 1,
            Some(_) => 0, // Wrap to beginning
            None => 0,
        });
    }

    pub fn select_prev(&mut self) {
        if self.len == 0 {
            return;
        }
        self.selected = Some(match self.selected {
            Some(0) => self.len.saturating_sub(1), // Wrap to end
            Some(i) => i - 1,
            None => 0,
        });
    }

    pub fn select_first(&mut self) {
        if self.len > 0 {
            self.selected = Some(0);
        }
    }

    pub fn select_last(&mut self) {
        if self.len > 0 {
            self.selected = Some(self.len.saturating_sub(1));
        }
    }

    pub fn update_len(&mut self, len: usize) {
        self.len = len;
        if let Some(i) = self.selected {
            if i >= len {
                self.selected = if len > 0 { Some(len - 1) } else { None };
            }
        } else if len > 0 {
            self.selected = Some(0);
        }
    }
}

// ============================================================================
// Command/Message System (TODO)
// ============================================================================

// TODO: Implement async command dispatch for daemon communication
// This will allow the TUI to send commands to the daemon and receive responses
// without blocking the UI thread.
//
// pub enum Command {
//     FetchStreams,
//     FetchItems(StreamId),
//     ExecuteAction(ItemId, ActionId),
//     Search(String),
// }
//
// pub enum Message {
//     StreamsLoaded(Vec<Stream>),
//     ItemsLoaded(StreamId, Vec<Item>),
//     ActionCompleted(ActionResult),
//     SearchResults(Vec<Item>),
//     Error(String),
// }

// ============================================================================
// Re-exports
// ============================================================================

pub mod prelude {
    pub use crate::{poll_event, AppEvent, AppState, FocusedPane, ListState, TerminalWrapper};
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ListState Tests
    #[test]
    fn test_list_state_new() {
        let state = ListState::new(5);
        assert_eq!(state.selected, Some(0));
        assert_eq!(state.len, 5);
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn test_list_state_new_empty() {
        let state = ListState::new(0);
        assert_eq!(state.selected, None);
        assert_eq!(state.len, 0);
    }

    #[test]
    fn test_list_state_select_next() {
        let mut state = ListState::new(3);
        assert_eq!(state.selected, Some(0));

        state.select_next();
        assert_eq!(state.selected, Some(1));

        state.select_next();
        assert_eq!(state.selected, Some(2));
    }

    #[test]
    fn test_list_state_select_next_wraps() {
        let mut state = ListState::new(3);
        state.selected = Some(2);
        state.select_next();
        assert_eq!(state.selected, Some(0)); // Should wrap to beginning
    }

    #[test]
    fn test_list_state_select_next_empty_list() {
        let mut state = ListState::new(0);
        state.select_next();
        assert_eq!(state.selected, None); // Should remain None
    }

    #[test]
    fn test_list_state_select_prev() {
        let mut state = ListState::new(3);
        state.selected = Some(2);

        state.select_prev();
        assert_eq!(state.selected, Some(1));

        state.select_prev();
        assert_eq!(state.selected, Some(0));
    }

    #[test]
    fn test_list_state_select_prev_wraps() {
        let mut state = ListState::new(3);
        state.selected = Some(0);
        state.select_prev();
        assert_eq!(state.selected, Some(2)); // Should wrap to end
    }

    #[test]
    fn test_list_state_select_prev_empty_list() {
        let mut state = ListState::new(0);
        state.select_prev();
        assert_eq!(state.selected, None); // Should remain None
    }

    #[test]
    fn test_list_state_select_first() {
        let mut state = ListState::new(5);
        state.selected = Some(3);

        state.select_first();
        assert_eq!(state.selected, Some(0));
    }

    #[test]
    fn test_list_state_select_first_empty_list() {
        let mut state = ListState::new(0);
        state.select_first();
        assert_eq!(state.selected, None);
    }

    #[test]
    fn test_list_state_select_last() {
        let mut state = ListState::new(5);
        state.selected = Some(0);

        state.select_last();
        assert_eq!(state.selected, Some(4));
    }

    #[test]
    fn test_list_state_select_last_empty_list() {
        let mut state = ListState::new(0);
        state.select_last();
        assert_eq!(state.selected, None);
    }

    #[test]
    fn test_list_state_update_len_shrink() {
        let mut state = ListState::new(5);
        state.selected = Some(4);

        state.update_len(3);
        assert_eq!(state.selected, Some(2)); // Should clamp to last valid index
        assert_eq!(state.len, 3);
    }

    #[test]
    fn test_list_state_update_len_grow() {
        let mut state = ListState::new(3);
        state.selected = Some(1);

        state.update_len(5);
        assert_eq!(state.selected, Some(1)); // Should keep current selection
        assert_eq!(state.len, 5);
    }

    #[test]
    fn test_list_state_update_len_to_empty() {
        let mut state = ListState::new(5);
        state.selected = Some(2);

        state.update_len(0);
        assert_eq!(state.selected, None); // Should clear selection
        assert_eq!(state.len, 0);
    }

    #[test]
    fn test_list_state_update_len_from_empty() {
        let mut state = ListState::new(0);
        assert_eq!(state.selected, None);

        state.update_len(3);
        assert_eq!(state.selected, Some(0)); // Should select first item
        assert_eq!(state.len, 3);
    }

    #[test]
    fn test_list_state_default() {
        let state = ListState::default();
        assert_eq!(state.selected, None);
        assert_eq!(state.len, 0);
        assert_eq!(state.offset, 0);
    }

    // FocusedPane Tests
    #[test]
    fn test_focused_pane_default() {
        let pane = FocusedPane::default();
        assert_eq!(pane, FocusedPane::StreamList);
    }

    #[test]
    fn test_focused_pane_next() {
        assert_eq!(FocusedPane::StreamList.next(), FocusedPane::ItemList);
        assert_eq!(FocusedPane::ItemList.next(), FocusedPane::Preview);
        assert_eq!(FocusedPane::Preview.next(), FocusedPane::StreamList);
        assert_eq!(FocusedPane::Omnibar.next(), FocusedPane::StreamList);
    }

    #[test]
    fn test_focused_pane_prev() {
        assert_eq!(FocusedPane::StreamList.prev(), FocusedPane::Preview);
        assert_eq!(FocusedPane::ItemList.prev(), FocusedPane::StreamList);
        assert_eq!(FocusedPane::Preview.prev(), FocusedPane::ItemList);
        assert_eq!(FocusedPane::Omnibar.prev(), FocusedPane::StreamList);
    }

    #[test]
    fn test_focused_pane_navigation_cycle() {
        let mut pane = FocusedPane::StreamList;

        // Next cycle
        pane = pane.next();
        assert_eq!(pane, FocusedPane::ItemList);
        pane = pane.next();
        assert_eq!(pane, FocusedPane::Preview);
        pane = pane.next();
        assert_eq!(pane, FocusedPane::StreamList); // Full cycle

        // Prev cycle
        pane = pane.prev();
        assert_eq!(pane, FocusedPane::Preview);
        pane = pane.prev();
        assert_eq!(pane, FocusedPane::ItemList);
        pane = pane.prev();
        assert_eq!(pane, FocusedPane::StreamList); // Full cycle back
    }

    // AppEvent Tests
    #[test]
    fn test_app_event_equality() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        let event1 = AppEvent::Key(key_event);
        let event2 = AppEvent::Key(key_event);
        assert_eq!(event1, event2);

        let resize1 = AppEvent::Resize(80, 24);
        let resize2 = AppEvent::Resize(80, 24);
        assert_eq!(resize1, resize2);

        assert_eq!(AppEvent::Tick, AppEvent::Tick);
        assert_eq!(AppEvent::Quit, AppEvent::Quit);
    }

    #[test]
    fn test_app_event_clone() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let key_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        let event = AppEvent::Key(key_event);
        let cloned = event.clone();
        assert_eq!(event, cloned);
    }
}
