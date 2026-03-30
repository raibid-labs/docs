// Terminal-specific modules
// Handles scrollback buffer, history management, and terminal state tracking

pub mod scrollback;

pub use scrollback::{ScrollbackBuffer, ScrollbackLine, ScrollbackPlugin, ScrollbackState};
