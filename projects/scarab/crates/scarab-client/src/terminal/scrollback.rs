// Scrollback buffer management for terminal history
// Provides efficient storage and retrieval of historical terminal lines

use bevy::prelude::*;
use scarab_protocol::Cell;
use std::collections::VecDeque;
use std::time::SystemTime;

/// Maximum number of lines to keep in scrollback (10,000 default)
pub const DEFAULT_MAX_SCROLLBACK_LINES: usize = 10_000;

/// A single line in the scrollback buffer
#[derive(Clone)]
pub struct ScrollbackLine {
    /// Cell data for this line
    pub cells: Vec<Cell>,
    /// When this line was added to scrollback
    pub timestamp: SystemTime,
    /// Whether this line has been wrapped from previous line
    pub wrapped: bool,
}

impl ScrollbackLine {
    /// Create a new scrollback line from cells
    pub fn new(cells: Vec<Cell>) -> Self {
        Self {
            cells,
            timestamp: SystemTime::now(),
            wrapped: false,
        }
    }

    /// Create a new wrapped scrollback line
    pub fn new_wrapped(cells: Vec<Cell>) -> Self {
        Self {
            cells,
            timestamp: SystemTime::now(),
            wrapped: true,
        }
    }

    /// Get text content of this line
    pub fn to_string(&self) -> String {
        let mut text = String::with_capacity(self.cells.len());
        for cell in &self.cells {
            if cell.char_codepoint == 0 || cell.char_codepoint == 32 {
                text.push(' ');
            } else if let Some(ch) = char::from_u32(cell.char_codepoint) {
                text.push(ch);
            } else {
                text.push('?');
            }
        }
        text
    }

    /// Check if line matches a search pattern
    pub fn matches(&self, pattern: &str, case_sensitive: bool) -> bool {
        let text = self.to_string();
        if case_sensitive {
            text.contains(pattern)
        } else {
            text.to_lowercase().contains(&pattern.to_lowercase())
        }
    }

    /// Check if line matches a regex pattern
    pub fn matches_regex(&self, regex: &regex::Regex) -> bool {
        let text = self.to_string();
        regex.is_match(&text)
    }
}

/// Scrollback buffer resource managing terminal history
#[derive(Resource)]
pub struct ScrollbackBuffer {
    /// Lines stored in scrollback (separate from visible grid)
    lines: VecDeque<ScrollbackLine>,
    /// Maximum lines to keep (default: 10,000)
    max_lines: usize,
    /// Current scroll position (0 = bottom/latest, positive = scrolled up)
    scroll_offset: usize,
    /// Cached search results (line indices that match current search)
    search_results: Vec<usize>,
    /// Current search query
    search_query: Option<String>,
    /// Current search index (which result we're viewing)
    search_index: usize,
    /// Whether search is case-sensitive
    search_case_sensitive: bool,
    /// Whether search uses regex
    search_use_regex: bool,
}

impl Default for ScrollbackBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_SCROLLBACK_LINES)
    }
}

