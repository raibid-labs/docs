//! Header component
//!
//! Displays the application title and current screen name.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::state::Screen;

/// Header component displaying title and navigation
pub struct Header<'a> {
    title: &'a str,
    current_screen: Screen,
}

impl<'a> Header<'a> {
    /// Create a new header with title and current screen
    pub fn new(title: &'a str, current_screen: Screen) -> Self {
        Self {
            title,
            current_screen,
        }
    }
}

impl Widget for Header<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create the header block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black));

        // Create title and navigation text
        let title_span = Span::styled(
            self.title,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

        let separator = Span::raw(" | ");

        let screen_span = Span::styled(
            self.current_screen.name(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::ITALIC),
        );

        let shortcuts = Span::styled(
            " [1-4: Screens] [?: Help] [q: Quit]",
            Style::default().fg(Color::DarkGray),
        );

        // Combine all spans into a line
        let header_line = Line::from(vec![
            Span::raw(" "),
            title_span,
            separator,
            screen_span,
            Span::raw(" "),
            shortcuts,
        ]);

        // Create paragraph and render
        let paragraph = Paragraph::new(header_line)
            .block(block)
            .style(Style::default());

        paragraph.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_creation() {
        let header = Header::new("Test App", Screen::Dashboard);
        assert_eq!(header.title, "Test App");
        assert_eq!(header.current_screen, Screen::Dashboard);
    }
}
