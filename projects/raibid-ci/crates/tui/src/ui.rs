//! UI rendering for the TUI dashboard
//!
//! This module contains the rendering logic for the 3-panel dashboard.

use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Sparkline, Table, Tabs},
    Frame,
};

use super::app::{InputMode, Tab, UiState};
use super::mock_data::{
    generate_system_logs, AgentStatus, JobStatus, LogLevel, MockAgent, MockJob, MockJobLogs,
    MockQueueData,
};

/// Main render function for the dashboard
#[allow(clippy::too_many_arguments)]
pub fn render(
    frame: &mut Frame,
    jobs: &[MockJob],
    agents: &[MockAgent],
    queue_data: &MockQueueData,
    current_tab: Tab,
    selected_job: usize,
    selected_agent: usize,
    ui_state: &UiState,
) {
    let size = frame.size();

    // Create main layout with header, tabs, content, and footer
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Render header
    render_header(frame, main_chunks[0]);

    // Render tabs
    render_tabs(frame, main_chunks[1], current_tab);

    // Render content based on current tab
    match current_tab {
        Tab::Jobs => render_jobs_tab(
            frame,
            main_chunks[2],
            jobs,
            agents,
            queue_data,
            selected_job,
        ),
        Tab::Agents => render_agents_tab(frame, main_chunks[2], agents, selected_agent),
        Tab::Config => render_config_tab(frame, main_chunks[2]),
        Tab::Logs => render_logs_tab(frame, main_chunks[2]),
    }

    // Render footer
    render_footer(frame, main_chunks[3], ui_state);

    // Render overlays (popups, help screen, etc.)
    if ui_state.show_help {
        render_help_screen(frame, size);
    } else if ui_state.show_detail_popup {
        if let Some(job) = jobs.get(selected_job) {
            render_job_detail_popup(frame, size, job);
        }
    } else if ui_state.show_filter_menu {
        render_filter_menu(frame, size, ui_state);
    } else if ui_state.show_confirmation {
        render_confirmation_dialog(frame, size, ui_state.confirmation_message);
    }
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

/// Render the tab bar
fn render_tabs(frame: &mut Frame, area: Rect, current_tab: Tab) {
    let all_tabs = Tab::all();
    let tab_titles: Vec<Line> = all_tabs.iter().map(|t| Line::from(t.as_str())).collect();

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .select(match current_tab {
            Tab::Jobs => 0,
            Tab::Agents => 1,
            Tab::Config => 2,
            Tab::Logs => 3,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

/// Render the footer with keybindings
fn render_footer(frame: &mut Frame, area: Rect, ui_state: &UiState) {
    let footer = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let mut footer_spans = Vec::new();

    // Context-specific shortcuts
    match ui_state.input_mode {
        InputMode::Search => {
            footer_spans.extend(vec![
                Span::styled("Search: ", Style::default().fg(Color::Cyan)),
                Span::raw(ui_state.search_query),
                Span::raw(" | "),
                Span::styled(
                    "Enter/Esc",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Exit"),
            ]);
        }
        InputMode::Filter => {
            footer_spans.extend(vec![
                Span::styled("Filter Mode", Style::default().fg(Color::Cyan)),
                Span::raw(" | "),
                Span::styled(
                    "Up/Down",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Select | "),
                Span::styled(
                    "Enter",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Apply | "),
                Span::styled(
                    "Esc",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(" Cancel"),
            ]);
        }
        InputMode::Normal => {
            if ui_state.show_confirmation {
                footer_spans.extend(vec![
                    Span::styled(
                        "Y",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Confirm | "),
                    Span::styled(
                        "N/Esc",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Cancel"),
                ]);
            } else if ui_state.show_help || ui_state.show_detail_popup {
                footer_spans.extend(vec![
                    Span::styled(
                        "Esc",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Close"),
                ]);
            } else {
                footer_spans.extend(vec![
                    Span::styled(
                        "Tab/←/→",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Switch | "),
                    Span::styled(
                        "↑/↓",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Navigate | "),
                    Span::styled(
                        "Enter",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Details | "),
                    Span::styled(
                        "f",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Filter | "),
                    Span::styled(
                        "/",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Search | "),
                    Span::styled(
                        "?",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Help | "),
                    Span::styled(
                        "q",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" Quit"),
                ]);
            }
        }
    }

    let footer_text = vec![Line::from(footer_spans)];

    let paragraph = Paragraph::new(footer_text)
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
    let header = Row::new(vec![
        "ID", "Repo", "Branch", "Status", "Progress", "Duration",
    ])
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
                JobStatus::Running => Style::default().fg(Color::Yellow),
                JobStatus::Pending => Style::default().fg(Color::Gray),
            };

            let progress_bar = if job.progress > 0 {
                format!("{:3}% {}", job.progress, progress_indicator(job.progress))
            } else {
                "  -".to_string()
            };

            let duration_str = if let Some(duration) = job.duration {
                format_duration(duration)
            } else {
                match job.status {
                    JobStatus::Running => {
                        let elapsed = (chrono::Utc::now() - job.start_time).num_seconds() as u64;
                        format_duration(elapsed)
                    }
                    _ => "-".to_string(),
                }
            };

            Row::new(vec![
                Cell::from(job.id.clone()),
                Cell::from(job.repo.clone()),
                Cell::from(job.branch.clone()),
                Cell::from(format!("{} {}", job.status.icon(), job.status.as_str()))
                    .style(status_style),
                Cell::from(progress_bar),
                Cell::from(duration_str),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(10), // ID
        Constraint::Length(20), // Repo
        Constraint::Length(18), // Branch
        Constraint::Length(12), // Status
        Constraint::Length(18), // Progress
        Constraint::Min(8),     // Duration
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
#[allow(dead_code)]
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

/// Format duration in human-readable form (same as uptime but different context)
fn format_duration(seconds: u64) -> String {
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

/// Render the Jobs tab (detailed view)
#[allow(clippy::too_many_arguments)]
#[allow(dead_code)]
fn render_jobs_tab(
    frame: &mut Frame,
    area: Rect,
    jobs: &[MockJob],
    agents: &[MockAgent],
    queue_data: &MockQueueData,
    _selected: usize,
) {
    // Create 3-panel layout for content
    let content_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60), // Jobs panel
            Constraint::Percentage(20), // Agents panel
            Constraint::Percentage(20), // Queue panel
        ])
        .split(area);

    // Render panels
    render_jobs_panel(frame, content_chunks[0], jobs);
    render_agents_panel(frame, content_chunks[1], agents);
    render_queue_panel(frame, content_chunks[2], queue_data);
}

/// Render the Agents tab (detailed view)
#[allow(dead_code)]
fn render_agents_tab(frame: &mut Frame, area: Rect, agents: &[MockAgent], _selected: usize) {
    // For now, delegate to the agents panel implementation
    render_agents_panel(frame, area, agents);
}

/// Render the Config tab (placeholder)
#[allow(dead_code)]
fn render_config_tab(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Configuration ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let config_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "API Server:  ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("http://localhost:8080"),
        ]),
        Line::from(vec![
            Span::styled(
                "Gitea URL:   ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("http://gitea.local:3000"),
        ]),
        Line::from(vec![
            Span::styled(
                "Redis URL:   ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("redis://localhost:6379"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Job Queue:   ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("raibid:jobs"),
        ]),
        Line::from(vec![
            Span::styled(
                "Max Agents:  ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("10"),
        ]),
        Line::from(vec![
            Span::styled(
                "Scale Down:  ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("300s idle"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Platform:    ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("NVIDIA DGX Spark (ARM64)"),
        ]),
        Line::from(vec![
            Span::styled(
                "CPU Cores:   ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("20 (10x Cortex-X925, 10x Cortex-A725)"),
        ]),
        Line::from(vec![
            Span::styled(
                "Memory:      ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("128GB LPDDR5x"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Note: Configuration editing coming in future release",
            Style::default().fg(Color::Gray),
        )]),
    ];

    let paragraph = Paragraph::new(config_text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Left);

    frame.render_widget(paragraph, area);
}

/// Render the Logs tab with scrolling system logs
#[allow(dead_code)]
fn render_logs_tab(frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" System Logs (Use ↑/↓ to scroll) ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    let system_logs = generate_system_logs();

    let log_entries: Vec<Line> = system_logs
        .iter()
        .map(|entry| {
            let level_color = match entry.level {
                LogLevel::Info => Color::Green,
                LogLevel::Warn => Color::Yellow,
                LogLevel::Error => Color::Red,
            };

            Line::from(vec![
                Span::styled(
                    format!("[{}]", entry.timestamp.format("%Y-%m-%d %H:%M:%S")),
                    Style::default().fg(Color::Gray),
                ),
                Span::raw(" "),
                Span::styled(
                    format!("{:5}", entry.level.as_str()),
                    Style::default()
                        .fg(level_color)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("  "),
                Span::raw(&entry.message),
            ])
        })
        .collect();

    let paragraph = Paragraph::new(log_entries)
        .block(block)
        .alignment(ratatui::layout::Alignment::Left);

    frame.render_widget(paragraph, area);
}

/// Render centered popup area
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

/// Render help screen
fn render_help_screen(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(80, 80, area);

    // Clear the background
    let clear_block = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(clear_block, area);

    let block = Block::default()
        .title(" Keyboard Shortcuts - Press any key to close ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));

    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "NAVIGATION",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Tab / Left / Right", Style::default().fg(Color::Green)),
            Span::raw("  Switch between tabs"),
        ]),
        Line::from(vec![
            Span::styled("  1 / 2 / 3 / 4", Style::default().fg(Color::Green)),
            Span::raw("        Jump directly to tab (Jobs/Agents/Config/Logs)"),
        ]),
        Line::from(vec![
            Span::styled("  Up / Down", Style::default().fg(Color::Green)),
            Span::raw("            Navigate items in lists"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "ACTIONS",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Green)),
            Span::raw("                 View job details (on Jobs tab)"),
        ]),
        Line::from(vec![
            Span::styled("  c", Style::default().fg(Color::Green)),
            Span::raw("                     Cancel selected job"),
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(Color::Green)),
            Span::raw("                     Refresh data"),
        ]),
        Line::from(vec![
            Span::styled("  f", Style::default().fg(Color::Green)),
            Span::raw("                     Filter jobs by status"),
        ]),
        Line::from(vec![
            Span::styled("  /", Style::default().fg(Color::Green)),
            Span::raw("                     Search jobs (by repo/branch/ID)"),
        ]),
        Line::from(vec![
            Span::styled("  Esc", Style::default().fg(Color::Green)),
            Span::raw("                   Close popup / Clear filters"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "GENERAL",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ?", Style::default().fg(Color::Green)),
            Span::raw("                     Show this help screen"),
        ]),
        Line::from(vec![
            Span::styled("  q / Ctrl+C", Style::default().fg(Color::Green)),
            Span::raw("            Quit application"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "IN JOB DETAIL POPUP",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  c", Style::default().fg(Color::Green)),
            Span::raw("                     Cancel job"),
        ]),
        Line::from(vec![
            Span::styled("  r", Style::default().fg(Color::Green)),
            Span::raw("                     Refresh job data"),
        ]),
        Line::from(vec![
            Span::styled("  Esc", Style::default().fg(Color::Green)),
            Span::raw("                   Close popup"),
        ]),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Left);

    frame.render_widget(paragraph, popup_area);
}

/// Render job detail popup
fn render_job_detail_popup(frame: &mut Frame, area: Rect, job: &MockJob) {
    let popup_area = centered_rect(90, 85, area);

    // Clear the background
    let clear_block = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(clear_block, area);

    let block = Block::default()
        .title(format!(" Job Details: {} ", job.id))
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    // Split popup into info and logs sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(10), // Job info
            Constraint::Min(0),     // Logs
        ])
        .split(block.inner(popup_area));

    // Render the block border
    frame.render_widget(block, popup_area);

    // Job info section
    let status_style = match job.status {
        JobStatus::Success => Style::default().fg(Color::Green),
        JobStatus::Failed => Style::default().fg(Color::Red),
        JobStatus::Running => Style::default().fg(Color::Yellow),
        JobStatus::Pending => Style::default().fg(Color::Gray),
    };

    let duration_str = if let Some(duration) = job.duration {
        format_duration(duration)
    } else {
        match job.status {
            JobStatus::Running => {
                let elapsed = (chrono::Utc::now() - job.start_time).num_seconds() as u64;
                format!("{} (running)", format_duration(elapsed))
            }
            _ => "N/A".to_string(),
        }
    };

    let info_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Repository: ", Style::default().fg(Color::Yellow)),
            Span::raw(&job.repo),
        ]),
        Line::from(vec![
            Span::styled("Branch:     ", Style::default().fg(Color::Yellow)),
            Span::raw(&job.branch),
        ]),
        Line::from(vec![
            Span::styled("Status:     ", Style::default().fg(Color::Yellow)),
            Span::styled(
                format!("{} {}", job.status.icon(), job.status.as_str()),
                status_style,
            ),
        ]),
        Line::from(vec![
            Span::styled("Progress:   ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{}%", job.progress)),
        ]),
        Line::from(vec![
            Span::styled("Duration:   ", Style::default().fg(Color::Yellow)),
            Span::raw(duration_str),
        ]),
        Line::from(vec![
            Span::styled("Started:    ", Style::default().fg(Color::Yellow)),
            Span::raw(job.start_time.format("%Y-%m-%d %H:%M:%S").to_string()),
        ]),
    ];

    let info_para = Paragraph::new(info_text);
    frame.render_widget(info_para, chunks[0]);

    // Logs section
    let logs = MockJobLogs::for_job(job);
    let log_lines = logs.formatted_lines();

    let logs_block = Block::default()
        .title(" Build Logs ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));

    let log_text: Vec<Line> = log_lines
        .iter()
        .map(|line| Line::from(line.clone()))
        .collect();

    let logs_para = Paragraph::new(log_text)
        .block(logs_block)
        .alignment(ratatui::layout::Alignment::Left);

    frame.render_widget(logs_para, chunks[1]);
}

/// Render filter menu
fn render_filter_menu(frame: &mut Frame, area: Rect, ui_state: &UiState) {
    let popup_area = centered_rect(30, 30, area);

    let block = Block::default()
        .title(" Filter by Status ")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    let options = ["All", "Running", "Success", "Failed", "Pending"];
    let items: Vec<ListItem> = options
        .iter()
        .enumerate()
        .map(|(i, option)| {
            let style = if i == ui_state.selected_filter_option {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if i == ui_state.selected_filter_option {
                "> "
            } else {
                "  "
            };

            ListItem::new(format!("{}{}", prefix, option)).style(style)
        })
        .collect();

    let list = List::new(items).block(block);
    frame.render_widget(list, popup_area);
}

/// Render confirmation dialog
fn render_confirmation_dialog(frame: &mut Frame, area: Rect, message: &str) {
    let popup_area = centered_rect(50, 20, area);

    // Clear the background
    let clear_block = Block::default().style(Style::default().bg(Color::Black));
    frame.render_widget(clear_block, area);

    let block = Block::default()
        .title(" Confirmation ")
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Black));

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            message,
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "Y",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" Yes  "),
            Span::styled(
                "N",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(" No"),
        ]),
    ];

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(paragraph, popup_area);
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
