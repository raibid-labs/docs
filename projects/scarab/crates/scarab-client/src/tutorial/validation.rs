//! Tutorial step validation helpers

use super::TerminalContext;

/// Validation helpers for tutorial steps
pub struct TutorialValidation;

impl TutorialValidation {
    /// Check if user has executed a command
    pub fn has_executed_command(ctx: &TerminalContext) -> bool {
        ctx.last_command.is_some()
    }

    /// Check if user has scrolled
    pub fn has_scrolled(ctx: &TerminalContext) -> bool {
        ctx.scroll_position != 0
    }

    /// Check if user opened link hints
    pub fn link_hints_opened(ctx: &TerminalContext) -> bool {
        ctx.link_hints_triggered
    }

    /// Check if user opened command palette
    pub fn palette_opened(ctx: &TerminalContext) -> bool {
        ctx.palette_opened
    }

    /// Always valid (for informational steps)
    pub fn always_valid(_ctx: &TerminalContext) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> TerminalContext {
        TerminalContext {
            last_command: None,
            scroll_position: 0,
            palette_opened: false,
            link_hints_triggered: false,
        }
    }

    #[test]
    fn test_has_executed_command() {
        let mut ctx = create_test_context();
        assert!(!TutorialValidation::has_executed_command(&ctx));

        ctx.last_command = Some("ls".to_string());
        assert!(TutorialValidation::has_executed_command(&ctx));
    }

    #[test]
    fn test_has_scrolled() {
        let mut ctx = create_test_context();
        assert!(!TutorialValidation::has_scrolled(&ctx));

        ctx.scroll_position = -10;
        assert!(TutorialValidation::has_scrolled(&ctx));

        ctx.scroll_position = 5;
        assert!(TutorialValidation::has_scrolled(&ctx));
    }

    #[test]
    fn test_link_hints_opened() {
        let mut ctx = create_test_context();
        assert!(!TutorialValidation::link_hints_opened(&ctx));

        ctx.link_hints_triggered = true;
        assert!(TutorialValidation::link_hints_opened(&ctx));
    }

    #[test]
    fn test_palette_opened() {
        let mut ctx = create_test_context();
        assert!(!TutorialValidation::palette_opened(&ctx));

        ctx.palette_opened = true;
        assert!(TutorialValidation::palette_opened(&ctx));
    }

    #[test]
    fn test_always_valid() {
        let ctx = create_test_context();
        assert!(TutorialValidation::always_valid(&ctx));
    }
}
