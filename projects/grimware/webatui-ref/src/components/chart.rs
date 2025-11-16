//! Chart component
//!
//! Simple chart components using ratatui widgets for data visualization

use crate::prelude::*;
use std::collections::VecDeque;

/// Chart widget for displaying time-series data
pub struct ChartWidget {
    /// Chart title
    title: String,
    /// Data points
    data: Vec<(f64, f64)>,
    /// Chart style
    style: ChartStyle,
    /// X-axis bounds
    x_bounds: [f64; 2],
    /// Y-axis bounds
    y_bounds: [f64; 2],
    /// X-axis label
    x_label: String,
    /// Y-axis label
    y_label: String,
}

/// Chart style options
#[derive(Debug, Clone, Copy)]
pub enum ChartStyle {
    /// Line chart
    Line,
    /// Sparkline (compact line chart)
    Sparkline,
}

impl ChartWidget {
    /// Create a new chart
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            data: Vec::new(),
            style: ChartStyle::Line,
            x_bounds: [0.0, 100.0],
            y_bounds: [0.0, 100.0],
            x_label: String::new(),
            y_label: String::new(),
        }
    }

    /// Set chart data
    pub fn data(mut self, data: Vec<(f64, f64)>) -> Self {
        self.data = data;
        self
    }

    /// Set chart style
    pub fn style(mut self, style: ChartStyle) -> Self {
        self.style = style;
        self
    }

    /// Set X-axis bounds
    pub fn x_bounds(mut self, bounds: [f64; 2]) -> Self {
        self.x_bounds = bounds;
        self
    }

    /// Set Y-axis bounds
    pub fn y_bounds(mut self, bounds: [f64; 2]) -> Self {
        self.y_bounds = bounds;
        self
    }

    /// Set axis labels
    pub fn labels(mut self, x_label: impl Into<String>, y_label: impl Into<String>) -> Self {
        self.x_label = x_label.into();
        self.y_label = y_label.into();
        self
    }

    /// Render the chart
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        match self.style {
            ChartStyle::Line => self.render_line_chart(frame, area),
            ChartStyle::Sparkline => self.render_sparkline(frame, area),
        }
    }

    /// Render as a line chart
    fn render_line_chart(&self, frame: &mut Frame, area: Rect) {
        let datasets = vec![
            Dataset::default()
                .name(self.title.as_str())
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Cyan))
                .graph_type(ratatui::widgets::GraphType::Line)
                .data(&self.data)
        ];

        let chart = Chart::new(datasets)
            .block(
                Block::bordered()
                    .title(self.title.clone())
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .x_axis(
                Axis::default()
                    .title(self.x_label.clone())
                    .style(Style::default().fg(Color::Gray))
                    .bounds(self.x_bounds)
            )
            .y_axis(
                Axis::default()
                    .title(self.y_label.clone())
                    .style(Style::default().fg(Color::Gray))
                    .bounds(self.y_bounds)
                    .labels(vec![
                        Line::from(format!("{:.0}", self.y_bounds[0])),
                        Line::from(format!("{:.0}", (self.y_bounds[0] + self.y_bounds[1]) / 2.0)),
                        Line::from(format!("{:.0}", self.y_bounds[1])),
                    ])
            );

        frame.render_widget(chart, area);
    }

    /// Render as a sparkline
    fn render_sparkline(&self, frame: &mut Frame, area: Rect) {
        let values: Vec<u64> = self.data.iter().map(|(_, y)| *y as u64).collect();

        let sparkline = Sparkline::default()
            .block(
                Block::bordered()
                    .title(self.title.clone())
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .data(&values)
            .style(Style::default().fg(Color::Cyan));

        frame.render_widget(sparkline, area);
    }
}

/// Network activity chart with upload/download sparklines
pub struct NetworkChart {
    /// Upload data
    upload: VecDeque<u64>,
    /// Download data
    download: VecDeque<u64>,
    /// Maximum data points to keep
    max_points: usize,
}

impl NetworkChart {
    /// Create a new network chart
    pub fn new(max_points: usize) -> Self {
        Self {
            upload: VecDeque::with_capacity(max_points),
            download: VecDeque::with_capacity(max_points),
            max_points,
        }
    }

    /// Add a data point
    pub fn add_data(&mut self, upload: u64, download: u64) {
        if self.upload.len() >= self.max_points {
            self.upload.pop_front();
        }
        if self.download.len() >= self.max_points {
            self.download.pop_front();
        }

        self.upload.push_back(upload);
        self.download.push_back(download);
    }

    /// Render the network chart
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Split area into main chart and stats
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70),
                Constraint::Percentage(30),
            ])
            .split(area);

        // Render chart with both datasets
        let download_data: Vec<(f64, f64)> = self.download
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v as f64))
            .collect();

        let upload_data: Vec<(f64, f64)> = self.upload
            .iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v as f64))
            .collect();

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

        let max_value = self.download.iter().chain(self.upload.iter())
            .max()
            .copied()
            .unwrap_or(100) as f64;

        let chart = Chart::new(datasets)
            .block(
                Block::bordered()
                    .title("Network Activity (MB/s)")
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .x_axis(
                Axis::default()
                    .title("Time (s)")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, self.max_points as f64])
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

        frame.render_widget(chart, chunks[0]);

        // Render current stats
        let current_upload = self.upload.back().copied().unwrap_or(0);
        let current_download = self.download.back().copied().unwrap_or(0);

        let stats = Paragraph::new(format!(
            " ⬆ {} MB/s    ⬇ {} MB/s",
            current_upload, current_download
        ))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center);

        frame.render_widget(stats, chunks[1]);
    }
}

/// Simple sparkline renderer
pub fn render_sparkline(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    data: &[u64],
    current_value: u64,
) {
    let block = Block::bordered()
        .title(title)
        .border_style(Style::default().fg(Color::Cyan));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render sparkline
    let sparkline = Sparkline::default()
        .data(data)
        .style(Style::default().fg(Color::Cyan));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
        ])
        .split(inner);

    frame.render_widget(sparkline, chunks[1]);

    // Render current value
    let value_text = Paragraph::new(format!("{} MB/s", current_value))
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Right);

    frame.render_widget(value_text, chunks[0]);
}
