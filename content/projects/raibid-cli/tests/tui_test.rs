//! Integration tests for TUI functionality
//!
//! These tests verify the TUI components without actually launching
//! the terminal interface.

use raibid_cli::tui::{App, AppConfig, MockDataConfig};
use std::time::Duration;

#[test]
fn test_tui_app_initialization() {
    let app = App::new();
    assert!(!app.should_quit());
}

#[test]
fn test_tui_app_with_custom_config() {
    let config = AppConfig {
        refresh_interval: Duration::from_millis(500),
        panel_proportions: (70, 15, 15),
    };

    let app = App::with_config(config.clone());
    assert!(!app.should_quit());
}

#[test]
fn test_mock_data_generation() {
    let app = App::new();

    // Verify jobs were generated
    assert!(!app.jobs().is_empty(), "Jobs should be generated");

    // Verify agents were generated
    assert!(!app.agents().is_empty(), "Agents should be generated");

    // Verify queue data was generated
    assert_eq!(
        app.queue_data().history.len(),
        60,
        "Queue history should have 60 data points"
    );
}

#[test]
fn test_app_state_updates() {
    let mut app = App::new();

    let initial_job_count = app.jobs().len();
    let initial_agent_count = app.agents().len();

    // Update the app state
    app.update();

    // Jobs and agents count should remain the same
    assert_eq!(app.jobs().len(), initial_job_count);
    assert_eq!(app.agents().len(), initial_agent_count);

    // Queue history should still have 60 data points
    assert_eq!(app.queue_data().history.len(), 60);
}

#[test]
fn test_app_quit_functionality() {
    let mut app = App::new();

    assert!(!app.should_quit());

    app.quit();

    assert!(app.should_quit());
}
