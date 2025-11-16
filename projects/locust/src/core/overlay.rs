/// Placeholder for future overlay-specific state.
///
/// For now this simply records whether any plugin requested an overlay
/// in the current frame. It can later be extended to track z-layers, etc.
#[derive(Debug, Default)]
pub struct OverlayState {
    pub has_overlay: bool,
}

impl OverlayState {
    pub fn begin_frame(&mut self) {
        self.has_overlay = false;
    }

    pub fn mark_has_overlay(&mut self) {
        self.has_overlay = true;
    }
}
