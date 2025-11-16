//! Interactive example
//!
//! Demonstrates event handling, state management, and user interaction.
//! Shows webatui's interactive features including buttons, lists, and navigation.
//!
//! ## Features
//! - Button interactions
//! - List navigation with selection
//! - Form-like input handling
//! - Menu navigation
//! - State updates based on user actions
//!
//! ## Running
//! ```bash
//! cargo run --example interactive
//! ```

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame, Terminal,
};
use std::io;

/// Interactive application state
struct InteractiveApp {
    should_quit: bool,
    /// Counter for button clicks
    counter: i32,
    /// List of items
    items: Vec<String>,
    /// Selected item in the list
    list_state: ListState,
    /// Current focus area
    focus: Focus,
    /// Status message
    status: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Focus {
    Counter,
    List,
    Menu,
}

impl InteractiveApp {
    fn new() -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            should_quit: false,
            counter: 0,
            items: vec![
                "First Item".to_string(),
                "Second Item".to_string(),
                "Third Item".to_string(),
                "Fourth Item".to_string(),
                "Fifth Item".to_string(),
            ],
            list_state,
            focus: Focus::Counter,
            status: "Press Tab to switch focus, Enter to interact".to_string(),
        }
    }

    fn handle_event(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                        KeyCode::Tab => {
                            self.focus = match self.focus {
                                Focus::Counter => Focus::List,
                                Focus::List => Focus::Menu,
                                Focus::Menu => Focus::Counter,
                            };
                            self.status = format!("Focus: {:?}", self.focus);
                        }
                        KeyCode::Enter => self.handle_action(),
                        KeyCode::Up => self.handle_up(),
                        KeyCode::Down => self.handle_down(),
                        KeyCode::Char('+') => {
                            if self.focus == Focus::Counter {
                                self.counter += 1;
                                self.status = format!("Counter incremented to {}", self.counter);
                            }
                        }
                        KeyCode::Char('-') => {
                            if self.focus == Focus::Counter {
                                self.counter -= 1;
                                self.status = format!("Counter decremented to {}", self.counter);
                            }
                        }
                        KeyCode::Char('a') => {
                            if self.focus == Focus::List {
                                self.items
                                    .push(format!("New Item {}", self.items.len() + 1));
                                self.status = "Item added to list".to_string();
                            }
                        }
                        KeyCode::Char('d') => {
                            if self.focus == Focus::List && !self.items.is_empty() {
                                if let Some(selected) = self.list_state.selected() {
                                    if selected < self.items.len() {
                                        self.items.remove(selected);
                                        let new_selected = if self.items.is_empty() {
                                            None
                                        } else if selected >= self.items.len() {
                                            Some(self.items.len() - 1)
                                        } else {
                                            Some(selected)
                                        };
                                        self.list_state.select(new_selected);
                                        self.status = "Item removed from list".to_string();
                                    }
                                }
                            }
                        }
                        KeyCode::Char('r') => {
                            self.counter = 0;
                            self.items.clear();
                            self.items = vec![
                                "First Item".to_string(),
                                "Second Item".to_string(),
                                "Third Item".to_string(),
                            ];
                            self.list_state.select(Some(0));
                            self.status = "State reset".to_string();
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_action(&mut self) {
        match self.focus {
            Focus::Counter => {
                self.counter += 10;
                self.status = format!("Button clicked! Counter: {}", self.counter);
            }
            Focus::List => {
                if let Some(selected) = self.list_state.selected() {
                    if selected < self.items.len() {
                        self.status = format!("Selected: {}", self.items[selected]);
                    }
                }
            }
            Focus::Menu => {
                self.status = "Menu action triggered".to_string();
            }
        }
    }

    fn handle_up(&mut self) {
        if self.focus == Focus::List && !self.items.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i == 0 {
                        self.items.len() - 1
                    } else {
                        i - 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    fn handle_down(&mut self) {
        if self.focus == Focus::List && !self.items.is_empty() {
            let i = match self.list_state.selected() {
                Some(i) => {
                    if i >= self.items.len() - 1 {
                        0
                    } else {
                        i + 1
                    }
                }
                None => 0,
            };
            self.list_state.select(Some(i));
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(5),
                Constraint::Length(3),
            ])
            .split(frame.area());

        // Header
        self.render_header(frame, chunks[0]);

        // Main content
        let content_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        self.render_counter_section(frame, content_chunks[0]);
        self.render_list_section(frame, content_chunks[1]);

        // Status and menu
        self.render_menu(frame, chunks[2]);

        // Footer
        self.render_footer(frame, chunks[3]);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header = Paragraph::new("WebATUI Interactive Example")
            .style(Style::default().fg(Color::Cyan).bold())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, area);
    }

    fn render_counter_section(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == Focus::Counter;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        let counter_display = Paragraph::new(format!(
            "Counter Value:\n\n{}\n\n\
             Press + or - to adjust\n\
             Press Enter to add 10",
            self.counter
        ))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Counter")
                .border_style(border_style),
        );
        frame.render_widget(counter_display, chunks[0]);

        let button_style = if is_focused {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        };

        let button = Paragraph::new("[ Click Me! ]")
            .alignment(Alignment::Center)
            .style(button_style)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(button, chunks[1]);
    }

    fn render_list_section(&mut self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == Focus::List;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let items: Vec<ListItem> = self
            .items
            .iter()
            .map(|item| ListItem::new(item.clone()).style(Style::default().fg(Color::White)))
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Items List")
                    .border_style(border_style),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("→ ");

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_menu(&self, frame: &mut Frame, area: Rect) {
        let is_focused = self.focus == Focus::Menu;
        let border_style = if is_focused {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::White)
        };

        let menu_items = vec![
            Span::raw("  [ Reset ]  "),
            Span::raw("  [ Save ]  "),
            Span::raw("  [ Load ]  "),
        ];

        let menu = Paragraph::new(Line::from(menu_items))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Menu")
                    .border_style(border_style),
            );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(2)])
            .split(area);

        frame.render_widget(menu, chunks[0]);

        let status = Paragraph::new(self.status.clone())
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center);
        frame.render_widget(status, chunks[1]);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = Paragraph::new(
            "q: Quit | Tab: Switch focus | Enter: Action | ↑↓: Navigate | a: Add | d: Delete | r: Reset"
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        frame.render_widget(footer, area);
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
    let mut app = InteractiveApp::new();

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
