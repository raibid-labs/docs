//! Text selection handling

use crate::types::Position;
use serde::{Deserialize, Serialize};

/// A text selection in the terminal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    pub start: Position,
    pub end: Position,
    pub kind: SelectionKind,
}

/// Type of selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionKind {
    /// Character-by-character selection
    Normal,
    /// Word-based selection
    Word,
    /// Line-based selection
    Line,
    /// Block/rectangular selection
    Block,
}

impl Selection {
    /// Create a new selection
    pub fn new(start: Position, end: Position, kind: SelectionKind) -> Self {
        Self { start, end, kind }
    }

    /// Create a character selection
    pub fn character(start: Position, end: Position) -> Self {
        Self::new(start, end, SelectionKind::Normal)
    }

    /// Create a word selection
    pub fn word(start: Position, end: Position) -> Self {
        Self::new(start, end, SelectionKind::Word)
    }

    /// Create a line selection
    pub fn line(start_y: u16, end_y: u16, cols: u16) -> Self {
        Self::new(
            Position::new(0, start_y),
            Position::new(cols.saturating_sub(1), end_y),
            SelectionKind::Line,
        )
    }

    /// Get normalized start and end (start is always before end)
    pub fn normalized(&self) -> (Position, Position) {
        if self.start.y < self.end.y || (self.start.y == self.end.y && self.start.x <= self.end.x) {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Check if a position is within this selection
    pub fn contains(&self, pos: Position) -> bool {
        let (start, end) = self.normalized();

        match self.kind {
            SelectionKind::Block => {
                // Rectangular selection
                let min_x = start.x.min(end.x);
                let max_x = start.x.max(end.x);
                let min_y = start.y.min(end.y);
                let max_y = start.y.max(end.y);

                pos.x >= min_x && pos.x <= max_x && pos.y >= min_y && pos.y <= max_y
            }
            _ => {
                // Linear selection
                if pos.y < start.y || pos.y > end.y {
                    return false;
                }

                if pos.y == start.y && pos.x < start.x {
                    return false;
                }

                if pos.y == end.y && pos.x > end.x {
                    return false;
                }

                true
            }
        }
    }

    /// Update selection end position (for drag operations)
    pub fn update_end(&mut self, new_end: Position) {
        self.end = new_end;
    }

    /// Expand selection to word boundaries
    pub fn expand_to_word(&mut self, get_char: impl Fn(Position) -> Option<char>) {
        let (start, end) = self.normalized();

        // Expand start backwards
        let mut new_start = start;
        while new_start.x > 0 {
            if let Some(ch) = get_char(Position::new(new_start.x - 1, new_start.y)) {
                if !is_word_char(ch) {
                    break;
                }
                new_start.x -= 1;
            } else {
                break;
            }
        }

        // Expand end forwards
        let mut new_end = end;
        loop {
            if let Some(ch) = get_char(Position::new(new_end.x + 1, new_end.y)) {
                if !is_word_char(ch) {
                    break;
                }
                new_end.x += 1;
            } else {
                break;
            }
        }

        self.start = new_start;
        self.end = new_end;
        self.kind = SelectionKind::Word;
    }

    /// Get the selected text from the terminal grid
    pub fn get_text(&self, get_char: impl Fn(Position) -> Option<char>, cols: u16) -> String {
        let (start, end) = self.normalized();
        let mut text = String::new();

        match self.kind {
            SelectionKind::Block => {
                // Rectangular selection - each row is a separate line
                let min_x = start.x.min(end.x);
                let max_x = start.x.max(end.x);

                for y in start.y..=end.y {
                    if y > start.y {
                        text.push('\n');
                    }
                    for x in min_x..=max_x {
                        if let Some(ch) = get_char(Position::new(x, y)) {
                            text.push(ch);
                        }
                    }
                }
            }
            _ => {
                // Linear selection
                for y in start.y..=end.y {
                    let line_start = if y == start.y { start.x } else { 0 };
                    let line_end = if y == end.y { end.x } else { cols - 1 };

                    for x in line_start..=line_end {
                        if let Some(ch) = get_char(Position::new(x, y)) {
                            text.push(ch);
                        }
                    }

                    // Add newline between lines (but not after last line)
                    if y < end.y {
                        text.push('\n');
                    }
                }
            }
        }

        text
    }
}

/// Check if a character is part of a word
fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '-'
}

/// Find word boundaries at a position
pub fn find_word_at(pos: Position, get_char: impl Fn(Position) -> Option<char>) -> Option<(Position, Position)> {
    // Check if position has a word character
    let ch = get_char(pos)?;
    if !is_word_char(ch) {
        return None;
    }

    // Find start of word
    let mut start = pos;
    while start.x > 0 {
        if let Some(ch) = get_char(Position::new(start.x - 1, start.y)) {
            if !is_word_char(ch) {
                break;
            }
            start.x -= 1;
        } else {
            break;
        }
    }

    // Find end of word
    let mut end = pos;
    loop {
        if let Some(ch) = get_char(Position::new(end.x + 1, end.y)) {
            if !is_word_char(ch) {
                break;
            }
            end.x += 1;
        } else {
            break;
        }
    }

    Some((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_normalization() {
        let sel = Selection::character(Position::new(5, 2), Position::new(3, 1));
        let (start, end) = sel.normalized();
        assert_eq!(start, Position::new(3, 1));
        assert_eq!(end, Position::new(5, 2));
    }

    #[test]
    fn test_selection_contains() {
        let sel = Selection::character(Position::new(2, 1), Position::new(5, 3));

        assert!(!sel.contains(Position::new(1, 1)));
        assert!(sel.contains(Position::new(2, 1)));
        assert!(sel.contains(Position::new(4, 2)));
        assert!(sel.contains(Position::new(5, 3)));
        assert!(!sel.contains(Position::new(6, 3)));
    }

    #[test]
    fn test_block_selection_contains() {
        let sel = Selection::new(
            Position::new(2, 1),
            Position::new(5, 3),
            SelectionKind::Block,
        );

        // Inside rectangle
        assert!(sel.contains(Position::new(3, 2)));

        // Outside rectangle
        assert!(!sel.contains(Position::new(1, 2)));
        assert!(!sel.contains(Position::new(6, 2)));
        assert!(!sel.contains(Position::new(3, 0)));
        assert!(!sel.contains(Position::new(3, 4)));
    }

    #[test]
    fn test_word_char() {
        assert!(is_word_char('a'));
        assert!(is_word_char('Z'));
        assert!(is_word_char('0'));
        assert!(is_word_char('_'));
        assert!(is_word_char('-'));
        assert!(!is_word_char(' '));
        assert!(!is_word_char('.'));
        assert!(!is_word_char('/'));
    }

    #[test]
    fn test_line_selection() {
        let sel = Selection::line(1, 3, 80);
        assert_eq!(sel.start, Position::new(0, 1));
        assert_eq!(sel.end, Position::new(79, 3));
        assert_eq!(sel.kind, SelectionKind::Line);
    }
}
