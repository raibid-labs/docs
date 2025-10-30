//! Event handling for the TUI
//!
//! This module manages keyboard and terminal events using crossterm.

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

/// Application events
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Key press event
    Key(KeyEvent),
    /// Terminal resize event
    Resize(u16, u16),
    /// Tick event for periodic updates
    Tick,
}

/// Event handler for the TUI
pub struct EventHandler {
    /// Tick rate for periodic updates
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler with the specified tick rate
    pub fn new(tick_rate: Duration) -> Self {
        Self { tick_rate }
    }

    /// Poll for the next event
    ///
    /// This method blocks until an event is available or the tick rate expires.
    pub fn next(&self) -> Result<Event> {
        if event::poll(self.tick_rate)? {
            match event::read()? {
                CrosstermEvent::Key(key) => Ok(Event::Key(key)),
                CrosstermEvent::Resize(width, height) => Ok(Event::Resize(width, height)),
                _ => Ok(Event::Tick),
            }
        } else {
            Ok(Event::Tick)
        }
    }
}

/// Check if the key event is a quit command (q or Ctrl+C)
pub fn is_quit_event(key: &KeyEvent) -> bool {
    matches!(
        key,
        KeyEvent {
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        } | KeyEvent {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::CONTROL,
            ..
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_handler_creation() {
        let handler = EventHandler::new(Duration::from_millis(100));
        assert_eq!(handler.tick_rate, Duration::from_millis(100));
    }

    #[test]
    fn test_is_quit_event() {
        let quit_q = KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(is_quit_event(&quit_q));

        let quit_ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(is_quit_event(&quit_ctrl_c));

        let not_quit = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!is_quit_event(&not_quit));
    }
}
