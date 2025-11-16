use ratatui::layout::Rect;

/// A navigable region in the UI, such as a list row, table cell, tab, or button.
#[derive(Debug, Clone)]
pub struct NavTarget {
    pub id: u64,
    pub rect: Rect,
    pub label: Option<String>,
    // In future this can be expanded into a rich action enum.
}

impl NavTarget {
    pub fn new(id: u64, rect: Rect) -> Self {
        Self { id, rect, label: None }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Registry of targets discovered during a frame.
#[derive(Debug, Default)]
pub struct TargetRegistry {
    targets: Vec<NavTarget>,
}

impl TargetRegistry {
    pub fn clear(&mut self) {
        self.targets.clear();
    }

    pub fn register(&mut self, target: NavTarget) {
        self.targets.push(target);
    }

    pub fn all(&self) -> &[NavTarget] {
        &self.targets
    }

    pub fn is_empty(&self) -> bool {
        self.targets.is_empty()
    }
}
