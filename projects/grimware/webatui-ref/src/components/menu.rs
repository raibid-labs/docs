//! Menu component
//!
//! Displays a selectable list of menu items with highlighting.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Widget},
};

/// Menu item data structure
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Display text for the menu item
    pub label: String,

    /// Optional description
    pub description: Option<String>,

    /// Whether the item is enabled
    pub enabled: bool,
}

impl MenuItem {
    /// Create a new menu item
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            enabled: true,
        }
    }

    /// Create a new menu item with description
    pub fn with_description(label: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: Some(description.into()),
            enabled: true,
        }
    }

    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Menu widget displaying a selectable list
pub struct Menu<'a> {
    items: &'a [String],
    selected_index: usize,
}

impl<'a> Menu<'a> {
    /// Create a new menu with items and selected index
    pub fn new(items: &'a [String], selected_index: usize) -> Self {
        Self {
            items,
            selected_index,
        }
    }
}

impl Widget for Menu<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create the menu block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White))
            .title("Navigation")
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black));

        // Create list items
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let is_selected = i == self.selected_index;

                // Build the line with prefix and styling
                let prefix = if is_selected { "▶ " } else { "  " };
                let content = format!("{}{}", prefix, item);

                let style = if is_selected {
                    Style::default()
                        .fg(Color::Yellow)
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                ListItem::new(Line::from(content)).style(style)
            })
            .collect();

        // Create and render the list
        let list = List::new(items)
            .block(block)
            .style(Style::default())
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        list.render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_creation() {
        let item = MenuItem::new("Test Item");
        assert_eq!(item.label, "Test Item");
        assert!(item.enabled);
        assert!(item.description.is_none());
    }

    #[test]
    fn test_menu_item_with_description() {
        let item = MenuItem::with_description("Test", "Description");
        assert_eq!(item.label, "Test");
        assert_eq!(item.description.as_deref(), Some("Description"));
    }

    #[test]
    fn test_menu_item_disabled() {
        let item = MenuItem::new("Test").enabled(false);
        assert!(!item.enabled);
    }

    #[test]
    fn test_menu_creation() {
        let items = vec!["Item 1".to_string(), "Item 2".to_string()];
        let menu = Menu::new(&items, 0);
        assert_eq!(menu.items.len(), 2);
        assert_eq!(menu.selected_index, 0);
    }
}
