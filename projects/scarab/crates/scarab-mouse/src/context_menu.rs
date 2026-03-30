//! Context menu component for right-click actions

use crate::types::Position;
use serde::{Deserialize, Serialize};

/// Context menu definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextMenu {
    pub position: Position,
    pub items: Vec<MenuItem>,
    pub selected_index: usize,
}

/// Menu item with action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItem {
    pub id: String,
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub separator: bool,
}

impl MenuItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            shortcut: None,
            enabled: true,
            separator: false,
        }
    }

    pub fn with_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            shortcut: None,
            enabled: false,
            separator: true,
        }
    }
}

impl ContextMenu {
    /// Create a new context menu
    pub fn new(position: Position) -> Self {
        Self {
            position,
            items: Vec::new(),
            selected_index: 0,
        }
    }

    /// Add a menu item
    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    /// Get standard terminal context menu
    pub fn standard(position: Position, has_selection: bool) -> Self {
        let mut menu = Self::new(position);

        menu.add_item(
            MenuItem::new("copy", "Copy")
                .with_shortcut("Ctrl+Shift+C")
                .with_enabled(has_selection),
        );

        menu.add_item(MenuItem::new("paste", "Paste").with_shortcut("Ctrl+Shift+V"));

        menu.add_item(MenuItem::separator());

        menu.add_item(MenuItem::new("select_all", "Select All").with_shortcut("Ctrl+Shift+A"));

        menu.add_item(MenuItem::new("clear_selection", "Clear Selection").with_enabled(has_selection));

        menu.add_item(MenuItem::separator());

        menu.add_item(MenuItem::new("search", "Search").with_shortcut("Ctrl+Shift+F"));

        menu.add_item(MenuItem::separator());

        menu.add_item(MenuItem::new("new_tab", "New Tab").with_shortcut("Ctrl+Shift+T"));

        menu.add_item(MenuItem::new("split_horizontal", "Split Horizontal"));

        menu.add_item(MenuItem::new("split_vertical", "Split Vertical"));

        menu
    }

    /// Get URL context menu
    pub fn url_menu(position: Position, url: String) -> Self {
        let mut menu = Self::new(position);

        menu.add_item(MenuItem::new("open_url", format!("Open: {}", url)));

        menu.add_item(MenuItem::new("copy_url", "Copy URL"));

        menu.add_item(MenuItem::separator());

        menu.add_item(MenuItem::new("copy", "Copy").with_shortcut("Ctrl+Shift+C"));

        menu.add_item(MenuItem::new("paste", "Paste").with_shortcut("Ctrl+Shift+V"));

        menu
    }

    /// Get file path context menu
    pub fn file_menu(position: Position, path: String) -> Self {
        let mut menu = Self::new(position);

        menu.add_item(MenuItem::new("open_file", format!("Open: {}", path)));

        menu.add_item(MenuItem::new("copy_path", "Copy Path"));

        menu.add_item(MenuItem::separator());

        menu.add_item(MenuItem::new("copy", "Copy").with_shortcut("Ctrl+Shift+C"));

        menu.add_item(MenuItem::new("paste", "Paste").with_shortcut("Ctrl+Shift+V"));

        menu
    }

    /// Select next item
    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        loop {
            self.selected_index = (self.selected_index + 1) % self.items.len();
            if !self.items[self.selected_index].separator {
                break;
            }
        }
    }

    /// Select previous item
    pub fn select_prev(&mut self) {
        if self.items.is_empty() {
            return;
        }

        loop {
            if self.selected_index == 0 {
                self.selected_index = self.items.len() - 1;
            } else {
                self.selected_index -= 1;
            }

            if !self.items[self.selected_index].separator {
                break;
            }
        }
    }

    /// Get selected item
    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.items.get(self.selected_index)
    }

    /// Get item by ID
    pub fn get_item(&self, id: &str) -> Option<&MenuItem> {
        self.items.iter().find(|item| item.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_creation() {
        let menu = ContextMenu::standard(Position::new(10, 10), true);
        assert!(menu.items.len() > 0);
        assert!(menu.get_item("copy").unwrap().enabled);
    }

    #[test]
    fn test_menu_without_selection() {
        let menu = ContextMenu::standard(Position::new(10, 10), false);
        assert!(!menu.get_item("copy").unwrap().enabled);
        assert!(!menu.get_item("clear_selection").unwrap().enabled);
    }

    #[test]
    fn test_menu_navigation() {
        let mut menu = ContextMenu::standard(Position::new(10, 10), true);
        let initial = menu.selected_index;

        menu.select_next();
        assert_ne!(menu.selected_index, initial);

        menu.select_prev();
        // Should skip separators
        assert!(!menu.selected_item().unwrap().separator);
    }

    #[test]
    fn test_url_menu() {
        let menu = ContextMenu::url_menu(
            Position::new(5, 5),
            "https://example.com".to_string(),
        );

        assert!(menu.get_item("open_url").is_some());
        assert!(menu.get_item("copy_url").is_some());
    }

    #[test]
    fn test_separator() {
        let sep = MenuItem::separator();
        assert!(sep.separator);
        assert!(!sep.enabled);
    }
}
