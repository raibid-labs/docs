mod commands;
mod manager;
mod store;

pub use commands::handle_session_command;
pub use manager::{Session, SessionManager};
pub use store::SessionStore;

/// Client identifier
pub type ClientId = u64;

/// Session identifier (UUID)
pub type SessionId = String;

/// Grid state for session (placeholder for now, should match VTE state)
#[derive(Clone, Debug)]
pub struct GridState {
    pub cols: u16,
    pub rows: u16,
    // Additional state will be integrated with VTE
}

impl GridState {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self { cols, rows }
    }
}