impl ScrollbackBuffer {
    /// Create a new scrollback buffer with specified capacity
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_lines.min(1024)), // Don't pre-allocate all
            max_lines,
            scroll_offset: 0,
            search_results: Vec::new(),
            search_query: None,
            search_index: 0,
            search_case_sensitive: false,
            search_use_regex: false,
        }
    }

    /// Push a new line to scrollback (LRU eviction if full)
    pub fn push_line(&mut self, line: ScrollbackLine) {
        // Evict oldest line if at capacity
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
            // Adjust scroll offset to maintain view position
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
        }

        self.lines.push_back(line);

        // Invalidate search results if they exist
        if self.search_query.is_some() {
            self.invalidate_search();
        }
    }

    /// Push multiple lines at once (more efficient)
    pub fn push_lines(&mut self, lines: Vec<ScrollbackLine>) {
        for line in lines {
            self.push_line(line);
        }
    }

    /// Get current scroll offset (0 = bottom)
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Get total number of lines in scrollback
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Scroll up by N lines
    pub fn scroll_up(&mut self, lines: usize) {
        let max_scroll = self.lines.len();
        self.scroll_offset = (self.scroll_offset + lines).min(max_scroll);
    }

    /// Scroll down by N lines
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
    }

    /// Scroll to top of scrollback
    pub fn scroll_to_top(&mut self) {
        self.scroll_offset = self.lines.len();
    }

    /// Scroll to bottom (live view)
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    /// Check if currently at bottom (live view)
    pub fn is_at_bottom(&self) -> bool {
        self.scroll_offset == 0
    }

    /// Get line at specific index (0 = oldest, len-1 = newest)
    pub fn get_line(&self, index: usize) -> Option<&ScrollbackLine> {
        self.lines.get(index)
    }

    /// Get lines in view range (returns lines to display based on scroll offset)
    pub fn get_visible_lines(&self, viewport_height: usize) -> Vec<&ScrollbackLine> {
        let total_lines = self.lines.len();
        if total_lines == 0 || self.scroll_offset == 0 {
            return Vec::new();
        }

        // Calculate which lines should be visible
        let start_idx = total_lines.saturating_sub(self.scroll_offset);
        let end_idx = (start_idx + viewport_height).min(total_lines);

        self.lines
            .range(start_idx..end_idx)
            .collect()
    }

    /// Search through scrollback buffer
    pub fn search(&mut self, query: String, case_sensitive: bool, use_regex: bool) {
        self.search_query = Some(query.clone());
        self.search_case_sensitive = case_sensitive;
        self.search_use_regex = use_regex;
        self.search_results.clear();
        self.search_index = 0;

        if use_regex {
            // Try to compile regex
            match regex::Regex::new(&query) {
                Ok(regex) => {
                    for (idx, line) in self.lines.iter().enumerate() {
                        if line.matches_regex(&regex) {
                            self.search_results.push(idx);
                        }
                    }
                }
                Err(e) => {
                    warn!("Invalid regex pattern '{}': {}", query, e);
                }
            }
        } else {
            // Simple text search
            for (idx, line) in self.lines.iter().enumerate() {
                if line.matches(&query, case_sensitive) {
                    self.search_results.push(idx);
                }
            }
        }

        // Jump to first result if any
        if !self.search_results.is_empty() {
            self.jump_to_search_result(0);
        }
    }

    /// Jump to specific search result
    fn jump_to_search_result(&mut self, result_index: usize) {
        if result_index >= self.search_results.len() {
            return;
        }

        self.search_index = result_index;
        let line_idx = self.search_results[result_index];

        // Scroll to this line (center it in viewport if possible)
        self.scroll_offset = self.lines.len().saturating_sub(line_idx);
    }

    /// Jump to next search result
    pub fn next_search_result(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        self.search_index = (self.search_index + 1) % self.search_results.len();
        self.jump_to_search_result(self.search_index);
    }

    /// Jump to previous search result
    pub fn prev_search_result(&mut self) {
        if self.search_results.is_empty() {
            return;
        }

        self.search_index = if self.search_index == 0 {
            self.search_results.len() - 1
        } else {
            self.search_index - 1
        };
        self.jump_to_search_result(self.search_index);
    }

    /// Clear search results
    pub fn clear_search(&mut self) {
        self.search_query = None;
        self.search_results.clear();
        self.search_index = 0;
    }

    /// Invalidate search results (called when buffer changes)
    fn invalidate_search(&mut self) {
        if let Some(query) = self.search_query.clone() {
            self.search(query, self.search_case_sensitive, self.search_use_regex);
        }
    }

    /// Get current search state
    pub fn search_state(&self) -> Option<SearchState> {
        self.search_query.as_ref().map(|query| SearchState {
            query: query.clone(),
            current_index: self.search_index,
            total_results: self.search_results.len(),
            case_sensitive: self.search_case_sensitive,
            use_regex: self.search_use_regex,
        })
    }

    /// Clear all scrollback
    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll_offset = 0;
        self.clear_search();
    }
}

/// Search state information
#[derive(Clone, Debug)]
pub struct SearchState {
    pub query: String,
    pub current_index: usize,
    pub total_results: usize,
    pub case_sensitive: bool,
    pub use_regex: bool,
}

/// Resource tracking scrollback UI state
#[derive(Resource, Default)]
pub struct ScrollbackState {
    /// Whether scrollback is currently active (not at bottom)
    pub is_scrolled: bool,
    /// Whether search overlay is visible
    pub search_visible: bool,
    /// Current search input text
    pub search_input: String,
    /// Lines per page for PageUp/PageDown
    pub lines_per_page: usize,
}

impl ScrollbackState {
    pub fn new(lines_per_page: usize) -> Self {
        Self {
            is_scrolled: false,
            search_visible: false,
            search_input: String::new(),
            lines_per_page,
        }
    }
}

/// System to handle mouse wheel scrolling
fn handle_mouse_scroll(
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut scrollback: ResMut<ScrollbackBuffer>,
    mut state: ResMut<ScrollbackState>,
) {
    use bevy::input::mouse::MouseScrollUnit;

    for event in scroll_events.read() {
        let lines = match event.unit {
            MouseScrollUnit::Line => (event.y * 3.0) as i32, // 3 lines per scroll notch
            MouseScrollUnit::Pixel => (event.y / 20.0) as i32, // ~20 pixels per line
        };

        if lines > 0 {
            // Scroll up
            scrollback.scroll_up(lines as usize);
        } else if lines < 0 {
            // Scroll down
            scrollback.scroll_down((-lines) as usize);
        }

        // Update scroll state
        state.is_scrolled = !scrollback.is_at_bottom();
    }
}

