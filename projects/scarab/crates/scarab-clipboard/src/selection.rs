//! Text selection management for terminal

/// Selection mode for terminal text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Character-wise selection (click & drag)
    Character,
    /// Word selection (double-click or word-wise expansion)
    Word,
    /// Line selection (triple-click or line-wise)
    Line,
    /// Block/rectangular selection (Alt+drag)
    Block,
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::Character
    }
}

/// Selection region in terminal grid coordinates
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectionRegion {
    pub start_x: u16,
    pub start_y: u16,
    pub end_x: u16,
    pub end_y: u16,
}

impl SelectionRegion {
    /// Create a new selection region
    pub fn new(start_x: u16, start_y: u16, end_x: u16, end_y: u16) -> Self {
        Self {
            start_x,
            start_y,
            end_x,
            end_y,
        }
    }

    /// Check if a point is within the selection region
    pub fn contains(&self, x: u16, y: u16) -> bool {
        let (min_x, max_x) = if self.start_x <= self.end_x {
            (self.start_x, self.end_x)
        } else {
            (self.end_x, self.start_x)
        };

        let (min_y, max_y) = if self.start_y <= self.end_y {
            (self.start_y, self.end_y)
        } else {
            (self.end_y, self.start_y)
        };

        x >= min_x && x <= max_x && y >= min_y && y <= max_y
    }

    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.start_x == self.end_x && self.start_y == self.end_y
    }

    /// Normalize region so start comes before end
    pub fn normalize(&mut self) {
        if self.start_y > self.end_y || (self.start_y == self.end_y && self.start_x > self.end_x) {
            std::mem::swap(&mut self.start_x, &mut self.end_x);
            std::mem::swap(&mut self.start_y, &mut self.end_y);
        }
    }

    /// Get normalized copy of the region
    pub fn normalized(&self) -> Self {
        let mut region = self.clone();
        region.normalize();
        region
    }

    /// Get width of selection (in columns)
    pub fn width(&self) -> u16 {
        if self.start_x <= self.end_x {
            self.end_x - self.start_x + 1
        } else {
            self.start_x - self.end_x + 1
        }
    }

    /// Get height of selection (in rows)
    pub fn height(&self) -> u16 {
        if self.start_y <= self.end_y {
            self.end_y - self.start_y + 1
        } else {
            self.start_y - self.end_y + 1
        }
    }

    /// Expand selection to include a point
    pub fn expand_to(&mut self, x: u16, y: u16) {
        self.end_x = x;
        self.end_y = y;
    }
}

/// Selection state manager
#[derive(Debug, Default)]
pub struct SelectionState {
    pub active: bool,
    pub mode: SelectionMode,
    pub region: SelectionRegion,
}

impl SelectionState {
    /// Create a new selection state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new selection
    pub fn start(&mut self, x: u16, y: u16, mode: SelectionMode) {
        self.active = true;
        self.mode = mode;
        self.region = SelectionRegion::new(x, y, x, y);
    }

    /// Update selection endpoint
    pub fn update(&mut self, x: u16, y: u16) {
        if !self.active {
            return;
        }

        self.region.end_x = x;
        self.region.end_y = y;
    }

    /// Clear selection
    pub fn clear(&mut self) {
        self.active = false;
        self.region = SelectionRegion::default();
    }

    /// Check if selection is active and non-empty
    pub fn has_selection(&self) -> bool {
        self.active && !self.region.is_empty()
    }

    /// Get normalized region
    pub fn normalized_region(&self) -> SelectionRegion {
        self.region.normalized()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_region_new() {
        let region = SelectionRegion::new(5, 10, 15, 20);
        assert_eq!(region.start_x, 5);
        assert_eq!(region.start_y, 10);
        assert_eq!(region.end_x, 15);
        assert_eq!(region.end_y, 20);
    }

    #[test]
    fn test_selection_region_contains() {
        let region = SelectionRegion::new(5, 5, 10, 10);

        // Points inside
        assert!(region.contains(5, 5));
        assert!(region.contains(10, 10));
        assert!(region.contains(7, 7));

        // Points outside
        assert!(!region.contains(4, 5));
        assert!(!region.contains(11, 10));
        assert!(!region.contains(5, 11));
    }

    #[test]
    fn test_selection_region_is_empty() {
        let region1 = SelectionRegion::new(5, 5, 5, 5);
        assert!(region1.is_empty());

        let region2 = SelectionRegion::new(5, 5, 10, 10);
        assert!(!region2.is_empty());
    }

    #[test]
    fn test_selection_region_normalize() {
        let mut region = SelectionRegion::new(10, 10, 5, 5);
        region.normalize();

        assert_eq!(region.start_x, 5);
        assert_eq!(region.start_y, 5);
        assert_eq!(region.end_x, 10);
        assert_eq!(region.end_y, 10);
    }

    #[test]
    fn test_selection_region_normalize_same_row() {
        let mut region = SelectionRegion::new(10, 5, 5, 5);
        region.normalize();

        assert_eq!(region.start_x, 5);
        assert_eq!(region.start_y, 5);
        assert_eq!(region.end_x, 10);
        assert_eq!(region.end_y, 5);
    }

    #[test]
    fn test_selection_region_width_height() {
        let region = SelectionRegion::new(5, 5, 10, 10);

        assert_eq!(region.width(), 6); // 10 - 5 + 1
        assert_eq!(region.height(), 6); // 10 - 5 + 1
    }

    #[test]
    fn test_selection_region_expand_to() {
        let mut region = SelectionRegion::new(5, 5, 5, 5);

        region.expand_to(10, 10);

        assert_eq!(region.start_x, 5);
        assert_eq!(region.start_y, 5);
        assert_eq!(region.end_x, 10);
        assert_eq!(region.end_y, 10);
    }

    #[test]
    fn test_selection_state_start() {
        let mut state = SelectionState::new();

        assert!(!state.active);

        state.start(5, 5, SelectionMode::Character);

        assert!(state.active);
        assert_eq!(state.mode, SelectionMode::Character);
        assert_eq!(state.region.start_x, 5);
        assert_eq!(state.region.start_y, 5);
        assert_eq!(state.region.end_x, 5);
        assert_eq!(state.region.end_y, 5);
    }

    #[test]
    fn test_selection_state_update() {
        let mut state = SelectionState::new();

        state.start(5, 5, SelectionMode::Character);
        state.update(10, 10);

        assert_eq!(state.region.end_x, 10);
        assert_eq!(state.region.end_y, 10);
    }

    #[test]
    fn test_selection_state_clear() {
        let mut state = SelectionState::new();

        state.start(5, 5, SelectionMode::Character);
        assert!(state.active);

        state.clear();

        assert!(!state.active);
        assert!(state.region.is_empty());
    }

    #[test]
    fn test_selection_state_has_selection() {
        let mut state = SelectionState::new();

        assert!(!state.has_selection());

        state.start(5, 5, SelectionMode::Character);
        assert!(!state.has_selection()); // Still at start point

        state.update(10, 10);
        assert!(state.has_selection()); // Now has actual selection
    }

    #[test]
    fn test_selection_modes() {
        let modes = [
            SelectionMode::Character,
            SelectionMode::Word,
            SelectionMode::Line,
            SelectionMode::Block,
        ];

        for mode in modes {
            let mut state = SelectionState::new();
            state.start(0, 0, mode);
            assert_eq!(state.mode, mode);
        }
    }
}
