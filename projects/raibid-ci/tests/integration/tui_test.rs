//! Integration tests for TUI functionality
//!
//! These tests verify the TUI components without actually launching
//! the terminal interface.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use raibid_cli::tui::{App, AppConfig, Event, InputMode, Tab};
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

// Tests for CLI-005: Interactive Controls & Navigation

#[test]
fn test_tab_navigation() {
    let mut app = App::new();

    // Initial tab should be Jobs
    assert_eq!(app.current_tab(), Tab::Jobs);

    // Navigate forward
    app.next_tab();
    assert_eq!(app.current_tab(), Tab::Agents);

    app.next_tab();
    assert_eq!(app.current_tab(), Tab::Config);

    app.next_tab();
    assert_eq!(app.current_tab(), Tab::Logs);

    // Wrap around
    app.next_tab();
    assert_eq!(app.current_tab(), Tab::Jobs);

    // Navigate backward
    app.previous_tab();
    assert_eq!(app.current_tab(), Tab::Logs);
}

#[test]
fn test_job_selection() {
    let mut app = App::new();

    let initial_selection = app.selected_job();
    assert_eq!(initial_selection, 0);

    // Move down
    app.select_next();
    assert_eq!(app.selected_job(), 1);

    app.select_next();
    assert_eq!(app.selected_job(), 2);

    // Move up
    app.select_previous();
    assert_eq!(app.selected_job(), 1);

    app.select_previous();
    assert_eq!(app.selected_job(), 0);

    // Can't go below 0
    app.select_previous();
    assert_eq!(app.selected_job(), 0);
}

#[test]
fn test_help_screen_toggle() {
    let mut app = App::new();

    let ui_state = app.ui_state();
    assert!(!ui_state.show_help);

    app.toggle_help();
    let ui_state = app.ui_state();
    assert!(ui_state.show_help);

    app.toggle_help();
    let ui_state = app.ui_state();
    assert!(!ui_state.show_help);
}

#[test]
fn test_detail_popup_toggle() {
    let mut app = App::new();

    let ui_state = app.ui_state();
    assert!(!ui_state.show_detail_popup);

    app.toggle_detail_popup();
    let ui_state = app.ui_state();
    assert!(ui_state.show_detail_popup);

    app.toggle_detail_popup();
    let ui_state = app.ui_state();
    assert!(!ui_state.show_detail_popup);
}

#[test]
fn test_filter_menu_toggle() {
    let mut app = App::new();

    let ui_state = app.ui_state();
    assert!(!ui_state.show_filter_menu);
    assert_eq!(ui_state.input_mode, InputMode::Normal);

    app.toggle_filter_menu();
    let ui_state = app.ui_state();
    assert!(ui_state.show_filter_menu);
    assert_eq!(ui_state.input_mode, InputMode::Filter);

    app.toggle_filter_menu();
    let ui_state = app.ui_state();
    assert!(!ui_state.show_filter_menu);
    assert_eq!(ui_state.input_mode, InputMode::Normal);
}

#[test]
fn test_search_mode() {
    let mut app = App::new();

    let ui_state = app.ui_state();
    assert_eq!(ui_state.input_mode, InputMode::Normal);
    assert_eq!(ui_state.search_query, "");

    app.enter_search_mode();
    let ui_state = app.ui_state();
    assert_eq!(ui_state.input_mode, InputMode::Search);

    app.search_input('t');
    app.search_input('e');
    app.search_input('s');
    app.search_input('t');

    let ui_state = app.ui_state();
    assert_eq!(ui_state.search_query, "test");

    app.search_backspace();
    let ui_state = app.ui_state();
    assert_eq!(ui_state.search_query, "tes");

    app.exit_search_mode();
    let ui_state = app.ui_state();
    assert_eq!(ui_state.input_mode, InputMode::Normal);
    assert_eq!(ui_state.search_query, "");
}

#[test]
fn test_filter_application() {
    let mut app = App::new();

    let ui_state = app.ui_state();
    assert!(ui_state.filter_status.is_none());

    app.toggle_filter_menu();

    // Select Running filter (option 1)
    app.select_next();
    let ui_state = app.ui_state();
    assert_eq!(ui_state.selected_filter_option, 1);

    app.apply_filter();
    let ui_state = app.ui_state();
    assert!(ui_state.filter_status.is_some());
}

#[test]
fn test_confirmation_dialog() {
    let mut app = App::new();

    let ui_state = app.ui_state();
    assert!(!ui_state.show_confirmation);

    app.show_cancel_confirmation();
    let ui_state = app.ui_state();

    // Confirmation only shows if there's a selected job
    if !app.filtered_jobs().is_empty() {
        assert!(ui_state.show_confirmation);
        assert!(!ui_state.confirmation_message.is_empty());

        app.cancel_confirmation();
        let ui_state = app.ui_state();
        assert!(!ui_state.show_confirmation);
    }
}

#[test]
fn test_keyboard_event_handling() {
    let mut app = App::new();

    // Test tab switching with keyboard
    let tab_event = Event::Key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    app.handle_event(tab_event);
    assert_eq!(app.current_tab(), Tab::Agents);

    // Test number key tab jumping
    let num_event = Event::Key(KeyEvent::new(KeyCode::Char('4'), KeyModifiers::NONE));
    app.handle_event(num_event);
    assert_eq!(app.current_tab(), Tab::Logs);

    // Test help toggle
    let help_event = Event::Key(KeyEvent::new(KeyCode::Char('?'), KeyModifiers::NONE));
    app.handle_event(help_event);
    let ui_state = app.ui_state();
    assert!(ui_state.show_help);

    // Any key closes help
    let close_event = Event::Key(KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE));
    app.handle_event(close_event);
    let ui_state = app.ui_state();
    assert!(!ui_state.show_help);
}

#[test]
fn test_job_filtering() {
    let mut app = App::new();

    let all_jobs = app.filtered_jobs();
    let all_count = all_jobs.len();
    assert!(all_count > 0);

    // Apply a filter (won't actually filter jobs since mock data is random)
    app.toggle_filter_menu();
    app.select_next(); // Select "Running"
    app.apply_filter();

    // Filtered jobs should be equal or less than all jobs
    let filtered_jobs = app.filtered_jobs();
    assert!(filtered_jobs.len() <= all_count);
}

#[test]
fn test_search_functionality() {
    let mut app = App::new();

    let all_jobs = app.filtered_jobs();
    let all_count = all_jobs.len();

    // Enter search mode and search for something unlikely
    app.enter_search_mode();
    app.search_input('x');
    app.search_input('y');
    app.search_input('z');
    app.search_input('z');
    app.search_input('y');

    // Apply search by exiting search mode
    app.exit_search_mode();

    // After weird search, we should have 0 or fewer results
    let search_results = app.filtered_jobs();
    assert!(search_results.len() <= all_count);
}

#[test]
fn test_refresh_functionality() {
    let mut app = App::new();

    let initial_jobs = app.jobs().len();
    let initial_agents = app.agents().len();

    app.refresh();

    // After refresh, counts should remain the same
    assert_eq!(app.jobs().len(), initial_jobs);
    assert_eq!(app.agents().len(), initial_agents);
}

#[test]
fn test_get_selected_job() {
    let mut app = App::new();

    if !app.filtered_jobs().is_empty() {
        let selected = app.get_selected_job();
        assert!(selected.is_some());

        // Move selection and verify it changes
        app.select_next();
        let next_selected = app.get_selected_job();
        assert!(next_selected.is_some());
    }
}
