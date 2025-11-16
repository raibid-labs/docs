//! Basic example
//!
//! Minimal webatui application demonstrating core functionality.
//! This is a "Hello World" style application showing the fundamental
//! pattern for building webatui applications.
//!
//! ## Features
//! - Simple struct implementing the app pattern
//! - Basic update and render methods
//! - Keyboard event handling
//! - Clean exit on 'q' key
//!
//! ## Running
//! ```bash
//! cargo run --example basic
//! ```

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::io;

/// Basic application state
struct BasicApp {
    /// Whether the application should quit
    should_quit: bool,
    /// Counter for demonstrating state updates
    counter: u32,
}

impl BasicApp {
    /// Create a new basic application
    fn new() -> Self {
        Self {
            should_quit: false,
            counter: 0,
        }
    }

    /// Handle keyboard events
    fn handle_event(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                        KeyCode::Char('+') | KeyCode::Up => self.counter += 1,
                        KeyCode::Char('-') | KeyCode::Down => {
                            self.counter = self.counter.saturating_sub(1)
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    /// Render the UI
    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.area());

        // Header
        let header = Paragraph::new("WebATUI Basic Example")
            .style(Style::default().fg(Color::Cyan).bold())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, chunks[0]);

        // Main content
        let content = Paragraph::new(format!(
            "Welcome to webatui!\n\n\
             This is a minimal example demonstrating the core pattern.\n\n\
             Counter: {}\n\n\
             The same code structure works in both terminal and web browser.",
            self.counter
        ))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Main Content"),
        );
        frame.render_widget(content, chunks[1]);

        // Footer
        let footer = Paragraph::new("Press 'q' to quit | '+/-' or Up/Down to change counter")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, chunks[2]);
    }
}

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = BasicApp::new();

    // Main loop
    loop {
        terminal.draw(|frame| app.render(frame))?;
        app.handle_event()?;

        if app.should_quit {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
