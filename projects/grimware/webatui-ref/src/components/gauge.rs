//! Gauge component
//!
//! Displays progress/percentage as a visual gauge with customizable styling

use crate::prelude::*;

/// Gauge widget for displaying metrics as progress bars
pub struct GaugeWidget {
    /// Label for the gauge
    label: String,
    /// Current value (0.0 to 1.0)
    ratio: f64,
    /// Gauge style
    style: GaugeStyle,
}

/// Styling options for gauges
#[derive(Debug, Clone, Copy)]
pub enum GaugeStyle {
    /// Horizontal bar gauge
    Horizontal,
    /// Vertical bar gauge
    Vertical,
}

impl GaugeWidget {
    /// Create a new gauge with default styling
    pub fn new(label: impl Into<String>, ratio: f64) -> Self {
        Self {
            label: label.into(),
            ratio: ratio.clamp(0.0, 1.0),
            style: GaugeStyle::Horizontal,
        }
    }

    /// Set the gauge style
    pub fn style(mut self, style: GaugeStyle) -> Self {
        self.style = style;
        self
    }

    /// Update the gauge ratio
    pub fn set_ratio(&mut self, ratio: f64) {
        self.ratio = ratio.clamp(0.0, 1.0);
    }

    /// Render the gauge widget
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        match self.style {
            GaugeStyle::Horizontal => self.render_horizontal(frame, area),
            GaugeStyle::Vertical => self.render_vertical(frame, area),
        }
    }

    /// Render horizontal gauge
    fn render_horizontal(&self, frame: &mut Frame, area: Rect) {
        // Create gauge with label and percentage
        let percentage = (self.ratio * 100.0) as u16;
        let gauge = Gauge::default()
            .block(
                Block::bordered()
                    .title(self.label.clone())
                    .border_style(Style::default().fg(Color::Cyan))
            )
            .gauge_style(
                Style::default()
                    .fg(self.get_color_for_ratio())
                    .bg(Color::DarkGray)
            )
            .percent(percentage)
            .label(format!("{}%", percentage));

        frame.render_widget(gauge, area);
    }

    /// Render vertical gauge
    fn render_vertical(&self, frame: &mut Frame, area: Rect) {
        // For vertical gauges, we'll use a custom implementation
        let block = Block::bordered()
            .title(self.label.clone())
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Calculate filled height
        let filled_height = ((inner.height as f64) * self.ratio) as u16;
        let color = self.get_color_for_ratio();

        // Render filled portion from bottom up
        for y in 0..inner.height {
            let is_filled = y >= (inner.height - filled_height);
            let style = if is_filled {
                Style::default().fg(color).bg(color)
            } else {
                Style::default().fg(Color::DarkGray).bg(Color::DarkGray)
            };

            let bar = Line::from(vec![
                Span::styled("█".repeat(inner.width as usize), style)
            ]);

            frame.render_widget(
                Paragraph::new(bar),
                Rect::new(inner.x, inner.y + y, inner.width, 1)
            );
        }

        // Render percentage at bottom
        let percentage = (self.ratio * 100.0) as u16;
        let label = format!("{}%", percentage);
        let label_widget = Paragraph::new(label)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);

        frame.render_widget(
            label_widget,
            Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1)
        );
    }

    /// Get color based on ratio (red = low, yellow = medium, green = high)
    fn get_color_for_ratio(&self) -> Color {
        if self.ratio < 0.33 {
            Color::Green
        } else if self.ratio < 0.66 {
            Color::Yellow
        } else {
            Color::Red
        }
    }
}

/// Simple gauge render function for convenience
pub fn render_gauge(frame: &mut Frame, area: Rect, label: &str, ratio: f64) {
    GaugeWidget::new(label, ratio).render(frame, area);
}

/// Render multiple gauges in a grid layout
pub fn render_gauge_grid(
    frame: &mut Frame,
    area: Rect,
    gauges: &[(String, f64)],
    columns: u16,
) {
    let rows = (gauges.len() as u16 + columns - 1) / columns;

    // Create grid layout
    let row_constraints: Vec<Constraint> = vec![Constraint::Percentage(100 / rows); rows as usize];
    let rows_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(area);

    for (row_idx, row_area) in rows_layout.iter().enumerate() {
        let start_idx = row_idx * columns as usize;
        let end_idx = (start_idx + columns as usize).min(gauges.len());
        let row_gauges = &gauges[start_idx..end_idx];

        let col_constraints: Vec<Constraint> =
            vec![Constraint::Percentage(100 / columns); row_gauges.len()];

        let cols_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(*row_area);

        for (col_idx, col_area) in cols_layout.iter().enumerate() {
            if let Some((label, ratio)) = row_gauges.get(col_idx) {
                render_gauge(frame, *col_area, label, *ratio);
            }
        }
    }
}
