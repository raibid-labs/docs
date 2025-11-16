//! Application state and event loop
//!
//! This module contains the main application state and event handling logic.

use anyhow::Result;
use std::time::Duration;

use super::events::{is_quit_event, Event, EventHandler};
use super::mock_data::{
    generate_mock_data, JobStatus, MockAgent, MockDataConfig, MockJob, MockQueueData,
};
use super::terminal::Terminal;
use super::ui;

/// Available tabs in the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Jobs,
    Agents,
    Config,
    Logs,
}

impl Tab {
    /// Get tab name as string
    pub fn as_str(&self) -> &str {
        match self {
            Tab::Jobs => "Jobs",
            Tab::Agents => "Agents",
            Tab::Config => "Config",
            Tab::Logs => "Logs",
        }
    }

    /// Get all tabs
    pub fn all() -> Vec<Tab> {
        vec![Tab::Jobs, Tab::Agents, Tab::Config, Tab::Logs]
    }

    /// Get next tab
    pub fn next(&self) -> Tab {
        match self {
            Tab::Jobs => Tab::Agents,
            Tab::Agents => Tab::Config,
            Tab::Config => Tab::Logs,
            Tab::Logs => Tab::Jobs,
        }
    }

    /// Get previous tab
    pub fn previous(&self) -> Tab {
        match self {
            Tab::Jobs => Tab::Logs,
            Tab::Agents => Tab::Jobs,
            Tab::Config => Tab::Agents,
            Tab::Logs => Tab::Config,
        }
    }
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Refresh interval for updating data
    pub refresh_interval: Duration,
    /// Panel proportions (jobs, agents, queue) - percentages that sum to 100
    #[allow(dead_code)]
    pub panel_proportions: (u16, u16, u16),
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Duration::from_secs(1),
            panel_proportions: (60, 20, 20),
        }
    }
}

/// Input mode for different interaction states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Normal navigation mode
    Normal,
    /// Search input mode
    Search,
    /// Filter selection mode
    Filter,
}

/// Main application state
pub struct App {
    /// Application configuration
    config: AppConfig,
    /// Mock data configuration
    mock_config: MockDataConfig,
    /// Current job list
    jobs: Vec<MockJob>,
    /// Current agent list
    agents: Vec<MockAgent>,
    /// Queue depth data
    queue_data: MockQueueData,
    /// Whether the application should quit
    should_quit: bool,
    /// Current active tab
    current_tab: Tab,
    /// Selected job index (for scrolling/selection)
    selected_job: usize,
    /// Selected agent index (for scrolling/selection)
    selected_agent: usize,
    /// Show job detail popup
    show_detail_popup: bool,
    /// Show help screen
    show_help: bool,
    /// Show filter menu
    show_filter_menu: bool,
    /// Show confirmation dialog
    show_confirmation: bool,
    /// Confirmation message
    confirmation_message: String,
    /// Current input mode
    input_mode: InputMode,
    /// Search query string
    search_query: String,
    /// Filter by job status (None = show all)
    filter_status: Option<JobStatus>,
    /// Selected filter option index
    selected_filter_option: usize,
    /// Log scroll offset
    log_scroll_offset: usize,
}

impl App {
    /// Create a new application with default configuration
    pub fn new() -> Self {
        Self::with_config(AppConfig::default())
    }

    /// Create a new application with custom configuration
    pub fn with_config(config: AppConfig) -> Self {
        let mock_config = MockDataConfig::default();
        let (jobs, agents, queue_data) = generate_mock_data(&mock_config);

        Self {
            config,
            mock_config,
            jobs,
            agents,
            queue_data,
            should_quit: false,
            current_tab: Tab::Jobs,
            selected_job: 0,
            selected_agent: 0,
            show_detail_popup: false,
            show_help: false,
            show_filter_menu: false,
            show_confirmation: false,
            confirmation_message: String::new(),
            input_mode: InputMode::Normal,
            search_query: String::new(),
            filter_status: None,
            selected_filter_option: 0,
            log_scroll_offset: 0,
        }
    }

    /// Get current tab
    #[allow(dead_code)]
    pub fn current_tab(&self) -> Tab {
        self.current_tab
    }

    /// Switch to next tab
    pub fn next_tab(&mut self) {
        self.current_tab = self.current_tab.next();
    }

