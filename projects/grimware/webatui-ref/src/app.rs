//! Main application structure
//!
//! The `TerminalApp` is the central coordinator for the webatui application,
//! handling state management, rendering, and event processing.

use crate::prelude::*;
use crate::state::{AppState, Message, Screen};
use crate::components::{Header, Footer, Menu};
use std::io;

/// Main terminal application structure
///
/// This struct manages the application lifecycle, including:
/// - State management
/// - Event handling
/// - Rendering coordination
/// - Screen navigation
pub struct TerminalApp {
    /// Current application state
    state: AppState,

    /// Whether the application should exit
    should_quit: bool,

    /// Last frame render time for FPS calculation
    #[cfg(not(target_arch = "wasm32"))]
    last_frame_time: Option<std::time::Instant>,
}

impl TerminalApp {
    /// Create a new terminal application with default state
    pub fn new() -> Self {
        Self {
            state: AppState::default(),
            should_quit: false,
            #[cfg(not(target_arch = "wasm32"))]
            last_frame_time: None,
        }
    }

    /// Check if the application should quit
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Get a reference to the current state
    pub fn state(&self) -> &AppState {
        &self.state
    }

    /// Get a mutable reference to the current state
    pub fn state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    /// Handle a message and update state
    ///
    /// This is the main state update function that processes all messages
    /// and modifies the application state accordingly.
    pub fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::Quit => {
                self.should_quit = true;
            }
            Message::Navigate(screen) => {
                self.state.navigate_to(screen);
            }
            Message::NavigateBack => {
                self.state.navigate_back();
            }
            Message::Increment => {
                self.state.increment_counter();
            }
            Message::Decrement => {
                self.state.decrement_counter();
            }
            Message::SelectNext => {
                self.state.select_next();
            }
            Message::SelectPrevious => {
                self.state.select_previous();
            }
            Message::AddItem(item) => {
                self.state.add_item(item);
            }
            Message::RemoveItem => {
                self.state.remove_selected_item();
            }
            Message::ToggleHelp => {
                self.state.toggle_help();
            }
            Message::UpdateMetrics { cpu, memory, network } => {
                self.state.update_metrics(cpu, memory, network);
            }
            Message::UpdateInput(input) => {
                self.state.input_buffer = input;
            }
            Message::ClearInput => {
                self.state.input_buffer.clear();
            }
            Message::ScrollUp => {
                self.state.scroll_offset = self.state.scroll_offset.saturating_sub(1);
            }
            Message::ScrollDown => {
                self.state.scroll_offset = self.state.scroll_offset.saturating_add(1);
            }
            Message::UpdateSettings(settings) => {
                self.state.settings = settings;
            }
            Message::Refresh => {
                // Trigger a refresh - could be used to reload data
            }
            Message::None => {
                // No-op message for ignored events
            }
        }
    }

    /// Render the application UI
    ///
    /// This method is responsible for rendering the entire application UI,
    /// including the header, content area, and footer.
    pub fn render(&mut self, frame: &mut Frame) {
        // Update frame time for FPS tracking
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.last_frame_time = Some(std::time::Instant::now());
        }

        let area = frame.area();

        // Create main layout: [Header, Content, Footer]
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),      // Header
                Constraint::Min(0),         // Content
                Constraint::Length(3),      // Footer
            ])
            .split(area);

        // Render header
        let header = Header::new(&self.state.title, self.state.current_screen());
        frame.render_widget(header, chunks[0]);

        // Render content based on current screen
        self.render_content(frame, chunks[1]);

        // Render footer
        let footer = Footer::new(self.state.current_screen());
        frame.render_widget(footer, chunks[2]);

        // Render help overlay if enabled
        if self.state.show_help {
            self.render_help_overlay(frame, area);
        }
    }

    /// Render the content area based on current screen
    fn render_content(&self, frame: &mut Frame, area: Rect) {
        match self.state.current_screen() {
            Screen::Dashboard => self.render_dashboard(frame, area),
            Screen::Settings => self.render_settings(frame, area),
            Screen::DataView => self.render_data_view(frame, area),
            Screen::Help => self.render_help(frame, area),
        }
    }

    /// Render the dashboard screen
    fn render_dashboard(&self, frame: &mut Frame, area: Rect) {
        // Create dashboard layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(5),  // Metrics row
                Constraint::Min(0),     // Content
            ])
            .split(area);

        // Render metrics gauges
        let metrics_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(chunks[0]);

        // CPU Gauge
        let cpu_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("CPU"))
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent((self.state.metrics.current_cpu() * 100.0) as u16);
        frame.render_widget(cpu_gauge, metrics_chunks[0]);

        // Memory Gauge
        let memory_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Memory"))
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent((self.state.metrics.current_memory() * 100.0) as u16);
        frame.render_widget(memory_gauge, metrics_chunks[1]);

        // Network Gauge
        let network_gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Network"))
            .gauge_style(Style::default().fg(Color::Green))
            .percent((self.state.metrics.current_network() * 100.0) as u16);
        frame.render_widget(network_gauge, metrics_chunks[2]);

        // Main content: Navigation menu
        let menu = Menu::new(&self.state.items, self.state.selected_index);
        frame.render_widget(menu, chunks[1]);
    }

    /// Render the settings screen
    fn render_settings(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Settings");

        let settings_text = vec![
            Line::from(vec![
                Span::styled("Theme: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&self.state.settings.theme),
            ]),
            Line::from(vec![
                Span::styled("Refresh Rate: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}ms", self.state.settings.refresh_rate)),
            ]),
            Line::from(vec![
                Span::styled("Max Data Points: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}", self.state.settings.max_data_points)),
            ]),
            Line::from(vec![
                Span::styled("Animations: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(if self.state.settings.enable_animations { "Enabled" } else { "Disabled" }),
            ]),
            Line::from(vec![
                Span::styled("Auto-save: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}s", self.state.settings.auto_save_interval)),
            ]),
        ];

        let paragraph = Paragraph::new(settings_text)
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    /// Render the data view screen
    fn render_data_view(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Data View");

        // Create sparklines for metrics history
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(33),
                Constraint::Percentage(34),
            ])
            .split(area);

        // CPU History Sparkline
        let cpu_data: Vec<u64> = self.state.metrics.cpu_history
            .iter()
            .map(|&v| (v * 100.0) as u64)
            .collect();

        if !cpu_data.is_empty() {
            let cpu_sparkline = Sparkline::default()
                .block(Block::default().borders(Borders::ALL).title("CPU History"))
                .data(&cpu_data)
                .style(Style::default().fg(Color::Cyan));
            frame.render_widget(cpu_sparkline, chunks[0]);
        }

        // Memory History Sparkline
        let memory_data: Vec<u64> = self.state.metrics.memory_history
            .iter()
            .map(|&v| (v * 100.0) as u64)
            .collect();

        if !memory_data.is_empty() {
            let memory_sparkline = Sparkline::default()
                .block(Block::default().borders(Borders::ALL).title("Memory History"))
                .data(&memory_data)
                .style(Style::default().fg(Color::Yellow));
            frame.render_widget(memory_sparkline, chunks[1]);
        }

        // Network History Sparkline
        let network_data: Vec<u64> = self.state.metrics.network_history
            .iter()
            .map(|&v| (v * 100.0) as u64)
            .collect();

        if !network_data.is_empty() {
            let network_sparkline = Sparkline::default()
                .block(Block::default().borders(Borders::ALL).title("Network History"))
                .data(&network_data)
                .style(Style::default().fg(Color::Green));
            frame.render_widget(network_sparkline, chunks[2]);
        }
    }

    /// Render the help screen
    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Help");

        let help_text = vec![
            Line::from(Span::styled(
                "WebATUI Reference Application",
                Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan),
            )),
            Line::from(""),
            Line::from(Span::styled("Navigation:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from("  1-4     - Navigate between screens"),
            Line::from("  Tab     - Next screen"),
            Line::from("  Esc     - Previous screen / Close help"),
            Line::from(""),
            Line::from(Span::styled("Actions:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from("  ↑/↓     - Navigate menu items"),
            Line::from("  +/-     - Increment/Decrement counter"),
            Line::from("  a       - Add item"),
            Line::from("  d       - Delete selected item"),
            Line::from("  r       - Refresh data"),
            Line::from("  ?       - Toggle this help"),
            Line::from("  q       - Quit application"),
            Line::from(""),
            Line::from(Span::styled("Screens:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from("  Dashboard   - Main overview with metrics"),
            Line::from("  Settings    - Application configuration"),
            Line::from("  Data View   - Visualization of metrics history"),
            Line::from("  Help        - This help screen"),
        ];

        let paragraph = Paragraph::new(help_text)
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    /// Render help overlay on top of current screen
    fn render_help_overlay(&self, frame: &mut Frame, area: Rect) {
        // Create centered popup area
        let popup_area = centered_rect(60, 70, area);

        // Clear the area
        frame.render_widget(Clear, popup_area);

        // Render help content
        self.render_help(frame, popup_area);
    }

    /// Run the application (native terminal only)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn run(&mut self) -> io::Result<()> {
        use crossterm::{
            execute,
            terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        };
        use std::io::stdout;

        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)?;

        // Main loop
        let result = self.run_loop(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        result
    }

    /// Main event loop (native terminal only)
    #[cfg(not(target_arch = "wasm32"))]
    fn run_loop<B: ratatui::backend::Backend>(&mut self, terminal: &mut ratatui::Terminal<B>) -> io::Result<()> {
        use crossterm::event::{poll, read, Event as CrosstermEvent, KeyCode, KeyEventKind};
        use std::time::Duration;

        while !self.should_quit() {
            // Render
            terminal.draw(|f| self.render(f))?;

            // Handle events with timeout
            if poll(Duration::from_millis(100))? {
                if let CrosstermEvent::Key(key) = read()? {
                    if key.kind == KeyEventKind::Press {
                        let message = self.handle_key_event(key.code);
                        self.handle_message(message);
                    }
                }
            }

            // Update metrics with simulated data
            self.simulate_metrics_update();
        }

        Ok(())
    }

    /// Handle keyboard input and convert to messages (native only)
    #[cfg(not(target_arch = "wasm32"))]
    fn handle_key_event(&self, key_code: KeyCode) -> Message {
        use crossterm::event::KeyCode;

        match key_code {
            KeyCode::Char('q') | KeyCode::Char('Q') => Message::Quit,
            KeyCode::Char('?') => Message::ToggleHelp,
            KeyCode::Char('1') => Message::Navigate(Screen::Dashboard),
            KeyCode::Char('2') => Message::Navigate(Screen::Settings),
            KeyCode::Char('3') => Message::Navigate(Screen::DataView),
            KeyCode::Char('4') => Message::Navigate(Screen::Help),
            KeyCode::Tab => {
                // Cycle through screens
                let next_screen = match self.state.current_screen() {
                    Screen::Dashboard => Screen::Settings,
                    Screen::Settings => Screen::DataView,
                    Screen::DataView => Screen::Help,
                    Screen::Help => Screen::Dashboard,
                };
                Message::Navigate(next_screen)
            }
            KeyCode::Esc => {
                if self.state.show_help {
                    Message::ToggleHelp
                } else {
                    Message::NavigateBack
                }
            }
            KeyCode::Up => Message::SelectPrevious,
            KeyCode::Down => Message::SelectNext,
            KeyCode::Char('+') | KeyCode::Char('=') => Message::Increment,
            KeyCode::Char('-') => Message::Decrement,
            KeyCode::Char('a') | KeyCode::Char('A') => {
                Message::AddItem(format!("Item {}", self.state.items.len() + 1))
            }
            KeyCode::Char('d') | KeyCode::Char('D') => Message::RemoveItem,
            KeyCode::Char('r') | KeyCode::Char('R') => Message::Refresh,
            _ => Message::None,
        }
    }

    /// Simulate metrics updates for demo purposes
    fn simulate_metrics_update(&mut self) {
        // Simple simulation - in real app, these would come from system monitoring
        use std::f64::consts::PI;

        let time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let cpu = ((time * 0.5).sin() * 0.3 + 0.5).clamp(0.0, 1.0);
        let memory = ((time * 0.3).cos() * 0.2 + 0.6).clamp(0.0, 1.0);
        let network = ((time * 0.8).sin() * 0.4 + 0.4).clamp(0.0, 1.0);

        self.state.update_metrics(cpu, memory, network);
    }
}

impl Default for TerminalApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a centered rectangle
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = TerminalApp::new();
        assert!(!app.should_quit());
        assert_eq!(app.state().current_screen(), Screen::Dashboard);
    }

    #[test]
    fn test_quit_message() {
        let mut app = TerminalApp::new();
        assert!(!app.should_quit());

        app.handle_message(Message::Quit);
        assert!(app.should_quit());
    }

    #[test]
    fn test_navigation() {
        let mut app = TerminalApp::new();
        assert_eq!(app.state().current_screen(), Screen::Dashboard);

        app.handle_message(Message::Navigate(Screen::Settings));
        assert_eq!(app.state().current_screen(), Screen::Settings);

        app.handle_message(Message::NavigateBack);
        assert_eq!(app.state().current_screen(), Screen::Dashboard);
    }

    #[test]
    fn test_counter_operations() {
        let mut app = TerminalApp::new();
        assert_eq!(app.state().counter, 0);

        app.handle_message(Message::Increment);
        assert_eq!(app.state().counter, 1);

        app.handle_message(Message::Decrement);
        assert_eq!(app.state().counter, 0);
    }
}
