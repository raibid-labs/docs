//! UI rendering for the TUI dashboard
//!
//! This module contains the rendering logic for the 3-panel dashboard.

use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Row, Sparkline, Table},
    Frame,
};

use super::mock_data::{AgentStatus, JobStatus, MockAgent, MockJob, MockQueueData};

/// Main render function for the dashboard
pub fn render(
    frame: &mut Frame,
    jobs: &[MockJob],
    agents: &[MockAgent],
    queue_data: &MockQueueData,
) {
    let size = frame.size();

    // Create main layout with header and content
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Render header
    render_header(frame, main_chunks[0]);

    // Create 3-panel layout for content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Jobs panel
            Constraint::Percentage(20), // Agents panel
            Constraint::Percentage(20), // Queue panel
        ])
        .split(main_chunks[1]);

    // Render panels
    render_jobs_panel(frame, content_chunks[0], jobs);
    render_agents_panel(frame, content_chunks[1], agents);
    render_queue_panel(frame, content_chunks[2], queue_data);

    // Render footer
    render_footer(frame, main_chunks[2]);
}

/// Render the header with title and system info
fn render_header(frame: &mut Frame, area: Rect) {
    let now = Local::now();
    let time_str = now.format("%Y-%m-%d %H:%M:%S").to_string();

    let header = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let header_text = vec![Line::from(vec![
        Span::styled(
            " Raibid CI Dashboard ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled("DGX Spark Agent Pool", Style::default().fg(Color::Gray)),
        Span::raw(" | "),
        Span::styled(time_str, Style::default().fg(Color::Yellow)),
    ])];

    let paragraph = ratatui::widgets::Paragraph::new(header_text)
        .block(header)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render the footer with keybindings
fn render_footer(frame: &mut Frame, area: Rect) {
    let footer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let footer_text = vec![Line::from(vec![
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Quit | "),
        Span::styled(
            "Ctrl+C",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" Exit"),
    ])];

    let paragraph = ratatui::widgets::Paragraph::new(footer_text)
        .block(footer)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, area);
}

/// Render the jobs panel with status table
fn render_jobs_panel(frame: &mut Frame, area: Rect, jobs: &[MockJob]) {
    let block = Block::default()
        .title(format!(" Jobs ({}) ", jobs.len()))
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    // Create table header
    let header = Row::new(vec!["ID", "Repo", "Branch", "Status", "Progress"])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    // Create table rows
    let rows: Vec<Row> = jobs
        .iter()
        .map(|job| {
            let status_style = match job.status {
                JobStatus::Success => Style::default().fg(Color::Green),
                JobStatus::Failed => Style::default().fg(Color::Red),
                JobStatus::Running => Style::default().fg(Color::Blue),
                JobStatus::Pending => Style::default().fg(Color::Gray),
            };

            let progress_bar = if job.progress > 0 {
                format!("{:3}% {}", job.progress, progress_indicator(job.progress))
            } else {
                "  -".to_string()
            };

            Row::new(vec![
                Cell::from(job.id.clone()),
                Cell::from(job.repo.clone()),
                Cell::from(job.branch.clone()),
                Cell::from(format!("{} {}", job.status.icon(), job.status.as_str()))
                    .style(status_style),
                Cell::from(progress_bar),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(10), // ID
        Constraint::Length(20), // Repo
        Constraint::Length(18), // Branch
        Constraint::Length(12), // Status
        Constraint::Min(10),    // Progress
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(block)
        .column_spacing(1);

    frame.render_widget(table, area);
}

/// Render the agents panel with agent list and resource usage
fn render_agents_panel(frame: &mut Frame, area: Rect, agents: &[MockAgent]) {
    let block = Block::default()
        .title(format!(" Agents ({}) ", agents.len()))
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let items: Vec<ListItem> = agents
        .iter()
        .map(|agent| {
            let status_style = match agent.status {
                AgentStatus::Idle => Style::default().fg(Color::Green),
                AgentStatus::Busy => Style::default().fg(Color::Yellow),
                AgentStatus::Starting => Style::default().fg(Color::Blue),
                AgentStatus::Stopping => Style::default().fg(Color::Gray),
            };

            let uptime_str = format_uptime(agent.uptime);

            let content = vec![
                Line::from(vec![
                    Span::styled(
                        agent.name.clone(),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" - "),
                    Span::styled(agent.status.as_str(), status_style),
                ]),
                Line::from(vec![
                    Span::raw("  CPU: "),
                    Span::styled(
                        format!("{:3}% ", agent.cpu),
                        Style::default().fg(cpu_color(agent.cpu)),
                    ),
                    Span::raw(resource_bar(agent.cpu)),
                ]),
                Line::from(vec![
                    Span::raw("  MEM: "),
                    Span::styled(
                        format!("{:3}% ", agent.memory),
                        Style::default().fg(memory_color(agent.memory)),
                    ),
                    Span::raw(resource_bar(agent.memory)),
                ]),
                Line::from(vec![
                    Span::raw("  UP:  "),
                    Span::styled(uptime_str, Style::default().fg(Color::Gray)),
                ]),
                Line::from(""), // Empty line for spacing
            ];

            ListItem::new(content)
        })
        .collect();

    let list = List::new(items).block(block);

    frame.render_widget(list, area);
}

/// Render the queue panel with sparkline chart
fn render_queue_panel(frame: &mut Frame, area: Rect, queue_data: &MockQueueData) {
    let _block = Block::default()
        .title(format!(" Queue Depth ({}) ", queue_data.current))
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    // Split the queue panel into info and sparkline sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),     // Info section
            Constraint::Length(10), // Sparkline chart
        ])
        .split(area);

    // Render queue info
    let max_depth = *queue_data.history.iter().max().unwrap_or(&0);
    let avg_depth = if !queue_data.history.is_empty() {
        queue_data.history.iter().sum::<u64>() / queue_data.history.len() as u64
    } else {
        0
    };

    let info_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Current: "),
            Span::styled(
                queue_data.current.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::raw("  Max:     "),
            Span::styled(max_depth.to_string(), Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::raw("  Average: "),
            Span::styled(avg_depth.to_string(), Style::default().fg(Color::Cyan)),
        ]),
    ];

    let info_paragraph = ratatui::widgets::Paragraph::new(info_text).block(Block::default());

    frame.render_widget(info_paragraph, chunks[0]);

    // Render sparkline
    let sparkline_block = Block::default()
        .title(" History (60s) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));

    let sparkline = Sparkline::default()
        .block(sparkline_block)
        .data(&queue_data.history)
        .style(Style::default().fg(Color::Cyan))
        .max(30); // Max queue depth for scaling

    frame.render_widget(sparkline, chunks[1]);
}

/// Generate a simple progress indicator
fn progress_indicator(progress: u8) -> String {
    let filled = (progress / 10) as usize;
    let empty = 10 - filled;
    format!("[{}{}]", "█".repeat(filled), " ".repeat(empty))
}

/// Generate a simple resource usage bar
fn resource_bar(percentage: u8) -> String {
    let filled = (percentage / 10) as usize;
    let empty = 10 - filled;
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

/// Get color for CPU usage
fn cpu_color(cpu: u8) -> Color {
    match cpu {
        0..=50 => Color::Green,
        51..=80 => Color::Yellow,
        _ => Color::Red,
    }
}

/// Get color for memory usage
fn memory_color(memory: u8) -> Color {
    match memory {
        0..=60 => Color::Green,
        61..=85 => Color::Yellow,
        _ => Color::Red,
    }
}

/// Format uptime in human-readable form
fn format_uptime(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}h {:02}m {:02}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {:02}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_indicator() {
        assert_eq!(progress_indicator(0), "[          ]");
        assert_eq!(progress_indicator(50), "[█████     ]");
        assert_eq!(progress_indicator(100), "[██████████]");
    }

    #[test]
    fn test_resource_bar() {
        assert_eq!(resource_bar(0), "[░░░░░░░░░░]");
        assert_eq!(resource_bar(50), "[█████░░░░░]");
        assert_eq!(resource_bar(100), "[██████████]");
    }

    #[test]
    fn test_cpu_color() {
        assert_eq!(cpu_color(30), Color::Green);
        assert_eq!(cpu_color(70), Color::Yellow);
        assert_eq!(cpu_color(90), Color::Red);
    }

    #[test]
    fn test_memory_color() {
        assert_eq!(memory_color(30), Color::Green);
        assert_eq!(memory_color(70), Color::Yellow);
        assert_eq!(memory_color(90), Color::Red);
    }

    #[test]
    fn test_format_uptime() {
        assert_eq!(format_uptime(30), "30s");
        assert_eq!(format_uptime(90), "1m 30s");
        assert_eq!(format_uptime(3661), "1h 01m 01s");
    }
}
