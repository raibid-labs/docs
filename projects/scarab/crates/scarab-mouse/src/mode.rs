//! Mouse mode detection and management

use crate::types::MouseMode;

/// Mouse mode detector that scans terminal output for mode change sequences
pub struct ModeDetector {
    current_mode: MouseMode,
}

impl Default for ModeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ModeDetector {
    pub fn new() -> Self {
        Self {
            current_mode: MouseMode::Normal,
        }
    }

    /// Get current mouse mode
    pub fn mode(&self) -> MouseMode {
        self.current_mode
    }

    /// Detect mode changes from ANSI escape sequences
    ///
    /// Mouse reporting modes:
    /// - CSI ? 1000 h/l - X10 mouse reporting (press only)
    /// - CSI ? 1002 h/l - Button-event tracking (press/release)
    /// - CSI ? 1003 h/l - Any-event tracking (motion too)
    /// - CSI ? 1006 h/l - SGR extended mouse mode
    /// - CSI ? 1015 h/l - urxvt mouse mode
    ///
    /// 'h' enables, 'l' disables
    pub fn scan_sequence(&mut self, data: &[u8]) -> Option<MouseMode> {
        let s = std::str::from_utf8(data).ok()?;

        // Look for mouse mode enable sequences
        if s.contains("\x1b[?1000h")
            || s.contains("\x1b[?1002h")
            || s.contains("\x1b[?1003h")
            || s.contains("\x1b[?1006h")
            || s.contains("\x1b[?1015h")
        {
            self.current_mode = MouseMode::Application;
            return Some(MouseMode::Application);
        }

        // Look for mouse mode disable sequences
        if s.contains("\x1b[?1000l")
            || s.contains("\x1b[?1002l")
            || s.contains("\x1b[?1003l")
            || s.contains("\x1b[?1006l")
            || s.contains("\x1b[?1015l")
        {
            self.current_mode = MouseMode::Normal;
            return Some(MouseMode::Normal);
        }

        None
    }

    /// Force mode change (for manual override)
    pub fn set_mode(&mut self, mode: MouseMode) {
        self.current_mode = mode;
    }
}

/// Heuristic detection for applications that typically use mouse mode
pub struct AppHeuristics;

impl AppHeuristics {
    /// Detect if current process is likely to use mouse mode
    pub fn detect_mouse_app(process_name: &str) -> bool {
        let name_lower = process_name.to_lowercase();

        matches!(
            name_lower.as_str(),
            "vim" | "nvim" | "emacs" | "nano" | "tmux" | "screen" | "htop" | "less" | "more"
                | "ranger" | "mc" | "fzf"
        )
    }

    /// Get description of mouse behavior for known apps
    pub fn get_app_description(process_name: &str) -> Option<&'static str> {
        let name_lower = process_name.to_lowercase();

        match name_lower.as_str() {
            "vim" | "nvim" => Some("Vim uses mouse for cursor positioning and visual selection"),
            "emacs" => Some("Emacs uses mouse for cursor and region selection"),
            "tmux" => Some("Tmux uses mouse for pane selection and resizing"),
            "screen" => Some("Screen uses mouse for window management"),
            "htop" => Some("Htop uses mouse for process selection"),
            "less" | "more" => Some("Pager uses mouse for scrolling"),
            "ranger" | "mc" => Some("File manager uses mouse for navigation"),
            "fzf" => Some("Fuzzy finder uses mouse for item selection"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_detector_default() {
        let detector = ModeDetector::new();
        assert_eq!(detector.mode(), MouseMode::Normal);
    }

    #[test]
    fn test_detect_enable() {
        let mut detector = ModeDetector::new();

        // Enable X10 mode
        let result = detector.scan_sequence(b"\x1b[?1000h");
        assert_eq!(result, Some(MouseMode::Application));
        assert_eq!(detector.mode(), MouseMode::Application);

        // Enable button tracking
        let result = detector.scan_sequence(b"\x1b[?1002h");
        assert_eq!(result, Some(MouseMode::Application));
    }

    #[test]
    fn test_detect_disable() {
        let mut detector = ModeDetector::new();

        // Enable then disable
        detector.scan_sequence(b"\x1b[?1000h");
        assert_eq!(detector.mode(), MouseMode::Application);

        let result = detector.scan_sequence(b"\x1b[?1000l");
        assert_eq!(result, Some(MouseMode::Normal));
        assert_eq!(detector.mode(), MouseMode::Normal);
    }

    #[test]
    fn test_app_detection() {
        assert!(AppHeuristics::detect_mouse_app("vim"));
        assert!(AppHeuristics::detect_mouse_app("tmux"));
        assert!(AppHeuristics::detect_mouse_app("htop"));
        assert!(!AppHeuristics::detect_mouse_app("bash"));
        assert!(!AppHeuristics::detect_mouse_app("zsh"));
    }

    #[test]
    fn test_app_description() {
        assert!(AppHeuristics::get_app_description("vim").is_some());
        assert!(AppHeuristics::get_app_description("tmux").is_some());
        assert!(AppHeuristics::get_app_description("bash").is_none());
    }
}
