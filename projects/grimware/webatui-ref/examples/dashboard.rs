//! Dashboard example
//!
//! Multi-widget dashboard demonstrating layout and component composition.
//! Shows how to build complex UIs by combining multiple components.
//!
//! ## Features
//! - Multiple widget areas
//! - Component composition
//! - Real-time updates
//! - Tab navigation between widgets
//! - State management across components
//!
//! ## Running
//! ```bash
//! cargo run --example dashboard
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
    widgets::{
        Bar, BarChart, BarGroup, Block, Borders, Gauge, List, ListItem, Paragraph, Sparkline,
        Tabs,
    },
    Frame, Terminal,
};
use std::io;

/// Dashboard application state
struct DashboardApp {
    should_quit: bool,
    /// Selected tab index
    selected_tab: usize,
    /// Counter values for different widgets
    counters: [u64; 4],
    /// Log messages
    logs: Vec<String>,
    /// Chart data
    chart_data: Vec<u64>,
}

impl DashboardApp {
    fn new() -> Self {
        Self {
            should_quit: false,
            selected_tab: 0,
            counters: [42, 78, 35, 91],
            logs: vec![
                "System initialized".to_string(),
                "Dashboard loaded".to_string(),
                "All components ready".to_string(),
            ],
            chart_data: vec![4, 7, 3, 9, 5, 8, 6],
        }
    }

    fn handle_event(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.should_quit = true,
                        KeyCode::Tab | KeyCode::Right => {
                            self.selected_tab = (self.selected_tab + 1) % 4;
                        }
                        KeyCode::BackTab | KeyCode::Left => {
                            self.selected_tab = if self.selected_tab == 0 {
                                3
                            } else {
                                self.selected_tab - 1
                            };
                        }
                        KeyCode::Char('u') => {
                            // Update counters
                            self.counters[self.selected_tab] += 1;
                            self.logs
                                .push(format!("Counter {} updated", self.selected_tab + 1));
                            if self.logs.len() > 10 {
                                self.logs.remove(0);
                            }
                        }
                        KeyCode::Char('r') => {
                            // Refresh data
                            self.chart_data.rotate_left(1);
                            self.chart_data[6] = (self.chart_data[5] + 2) % 10;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(frame.area());

        // Header
        self.render_header(frame, chunks[0]);

        // Tabs
        self.render_tabs(frame, chunks[1]);

        // Main content area
        self.render_content(frame, chunks[2]);

        // Footer
        self.render_footer(frame, chunks[3]);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header = Paragraph::new("WebATUI Dashboard Example")
            .style(Style::default().fg(Color::Cyan).bold())
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(header, area);
    }

    fn render_tabs(&self, frame: &mut Frame, area: Rect) {
        let tab_titles = vec!["Overview", "Metrics", "Charts", "Logs"];
        let tabs = Tabs::new(tab_titles)
            .block(Block::default().borders(Borders::ALL).title("Navigation"))
            .select(self.selected_tab)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow).bold());
        frame.render_widget(tabs, area);
    }

    fn render_content(&self, frame: &mut Frame, area: Rect) {
        match self.selected_tab {
            0 => self.render_overview(frame, area),
            1 => self.render_metrics(frame, area),
            2 => self.render_charts(frame, area),
            3 => self.render_logs(frame, area),
            _ => {}
        }
    }

    fn render_overview(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);

        // Status cards
        for (i, chunk) in top_chunks.iter().enumerate() {
            let card = Paragraph::new(format!(
                "Component {}\n\nStatus: Active\nValue: {}",
                i + 1,
                self.counters[i]
            ))
            .style(Style::default().fg(Color::Green))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!("Widget {}", i + 1)),
            );
            frame.render_widget(card, *chunk);
        }

        // Summary
        let summary = Paragraph::new(format!(
            "Dashboard Overview\n\n\
             Total Components: 4\n\
             Active Widgets: 4\n\
             Logs: {}\n\
             Data Points: {}",
            self.logs.len(),
            self.chart_data.len()
        ))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).title("Summary"));
        frame.render_widget(summary, chunks[1]);
    }

    fn render_metrics(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(area);

        for (i, chunk) in chunks.iter().take(3).enumerate() {
            let ratio = self.counters[i] as f64 / 100.0;
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL).title(format!(
                    "Metric {} - {}%",
                    i + 1,
                    (ratio * 100.0) as u64
                )))
                .gauge_style(Style::default().fg(Color::Cyan))
                .ratio(ratio.min(1.0));
            frame.render_widget(gauge, *chunk);
        }

        // Sparkline
        let sparkline = Sparkline::default()
            .block(Block::default().borders(Borders::ALL).title("Trend"))
            .data(&self.chart_data)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(sparkline, chunks[3]);
    }

    fn render_charts(&self, frame: &mut Frame, area: Rect) {
        let data: Vec<(&str, u64)> = self
            .chart_data
            .iter()
            .enumerate()
            .map(|(i, &v)| (["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][i], v))
            .collect();

        let bars: Vec<Bar> = data
            .iter()
            .map(|(label, value)| Bar::default().value(*value).label(Line::from(*label)))
            .collect();

        let group = BarGroup::default().bars(&bars);
        let chart = BarChart::default()
            .block(Block::default().borders(Borders::ALL).title("Weekly Data"))
            .data(group)
            .bar_width(5)
            .bar_gap(1)
            .bar_style(Style::default().fg(Color::Green))
            .value_style(Style::default().fg(Color::White).bold());

        frame.render_widget(chart, area);
    }

    fn render_logs(&self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .logs
            .iter()
            .enumerate()
            .map(|(i, log)| {
                ListItem::new(format!("[{}] {}", i + 1, log))
                    .style(Style::default().fg(Color::White))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title("Activity Logs"),
        );

        frame.render_widget(list, area);
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = Paragraph::new("q: Quit | Tab/←→: Switch tabs | u: Update | r: Refresh data")
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
    let mut app = DashboardApp::new();

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
