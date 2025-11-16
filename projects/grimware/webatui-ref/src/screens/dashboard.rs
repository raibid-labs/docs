//! Dashboard screen
//!
//! Multi-widget dashboard view with system metrics and process monitoring

use crate::prelude::*;
use crate::components::gauge::GaugeWidget;
use crate::components::chart::NetworkChart;
use crate::components::metrics_state::MetricsState;

/// Dashboard screen with system metrics
pub struct DashboardScreen {
    /// Metrics data
    pub metrics: MetricsState,
    /// Selected process index
    pub selected_process: usize,
    /// Network chart
    network_chart: NetworkChart,
}

impl DashboardScreen {
    /// Create a new dashboard screen
    pub fn new() -> Self {
        let mut metrics = MetricsState::new(60);
        // Initialize with some data
        metrics.simulate_processes();
        for _ in 0..30 {
            metrics.simulate_update();
        }

        Self {
            metrics,
            selected_process: 0,
            network_chart: NetworkChart::new(60),
        }
    }

    /// Update metrics (call this periodically for live updates)
    pub fn update_metrics(&mut self) {
        self.metrics.simulate_update();

        // Update network chart
        let upload = self.metrics.network_upload.back().copied().unwrap_or(0);
        let download = self.metrics.network_download.back().copied().unwrap_or(0);
        self.network_chart.add_data(upload, download);
    }

