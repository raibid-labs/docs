//! # fusabi-tui-widgets
//!
//! Reusable TUI widgets for Fusabi applications built on Ratatui.
//!
//! This crate provides pre-built widgets commonly needed for information browser
//! applications like Scryforge:
//!
//! - [`StreamListWidget`] - Sidebar showing available streams
//! - [`ItemListWidget`] - Scrollable list of items with filtering
//! - [`PreviewWidget`] - Rich preview of selected item
//! - [`StatusBarWidget`] - Connection status, sync state, notifications
//! - [`OmnibarWidget`] - Command palette / quick search
//!
//! ## Design Philosophy
//!
//! These widgets are designed to be:
//! - Composable: Mix and match to build your layout
//! - Themeable: Colors and styles can be customized
//! - Accessible: Keyboard-first navigation
//! - Responsive: Adapt to terminal size

use fusabi_streams_core::{Item, Stream};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

// ============================================================================
// Theme / Styling
// ============================================================================

/// Theme configuration for widgets.
#[derive(Debug, Clone)]
pub struct Theme {
    pub background: Color,
    pub foreground: Color,
    pub border: Color,
    pub border_focused: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
    pub unread: Color,
    pub muted: Color,
    pub accent: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: Color::Reset,
            foreground: Color::Reset,
            border: Color::DarkGray,
            border_focused: Color::Cyan,
            selection_bg: Color::DarkGray,
            selection_fg: Color::White,
            unread: Color::Yellow,
            muted: Color::DarkGray,
            accent: Color::Cyan,
        }
    }
}

// ============================================================================
// Stream List Widget
// ============================================================================

/// Widget displaying a list of streams in a sidebar.
pub struct StreamListWidget<'a> {
    streams: &'a [Stream],
    selected: Option<usize>,
    focused: bool,
    theme: &'a Theme,
}

