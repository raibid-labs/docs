//! Terminal User Interface (TUI) module
//!
//! This module provides a rich terminal-based dashboard for monitoring
//! and managing CI/CD jobs, agents, and queue metrics.

mod app;
mod events;
mod mock_data;
mod terminal;
mod ui;

pub use app::{App, AppConfig};
pub use mock_data::{
    generate_mock_data, AgentStatus, JobStatus, MockAgent, MockDataConfig, MockJob, MockQueueData,
};
pub use terminal::{Terminal, MIN_HEIGHT, MIN_WIDTH};

use anyhow::Result;

/// Launch the TUI application
///
/// This is the main entry point for the TUI. It handles:
/// - Terminal initialization
/// - Application creation and event loop
/// - Terminal cleanup (even on errors)
pub fn launch() -> Result<()> {
    // Initialize terminal
    let mut terminal = terminal::init()?;

    // Create and run the application
    let result = {
        let mut app = App::new();
        app.run(&mut terminal)
    };

    // Restore terminal state
    terminal::restore()?;

    // Return the result of running the app
    result
}

/// Launch the TUI application with custom configuration
pub fn launch_with_config(config: AppConfig) -> Result<()> {
    // Initialize terminal
    let mut terminal = terminal::init()?;

    // Create and run the application with config
    let result = {
        let mut app = App::with_config(config);
        app.run(&mut terminal)
    };

    // Restore terminal state
    terminal::restore()?;

    // Return the result of running the app
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert!(!app.should_quit());
    }

    #[test]
    fn test_app_with_config() {
        let config = AppConfig::default();
        let app = App::with_config(config);
        assert!(!app.should_quit());
    }
}