/// System to handle keyboard navigation
fn handle_keyboard_scrolling(
    keys: Res<ButtonInput<KeyCode>>,
    mut scrollback: ResMut<ScrollbackBuffer>,
    mut state: ResMut<ScrollbackState>,
) {
    // Shift+PageUp: Scroll up one page
    if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
        if keys.just_pressed(KeyCode::PageUp) {
            scrollback.scroll_up(state.lines_per_page);
            state.is_scrolled = true;
        }

        // Shift+PageDown: Scroll down one page
        if keys.just_pressed(KeyCode::PageDown) {
            scrollback.scroll_down(state.lines_per_page);
            state.is_scrolled = !scrollback.is_at_bottom();
        }

        // Shift+Home: Jump to top
        if keys.just_pressed(KeyCode::Home) {
            scrollback.scroll_to_top();
            state.is_scrolled = true;
        }

        // Shift+End: Jump to bottom (live view)
        if keys.just_pressed(KeyCode::End) {
            scrollback.scroll_to_bottom();
            state.is_scrolled = false;
        }
    }

    // Ctrl+F: Open search
    if (keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight))
        && keys.just_pressed(KeyCode::KeyF)
    {
        state.search_visible = !state.search_visible;
        if !state.search_visible {
            scrollback.clear_search();
            state.search_input.clear();
        }
    }

    // Escape: Close search
    if keys.just_pressed(KeyCode::Escape) && state.search_visible {
        state.search_visible = false;
        scrollback.clear_search();
        state.search_input.clear();
    }
}

/// System to handle search navigation
fn handle_search_navigation(
    keys: Res<ButtonInput<KeyCode>>,
    mut scrollback: ResMut<ScrollbackBuffer>,
    state: Res<ScrollbackState>,
) {
    if !state.search_visible {
        return;
    }

    // Enter: Next result
    if keys.just_pressed(KeyCode::Enter) {
        if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            // Shift+Enter: Previous result
            scrollback.prev_search_result();
        } else {
            // Enter: Next result
            scrollback.next_search_result();
        }
    }
}

/// Bevy plugin for scrollback functionality
pub struct ScrollbackPlugin;

impl Plugin for ScrollbackPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.insert_resource(ScrollbackBuffer::default())
            .insert_resource(ScrollbackState::new(25)) // 25 lines per page default
            .add_systems(
                Update,
                (
                    handle_mouse_scroll,
                    handle_keyboard_scrolling,
                    handle_search_navigation,
                )
                    .chain(),
            );

        info!("Scrollback plugin initialized");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollback_push_and_eviction() {
        let mut buffer = ScrollbackBuffer::new(10);

        // Push 15 lines (should evict 5 oldest)
        for i in 0..15 {
            let cells = vec![Cell {
                char_codepoint: ('A' as u32) + i,
                fg: 0xFFFFFFFF,
                bg: 0x000000FF,
                flags: 0,
                _padding: [0; 3],
            }];
            buffer.push_line(ScrollbackLine::new(cells));
        }

        assert_eq!(buffer.line_count(), 10);
        // First line should be 'F' (index 5), not 'A'
        assert_eq!(buffer.get_line(0).unwrap().cells[0].char_codepoint, 'F' as u32);
    }

    #[test]
    fn test_scrollback_scroll_navigation() {
        let mut buffer = ScrollbackBuffer::new(100);

        // Push 50 lines
        for _i in 0..50 {
            let cells = vec![Cell::default()];
            buffer.push_line(ScrollbackLine::new(cells));
        }

        // Test scroll up
        buffer.scroll_up(10);
        assert_eq!(buffer.scroll_offset(), 10);

        // Test scroll down
        buffer.scroll_down(5);
        assert_eq!(buffer.scroll_offset(), 5);

        // Test scroll to top
        buffer.scroll_to_top();
        assert_eq!(buffer.scroll_offset(), 50);

        // Test scroll to bottom
        buffer.scroll_to_bottom();
        assert_eq!(buffer.scroll_offset(), 0);
        assert!(buffer.is_at_bottom());
    }

    #[test]
    fn test_scrollback_search() {
        let mut buffer = ScrollbackBuffer::new(100);

        // Push lines with searchable content
        let test_lines = vec!["Hello world", "Test line", "Hello again", "Another test"];

        for text in test_lines {
            let cells: Vec<Cell> = text
                .chars()
                .map(|c| Cell {
                    char_codepoint: c as u32,
                    fg: 0xFFFFFFFF,
                    bg: 0x000000FF,
                    flags: 0,
                    _padding: [0; 3],
                })
                .collect();
            buffer.push_line(ScrollbackLine::new(cells));
        }

        // Search for "Hello"
        buffer.search("Hello".to_string(), false, false);

        let state = buffer.search_state().unwrap();
        assert_eq!(state.total_results, 2);
        assert_eq!(state.current_index, 0);

        // Navigate to next result
        buffer.next_search_result();
        let state = buffer.search_state().unwrap();
        assert_eq!(state.current_index, 1);
    }

    #[test]
    fn test_scrollback_line_to_string() {
        let text = "Test line";
        let cells: Vec<Cell> = text
            .chars()
            .map(|c| Cell {
                char_codepoint: c as u32,
                fg: 0xFFFFFFFF,
                bg: 0x000000FF,
                flags: 0,
                _padding: [0; 3],
            })
            .collect();

        let line = ScrollbackLine::new(cells);
        assert_eq!(line.to_string(), text);
    }
}
