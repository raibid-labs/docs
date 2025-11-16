use crate::core::targets::{NavTarget, TargetRegistry};
use ratatui::layout::Rect;

/// Trait implemented by widgets (or their wrappers) that can expose
/// navigable targets to Locust.
pub trait Navigable {
    fn nav_targets(&self, area: Rect, out: &mut TargetRegistry);
}

/// Helper for common "one target per row" widgets like List or simple Tables.
pub fn register_simple_row_targets(
    area: Rect,
    row_count: usize,
    start_id: u64,
    out: &mut TargetRegistry,
) {
    let visible_rows = row_count.min(area.height as usize);
    for i in 0..visible_rows {
        let y = area.y + i as u16;
        let rect = Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        };
        let id = start_id + i as u64;
        out.register(NavTarget::new(id, rect));
    }
}