    /// Switch to previous tab
    pub fn previous_tab(&mut self) {
        self.current_tab = self.current_tab.previous();
    }

    /// Get selected job index
    #[allow(dead_code)]
    pub fn selected_job(&self) -> usize {
        self.selected_job
    }

    /// Get selected agent index
    #[allow(dead_code)]
    pub fn selected_agent(&self) -> usize {
        self.selected_agent
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        if self.show_filter_menu {
            // Navigate filter options
            if self.selected_filter_option > 0 {
                self.selected_filter_option -= 1;
            }
        } else {
            match self.current_tab {
                Tab::Jobs => {
                    if self.selected_job > 0 {
                        self.selected_job -= 1;
                    }
                }
                Tab::Agents => {
                    if self.selected_agent > 0 {
                        self.selected_agent -= 1;
                    }
                }
                Tab::Logs => {
                    // Scroll logs up
                    self.log_scroll_offset = self.log_scroll_offset.saturating_sub(1);
                }
                _ => {}
            }
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        if self.show_filter_menu {
            // Navigate filter options
            if self.selected_filter_option < 4 {
                self.selected_filter_option += 1;
            }
        } else {
            match self.current_tab {
                Tab::Jobs => {
                    if self.selected_job < self.filtered_jobs().len().saturating_sub(1) {
                        self.selected_job += 1;
                    }
                }
                Tab::Agents => {
                    if self.selected_agent < self.agents.len().saturating_sub(1) {
                        self.selected_agent += 1;
                    }
                }
                Tab::Logs => {
                    // Scroll logs down
                    self.log_scroll_offset = self.log_scroll_offset.saturating_add(1).min(100);
                }
                _ => {}
            }
        }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Request the application to quit
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Update application state (refresh mock data)
    pub fn update(&mut self) {
        // Regenerate mock data to simulate changes
        let (jobs, agents, _) = generate_mock_data(&self.mock_config);
        self.jobs = jobs;
        self.agents = agents;

        // Update queue data incrementally
        let mut rng = rand::thread_rng();
        self.queue_data.update(&mut rng);
    }

    /// Handle an event
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => {
                use crossterm::event::KeyCode;

                // Handle input mode-specific keys
                if self.input_mode == InputMode::Search {
                    match key.code {
                        KeyCode::Esc => self.exit_search_mode(),
                        KeyCode::Enter => self.exit_search_mode(),
                        KeyCode::Char(c) => self.search_input(c),
                        KeyCode::Backspace => self.search_backspace(),
                        _ => {}
                    }
                } else if self.input_mode == InputMode::Filter {
                    match key.code {
                        KeyCode::Esc => self.toggle_filter_menu(),
                        KeyCode::Enter => self.apply_filter(),
                        KeyCode::Up => self.select_previous(),
                        KeyCode::Down => self.select_next(),
                        _ => {}
                    }
                } else {
                    // InputMode::Normal
                    {
                        // Handle confirmation dialog
                        if self.show_confirmation {
                            match key.code {
                                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                                    self.confirm_action()
                                }
                                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                    self.cancel_confirmation()
                                }
                                _ => {}
                            }
                        } else if self.show_help {
                            // Handle help screen - any key closes it
                            self.toggle_help();
                        } else if self.show_detail_popup {
                            // Handle detail popup
                            match key.code {
                                KeyCode::Esc => self.toggle_detail_popup(),
                                KeyCode::Char('c') => {
                                    self.toggle_detail_popup();
                                    self.show_cancel_confirmation();
                                }
                                KeyCode::Char('r') => self.refresh(),
                                _ => {}
                            }
                        } else if is_quit_event(&key) {
                            // Normal mode key handling
                            self.quit();
                        } else {
                            match key.code {
                                // Navigation
                                KeyCode::Tab => self.next_tab(),
                                KeyCode::BackTab => self.previous_tab(),
                                KeyCode::Right => self.next_tab(),
                                KeyCode::Left => self.previous_tab(),
                                KeyCode::Up => self.select_previous(),
                                KeyCode::Down => self.select_next(),
                                // Tab jumping
                                KeyCode::Char('1') => self.current_tab = Tab::Jobs,
                                KeyCode::Char('2') => self.current_tab = Tab::Agents,
                                KeyCode::Char('3') => self.current_tab = Tab::Config,
                                KeyCode::Char('4') => self.current_tab = Tab::Logs,
                                // Actions
                                KeyCode::Enter => self.toggle_detail_popup(),
                                KeyCode::Char('?') => self.toggle_help(),
                                KeyCode::Char('f') => self.toggle_filter_menu(),
                                KeyCode::Char('/') => self.enter_search_mode(),
                                KeyCode::Char('c') => self.show_cancel_confirmation(),
                                KeyCode::Char('r') => self.refresh(),
                                KeyCode::Esc => {
                                    // Clear filters and search
                                    if self.filter_status.is_some() || !self.search_query.is_empty()
                                    {
                                        self.filter_status = None;
                                        self.search_query.clear();
                                        self.selected_job = 0;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            Event::Resize(_, _) => {
                // Terminal resize is handled automatically by ratatui
            }
            Event::Tick => {
                // Auto-refresh on tick
                self.update();
            }
        }
    }

    /// Run the main event loop
    pub fn run(&mut self, terminal: &mut Terminal) -> Result<()> {
        let event_handler = EventHandler::new(self.config.refresh_interval);

        while !self.should_quit() {
            // Render the UI
            let filtered_jobs: Vec<MockJob> =
                self.filtered_jobs().iter().map(|&j| j.clone()).collect();
            let ui_state = self.ui_state();

            terminal.draw(|frame| {
                ui::render(
                    frame,
                    &filtered_jobs,
                    &self.agents,
                    &self.queue_data,
                    self.current_tab,
                    self.selected_job,
                    self.selected_agent,
                    &ui_state,
                );
            })?;

            // Handle events
            let event = event_handler.next()?;
            self.handle_event(event);
        }

        Ok(())
    }

    /// Get jobs reference (for testing and integration tests)
    #[allow(dead_code)]
    pub fn jobs(&self) -> &[MockJob] {
        &self.jobs
    }

    /// Get agents reference (for testing and integration tests)
    #[allow(dead_code)]
    pub fn agents(&self) -> &[MockAgent] {
        &self.agents
    }

    /// Get queue data reference (for testing and integration tests)
    #[allow(dead_code)]
    pub fn queue_data(&self) -> &MockQueueData {
        &self.queue_data
    }

    /// Toggle job detail popup
    pub fn toggle_detail_popup(&mut self) {
        if self.current_tab == Tab::Jobs && !self.filtered_jobs().is_empty() {
            self.show_detail_popup = !self.show_detail_popup;
        }
    }

    /// Toggle help screen
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    /// Toggle filter menu
    pub fn toggle_filter_menu(&mut self) {
        if self.current_tab == Tab::Jobs {
            self.show_filter_menu = !self.show_filter_menu;
            if !self.show_filter_menu {
                self.input_mode = InputMode::Normal;
            } else {
                self.input_mode = InputMode::Filter;
            }
        }
    }

    /// Apply selected filter
    pub fn apply_filter(&mut self) {
        self.filter_status = match self.selected_filter_option {
            0 => None, // All
            1 => Some(JobStatus::Running),
            2 => Some(JobStatus::Success),
            3 => Some(JobStatus::Failed),
            4 => Some(JobStatus::Pending),
            _ => None,
        };
        self.show_filter_menu = false;
        self.input_mode = InputMode::Normal;
        self.selected_job = 0; // Reset selection
    }

    /// Enter search mode
    pub fn enter_search_mode(&mut self) {
        if self.current_tab == Tab::Jobs {
            self.input_mode = InputMode::Search;
            self.search_query.clear();
        }
    }

    /// Exit search mode
    pub fn exit_search_mode(&mut self) {
        self.input_mode = InputMode::Normal;
        self.search_query.clear();
    }

    /// Add character to search query
    pub fn search_input(&mut self, c: char) {
        if self.input_mode == InputMode::Search {
            self.search_query.push(c);
        }
    }

    /// Remove last character from search query
    pub fn search_backspace(&mut self) {
        if self.input_mode == InputMode::Search {
            self.search_query.pop();
        }
    }

    /// Get filtered jobs based on status filter and search query
    pub fn filtered_jobs(&self) -> Vec<&MockJob> {
        self.jobs
            .iter()
            .filter(|job| {
                // Apply status filter
                if let Some(status) = self.filter_status {
                    if job.status != status {
                        return false;
                    }
                }

                // Apply search query
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    let matches_repo = job.repo.to_lowercase().contains(&query);
                    let matches_branch = job.branch.to_lowercase().contains(&query);
                    let matches_id = job.id.to_lowercase().contains(&query);
                    if !matches_repo && !matches_branch && !matches_id {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Get the currently selected job
    pub fn get_selected_job(&self) -> Option<&MockJob> {
        let filtered = self.filtered_jobs();
        filtered.get(self.selected_job).copied()
    }

    /// Show confirmation dialog for job cancellation
    pub fn show_cancel_confirmation(&mut self) {
        if let Some(job) = self.get_selected_job() {
            self.confirmation_message = format!("Cancel job {}?", job.id);
            self.show_confirmation = true;
        }
    }

    /// Cancel confirmation
    pub fn cancel_confirmation(&mut self) {
        self.show_confirmation = false;
        self.confirmation_message.clear();
    }

    /// Confirm action (e.g., cancel job)
    pub fn confirm_action(&mut self) {
        if self.show_confirmation {
            // In a real implementation, this would send a cancel request
            // For now, just close the dialog
            self.show_confirmation = false;
            self.confirmation_message.clear();
        }
    }

    /// Refresh data manually
    pub fn refresh(&mut self) {
        self.update();
    }

    /// Get UI state for rendering
    pub fn ui_state(&self) -> UiState<'_> {
        UiState {
            show_detail_popup: self.show_detail_popup,
            show_help: self.show_help,
            show_filter_menu: self.show_filter_menu,
            show_confirmation: self.show_confirmation,
            confirmation_message: &self.confirmation_message,
            input_mode: self.input_mode,
            search_query: &self.search_query,
            filter_status: self.filter_status,
            selected_filter_option: self.selected_filter_option,
            log_scroll_offset: self.log_scroll_offset,
        }
    }
}

/// UI state for rendering (to avoid passing too many parameters)
pub struct UiState<'a> {
    pub show_detail_popup: bool,
    pub show_help: bool,
    pub show_filter_menu: bool,
    pub show_confirmation: bool,
    pub confirmation_message: &'a str,
    pub input_mode: InputMode,
    pub search_query: &'a str,
    #[allow(dead_code)]
    pub filter_status: Option<JobStatus>,
    pub selected_filter_option: usize,
    #[allow(dead_code)]
    pub log_scroll_offset: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert!(!app.should_quit());
        assert!(!app.jobs().is_empty());
        assert!(!app.agents().is_empty());
        assert_eq!(app.queue_data().history.len(), 60);
    }

    #[test]
    fn test_app_quit() {
        let mut app = App::new();
        assert!(!app.should_quit());

        app.quit();
        assert!(app.should_quit());
    }

    #[test]
    fn test_app_update() {
        let mut app = App::new();
        let initial_jobs = app.jobs().len();
        let initial_agents = app.agents().len();

        app.update();

        // Should still have the same count of jobs and agents
        assert_eq!(app.jobs().len(), initial_jobs);
        assert_eq!(app.agents().len(), initial_agents);
    }

    #[test]
    fn test_app_config_default() {
        let config = AppConfig::default();
        assert_eq!(config.refresh_interval, Duration::from_secs(1));
        assert_eq!(config.panel_proportions, (60, 20, 20));
    }

    #[test]
    fn test_app_with_custom_config() {
        let config = AppConfig {
            refresh_interval: Duration::from_millis(500),
            panel_proportions: (70, 15, 15),
        };

        let app = App::with_config(config.clone());
        assert_eq!(app.config.refresh_interval, config.refresh_interval);
        assert_eq!(app.config.panel_proportions, config.panel_proportions);
    }

    #[test]
    fn test_handle_quit_event() {
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut app = App::new();

        // Test 'q' key
        let quit_event = Event::Key(KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        app.handle_event(quit_event);
        assert!(app.should_quit());
    }

    #[test]
    fn test_handle_tick_event() {
        let mut app = App::new();
        let initial_queue_len = app.queue_data().history.len();

        app.handle_event(Event::Tick);

        // Should maintain queue history length
        assert_eq!(app.queue_data().history.len(), initial_queue_len);
    }
}
