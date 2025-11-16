//! Footer component
//!
//! Displays a status bar with key hints and contextual information.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::state::Screen;

/// Footer component displaying status and key hints
pub struct Footer {
    current_screen: Screen,
}

impl Footer {
    /// Create a new footer for the current screen
    pub fn new(current_screen: Screen) -> Self {
        Self { current_screen }
    }

    /// Get key hints based on current screen
    fn get_key_hints(&self) -> Vec<(&'static str, &'static str)> {
        match self.current_screen {
            Screen::Home => vec![
                ("↑/↓", "Navigate"),
                ("Enter", "Select"),
            ],
            Screen::Dashboard => vec![
                ("↑/↓", "Navigate"),
                ("r", "Refresh"),
            ],
            Screen::Interactive => vec![
                ("+/-", "Counter"),
                ("a", "Add"),
                ("d", "Delete"),
            ],
            Screen::Settings => vec![
                ("Tab", "Next"),
                ("Esc", "Back"),
                ("Enter", "Select"),
            ],
        }
    }
}

impl Widget for Footer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create the footer block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        // Build key hints
        let mut spans = Vec::new();

        // Add status indicator
        spans.push(Span::styled(
            " ● ",
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "Ready",
            Style::default().fg(Color::Green),
        ));
        spans.push(Span::raw(" | "));

        // Add key hints for current screen
        let hints = self.get_key_hints();
        for (i, (key, action)) in hints.iter().enumerate() {
            if i > 0 {
                spans.push(Span::raw(" "));
            }

            // Key in bold cyan
            spans.push(Span::styled(
                *key,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ));

            // Separator
            spans.push(Span::raw(": "));

            // Action in white
            spans.push(Span::styled(
                *action,
                Style::default().fg(Color::White),
            ));

            // Separator between hints
            if i < hints.len() - 1 {
                spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
            }
        }

        // Add common hints
        spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(
            "?",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(": Help "));

        spans.push(Span::styled(
            "q",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::raw(": Quit "));

        let footer_line = Line::from(spans);

        // Create paragraph and render
        let paragraph = Paragraph::new(footer_line)
            .block(block)
            .style(Style::default());

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_footer_creation() {
        let footer = Footer::new(Screen::Dashboard);
        assert_eq!(footer.current_screen, Screen::Dashboard);
    }

    #[test]
    fn test_key_hints_per_screen() {
        let dashboard_footer = Footer::new(Screen::Dashboard);
        let hints = dashboard_footer.get_key_hints();
        assert!(!hints.is_empty());

        let settings_footer = Footer::new(Screen::Settings);
        let settings_hints = settings_footer.get_key_hints();
        assert!(!settings_hints.is_empty());
    }
}