    /// Render the dashboard screen
    pub fn render(frame: &mut Frame, area: Rect, metrics: &MetricsState, selected_process: usize) {
        // Main layout: header, body, footer
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // Body
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // Render header
        Self::render_header(frame, main_chunks[0]);

        // Body layout: metrics + network on top, processes + actions on bottom
        let body_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40),  // Metrics panels
                Constraint::Percentage(60),  // Processes and actions
            ])
            .split(main_chunks[1]);

        // Top section: system metrics + network
        let top_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50),  // System metrics
                Constraint::Percentage(50),  // Network
            ])
            .split(body_chunks[0]);

        // Render system metrics (2x2 grid)
        Self::render_system_metrics(frame, top_chunks[0], metrics);

        // Render network chart
        Self::render_network_panel(frame, top_chunks[1], metrics);

        // Bottom section: processes table + quick actions
        let bottom_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(80),  // Processes
                Constraint::Percentage(20),  // Quick actions
            ])
            .split(body_chunks[1]);

        // Render processes table
        Self::render_processes(frame, bottom_chunks[0], metrics, selected_process);

        // Render quick actions
        Self::render_quick_actions(frame, bottom_chunks[1]);

        // Render footer
        Self::render_footer(frame, main_chunks[2], metrics);
    }

    /// Render header
    fn render_header(frame: &mut Frame, area: Rect) {
        let _header = Paragraph::new("WebaTUI Dashboard")
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .block(
                Block::bordered()
                    .border_type(ratatui::widgets::BorderType::Thick)
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .alignment(Alignment::Left);

        // Add help text on the right
        let header_text = vec![
            Line::from(vec![
                Span::styled("WebaTUI Dashboard", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::raw("                         "),
                Span::styled("[?]", Style::default().fg(Color::Yellow)),
                Span::raw(" Help  "),
                Span::styled("[⚙]", Style::default().fg(Color::Yellow)),
                Span::raw(" Settings  "),
                Span::styled("[Q]", Style::default().fg(Color::Red)),
                Span::raw(" Quit"),
            ])
        ];

        let header_widget = Paragraph::new(header_text)
            .block(
                Block::bordered()
                    .border_type(ratatui::widgets::BorderType::Thick)
                    .border_style(Style::default().fg(Color::Cyan))
            );

        frame.render_widget(header_widget, area);
    }

    /// Render system metrics (CPU, Memory, Disk, Temp)
    fn render_system_metrics(frame: &mut Frame, area: Rect, metrics: &MetricsState) {
        let block = Block::bordered()
            .title("System Metrics")
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // 2x2 grid for gauges
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner);

        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[0]);

        let bottom_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        // Render CPU gauge
        let cpu_gauge = GaugeWidget::new("CPU Usage", metrics.current_cpu());
        cpu_gauge.render(frame, top_cols[0]);

        // Render Memory gauge
        let mem_gauge = GaugeWidget::new("Memory", metrics.current_memory());
        mem_gauge.render(frame, top_cols[1]);

        // Render Disk I/O gauge
        let disk_gauge = GaugeWidget::new("Disk I/O", metrics.current_disk());
        disk_gauge.render(frame, bottom_cols[0]);

        // Render Temperature display
        let temp_percent = (metrics.current_temp() / 100.0).clamp(0.0, 1.0);
        let temp_label = format!("Temp ({}°C)", metrics.current_temp() as u16);
        let temp_gauge = GaugeWidget::new(temp_label, temp_percent);
        temp_gauge.render(frame, bottom_cols[1]);
    }

    /// Render network activity panel
    fn render_network_panel(frame: &mut Frame, area: Rect, metrics: &MetricsState) {
        let block = Block::bordered()
            .title("Network Activity")
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Convert VecDeque to Vec for chart
        let download_data: Vec<(f64, f64)> = metrics.network_download
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v as f64))
            .collect();

        let upload_data: Vec<(f64, f64)> = metrics.network_upload
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v as f64))
            .collect();

        // Find max value for y-axis
        let max_value = metrics.network_download.iter()
            .chain(metrics.network_upload.iter())
            .max()
            .copied()
            .unwrap_or(100) as f64;

        // Render chart
        let datasets = vec![
            Dataset::default()
                .name("Download")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Green))
                .graph_type(ratatui::widgets::GraphType::Line)
                .data(&download_data),
            Dataset::default()
                .name("Upload")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(ratatui::widgets::GraphType::Line)
                .data(&upload_data),
        ];

        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .title("Time (s)")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, 60.0])
            )
            .y_axis(
                Axis::default()
                    .title("MB/s")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, max_value * 1.1])
                    .labels(vec![
                        Line::from("0"),
                        Line::from(format!("{:.0}", max_value * 0.5)),
                        Line::from(format!("{:.0}", max_value)),
                    ])
            );

        frame.render_widget(chart, inner);

        // Render current values at bottom
        let stats_area = Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1);
        let current_upload = metrics.network_upload.back().copied().unwrap_or(0);
        let current_download = metrics.network_download.back().copied().unwrap_or(0);

        let stats = Paragraph::new(format!(
            "⬆ {} MB/s    ⬇ {} MB/s",
            current_upload, current_download
        ))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

        frame.render_widget(stats, stats_area);
    }

    /// Render processes table
    fn render_processes(frame: &mut Frame, area: Rect, metrics: &MetricsState, selected: usize) {
        let rows: Vec<Row> = metrics.processes.iter().enumerate().map(|(idx, proc)| {
            let style = if idx == selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };

            Row::new(vec![
                if idx == selected { "▶" } else { " " }.to_string(),
                proc.pid.to_string(),
                proc.name.clone(),
                format!("{:.1}%", proc.cpu_percent),
                format!("{} MB", proc.memory_mb),
                proc.threads.to_string(),
                proc.status.as_str().to_string(),
            ])
            .style(style)
        }).collect();

        let widths = [
            Constraint::Length(2),   // Selection
            Constraint::Length(8),   // PID
            Constraint::Percentage(30),  // Name
            Constraint::Length(10),  // CPU
            Constraint::Length(12),  // Memory
            Constraint::Length(10),  // Threads
            Constraint::Percentage(20),  // Status
        ];

        let table = Table::new(rows, widths)
            .header(
                Row::new(vec!["", "PID", "Name", "CPU%", "Memory", "Threads", "Status"])
                    .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            )
            .block(
                Block::bordered()
                    .title("Active Processes")
                    .border_style(Style::default().fg(Color::Cyan))
            );

        frame.render_widget(table, area);
    }

    /// Render quick actions panel
    fn render_quick_actions(frame: &mut Frame, area: Rect) {
        let actions = vec![
            Line::from(vec![
                Span::styled("[1]", Style::default().fg(Color::Yellow)),
                Span::raw(" Settings  "),
                Span::styled("[2]", Style::default().fg(Color::Yellow)),
                Span::raw(" Data View  "),
                Span::styled("[3]", Style::default().fg(Color::Yellow)),
                Span::raw(" Help  "),
                Span::styled("[R]", Style::default().fg(Color::Yellow)),
                Span::raw(" Refresh  "),
                Span::styled("[Q]", Style::default().fg(Color::Red)),
                Span::raw(" Quit"),
            ])
        ];

        let actions_widget = Paragraph::new(actions)
            .block(
                Block::bordered()
                    .title("Quick Actions")
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .alignment(Alignment::Center);

        frame.render_widget(actions_widget, area);
    }

    /// Render footer with status info
    fn render_footer(frame: &mut Frame, area: Rect, metrics: &MetricsState) {
        let elapsed = metrics.time_since_update();
        let status_text = format!(
            "Status: Connected | Updated: {}s ago | FPS: 60 | Memory: {} MB",
            elapsed.as_secs(),
            (metrics.current_memory() * 1024.0) as u64
        );

        let footer = Paragraph::new(status_text)
            .style(Style::default().fg(Color::Gray))
            .block(
                Block::bordered()
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .alignment(Alignment::Left);

        frame.render_widget(footer, area);
    }
}

impl Default for DashboardScreen {
    fn default() -> Self {
        Self::new()
    }
}