impl<'a> StreamListWidget<'a> {
    pub fn new(streams: &'a [Stream], selected: Option<usize>, theme: &'a Theme) -> Self {
        Self {
            streams,
            selected,
            focused: false,
            theme,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Render the widget to a frame at the given area.
    pub fn render(self, frame: &mut Frame, area: Rect) {
        let border_color = if self.focused {
            self.theme.border_focused
        } else {
            self.theme.border
        };

        let block = Block::default()
            .title(" Streams ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let items: Vec<ListItem> = self
            .streams
            .iter()
            .enumerate()
            .map(|(i, stream)| {
                let is_selected = self.selected == Some(i);
                let unread = stream.unread_count.unwrap_or(0);

                let mut line = vec![Span::raw(&stream.name)];
                if unread > 0 {
                    line.push(Span::styled(
                        format!(" ({unread})"),
                        Style::default().fg(self.theme.unread),
                    ));
                }

                let style = if is_selected {
                    Style::default()
                        .bg(self.theme.selection_bg)
                        .fg(self.theme.selection_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(line)).style(style)
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}

// ============================================================================
// Item List Widget
// ============================================================================

/// Widget displaying a list of items.
pub struct ItemListWidget<'a> {
    items: &'a [Item],
    selected: Option<usize>,
    focused: bool,
    theme: &'a Theme,
}

impl<'a> ItemListWidget<'a> {
    pub fn new(items: &'a [Item], selected: Option<usize>, theme: &'a Theme) -> Self {
        Self {
            items,
            selected,
            focused: false,
            theme,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn render(self, frame: &mut Frame, area: Rect) {
        let border_color = if self.focused {
            self.theme.border_focused
        } else {
            self.theme.border
        };

        let block = Block::default()
            .title(" Items ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = self.selected == Some(i);

                let mut spans = vec![];

                // Unread indicator
                if !item.is_read {
                    spans.push(Span::styled("● ", Style::default().fg(self.theme.unread)));
                }

                // Title
                spans.push(Span::raw(&item.title));

                // Author if present
                if let Some(ref author) = item.author {
                    spans.push(Span::styled(
                        format!(" - {}", author.name),
                        Style::default().fg(self.theme.muted),
                    ));
                }

                let style = if is_selected {
                    Style::default()
                        .bg(self.theme.selection_bg)
                        .fg(self.theme.selection_fg)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(spans)).style(style)
            })
            .collect();

        let list = List::new(items).block(block);
        frame.render_widget(list, area);
    }
}

// ============================================================================
// Preview Widget
// ============================================================================

/// Widget displaying a preview of the selected item.
pub struct PreviewWidget<'a> {
    item: Option<&'a Item>,
    focused: bool,
    theme: &'a Theme,
}

impl<'a> PreviewWidget<'a> {
    pub fn new(item: Option<&'a Item>, theme: &'a Theme) -> Self {
        Self {
            item,
            focused: false,
            theme,
        }
    }

    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn render(self, frame: &mut Frame, area: Rect) {
        let border_color = if self.focused {
            self.theme.border_focused
        } else {
            self.theme.border
        };

        let block = Block::default()
            .title(" Preview ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let content = match self.item {
            Some(item) => {
                let mut lines = vec![
                    Line::from(Span::styled(
                        &item.title,
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                ];

                if let Some(ref author) = item.author {
                    lines.push(Line::from(Span::styled(
                        format!("By: {}", author.name),
                        Style::default().fg(self.theme.muted),
                    )));
                }

                if let Some(published) = item.published {
                    lines.push(Line::from(Span::styled(
                        format!("Date: {}", published.format("%Y-%m-%d %H:%M")),
                        Style::default().fg(self.theme.muted),
                    )));
                }

                lines.push(Line::from(""));

                // Extract text content based on item type
                let body = extract_preview_text(&item.content);
                for line in body.lines() {
                    lines.push(Line::from(line.to_string()));
                }

                lines
            }
            None => vec![Line::from(Span::styled(
                "No item selected",
                Style::default().fg(self.theme.muted),
            ))],
        };

        let paragraph = Paragraph::new(content)
            .block(block)
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }
}

fn extract_preview_text(content: &fusabi_streams_core::ItemContent) -> String {
    use fusabi_streams_core::ItemContent::*;
    match content {
        Text(s) => s.clone(),
        Markdown(s) => s.clone(), // TODO: render markdown
        Html(s) => s.clone(),     // TODO: strip HTML tags
        Email {
            snippet, body_text, ..
        } => body_text.clone().unwrap_or_else(|| snippet.clone()),
        Article {
            summary,
            full_content,
        } => full_content
            .clone()
            .or_else(|| summary.clone())
            .unwrap_or_default(),
        Video { description, .. } => description.clone(),
        Track { album, artists, .. } => {
            let artist_str = artists.join(", ");
            match album {
                Some(a) => format!("{artist_str} - {a}"),
                None => artist_str,
            }
        }
        Task { body, .. } => body.clone().unwrap_or_default(),
        Event {
            description,
            location,
            ..
        } => {
            let desc = description.clone().unwrap_or_default();
            match location {
                Some(loc) => format!("{desc}\n\nLocation: {loc}"),
                None => desc,
            }
        }
        Bookmark { description } => description.clone().unwrap_or_default(),
        Generic { body } => body.clone().unwrap_or_default(),
    }
}

// ============================================================================
// Status Bar Widget
// ============================================================================

/// Widget displaying status information at the bottom.
pub struct StatusBarWidget<'a> {
    message: &'a str,
    provider_status: &'a str,
    theme: &'a Theme,
}

impl<'a> StatusBarWidget<'a> {
    pub fn new(message: &'a str, provider_status: &'a str, theme: &'a Theme) -> Self {
        Self {
            message,
            provider_status,
            theme,
        }
    }

    pub fn render(self, frame: &mut Frame, area: Rect) {
        let spans = vec![
            Span::styled(self.message, Style::default().fg(self.theme.foreground)),
            Span::raw(" | "),
            Span::styled(self.provider_status, Style::default().fg(self.theme.accent)),
        ];

        let paragraph =
            Paragraph::new(Line::from(spans)).style(Style::default().bg(self.theme.selection_bg));

        frame.render_widget(paragraph, area);
    }
}

// ============================================================================
// Omnibar Widget (TODO)
// ============================================================================

/// Widget for command palette / quick search.
///
/// TODO: Implement full omnibar functionality:
/// - Text input for search/commands
/// - Autocomplete suggestions
/// - Command history
/// - Fuzzy matching
pub struct OmnibarWidget<'a> {
    input: &'a str,
    placeholder: &'a str,
    active: bool,
    theme: &'a Theme,
}

impl<'a> OmnibarWidget<'a> {
    pub fn new(input: &'a str, theme: &'a Theme) -> Self {
        Self {
            input,
            placeholder: "Type to search or press : for commands...",
            active: false,
            theme,
        }
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.placeholder = placeholder;
        self
    }

    pub fn render(self, frame: &mut Frame, area: Rect) {
        let border_color = if self.active {
            self.theme.border_focused
        } else {
            self.theme.border
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let display_text = if self.input.is_empty() {
            Span::styled(self.placeholder, Style::default().fg(self.theme.muted))
        } else {
            Span::raw(self.input)
        };

        let paragraph = Paragraph::new(Line::from(display_text)).block(block);
        frame.render_widget(paragraph, area);
    }
}

// ============================================================================
// Re-exports
// ============================================================================

pub mod prelude {
    pub use crate::{
        ItemListWidget, OmnibarWidget, PreviewWidget, StatusBarWidget, StreamListWidget, Theme,
    };
}
