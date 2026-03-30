//! Clipboard manager module for cross-platform clipboard operations

use arboard::Clipboard;
use std::fmt;

/// Clipboard type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardType {
    /// Standard system clipboard (Ctrl+C/V)
    Standard,
    /// X11 primary selection (select = copy, middle-click = paste)
    #[cfg(target_os = "linux")]
    Primary,
}

impl fmt::Display for ClipboardType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardType::Standard => write!(f, "standard clipboard"),
            #[cfg(target_os = "linux")]
            ClipboardType::Primary => write!(f, "primary selection"),
        }
    }
}

/// Paste confirmation options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteConfirmation {
    /// Always confirm before pasting
    Always,
    /// Confirm only for multiline or large pastes
    Smart,
    /// Never confirm
    Never,
}

/// Cross-platform clipboard manager
pub struct ClipboardManager {
    clipboard: Option<Clipboard>,
    confirmation_mode: PasteConfirmation,
}

impl ClipboardManager {
    /// Create a new clipboard manager
    pub fn new() -> Self {
        let clipboard = match Clipboard::new() {
            Ok(cb) => {
                log::info!("Clipboard initialized successfully");
                Some(cb)
            }
            Err(e) => {
                log::error!("Failed to initialize clipboard: {}", e);
                None
            }
        };

        Self {
            clipboard,
            confirmation_mode: PasteConfirmation::Smart,
        }
    }

    /// Copy text to clipboard
    pub fn copy(&mut self, text: &str, clipboard_type: ClipboardType) -> Result<(), String> {
        let clipboard = self
            .clipboard
            .as_mut()
            .ok_or_else(|| "Clipboard not initialized".to_string())?;

        match clipboard_type {
            ClipboardType::Standard => {
                clipboard
                    .set_text(text)
                    .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
            }

            #[cfg(target_os = "linux")]
            ClipboardType::Primary => {
                // On Linux, arboard supports primary selection via the Selection type
                // For now, we'll fall back to standard clipboard
                // TODO: Implement proper X11 primary selection support
                clipboard
                    .set_text(text)
                    .map_err(|e| format!("Failed to copy to primary selection: {}", e))?;
                log::warn!("Primary selection not yet implemented, copied to standard clipboard");
            }
        }

        Ok(())
    }

    /// Paste text from clipboard
    pub fn paste(&mut self, clipboard_type: ClipboardType) -> Result<String, String> {
        let clipboard = self
            .clipboard
            .as_mut()
            .ok_or_else(|| "Clipboard not initialized".to_string())?;

        match clipboard_type {
            ClipboardType::Standard => clipboard
                .get_text()
                .map_err(|e| format!("Failed to paste from clipboard: {}", e)),

            #[cfg(target_os = "linux")]
            ClipboardType::Primary => {
                // TODO: Implement proper X11 primary selection support
                clipboard
                    .get_text()
                    .map_err(|e| format!("Failed to paste from primary selection: {}", e))
            }
        }
    }

    /// Set paste confirmation mode
    pub fn set_confirmation_mode(&mut self, mode: PasteConfirmation) {
        self.confirmation_mode = mode;
        log::info!("Paste confirmation mode set to: {:?}", mode);
    }

    /// Get current confirmation mode
    pub fn confirmation_mode(&self) -> PasteConfirmation {
        self.confirmation_mode
    }

    /// Check if clipboard is available
    pub fn is_available(&self) -> bool {
        self.clipboard.is_some()
    }

    /// Clear clipboard contents
    pub fn clear(&mut self) -> Result<(), String> {
        let clipboard = self
            .clipboard
            .as_mut()
            .ok_or_else(|| "Clipboard not initialized".to_string())?;

        clipboard
            .clear()
            .map_err(|e| format!("Failed to clear clipboard: {}", e))
    }
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_manager_creation() {
        let manager = ClipboardManager::new();
        // Clipboard may or may not be available in test environments
        // Just verify it doesn't panic
        let _ = manager.is_available();
    }

    #[test]
    fn test_confirmation_mode() {
        let mut manager = ClipboardManager::new();

        assert_eq!(manager.confirmation_mode(), PasteConfirmation::Smart);

        manager.set_confirmation_mode(PasteConfirmation::Always);
        assert_eq!(manager.confirmation_mode(), PasteConfirmation::Always);

        manager.set_confirmation_mode(PasteConfirmation::Never);
        assert_eq!(manager.confirmation_mode(), PasteConfirmation::Never);
    }

    #[test]
    #[ignore] // Ignore by default as it requires a display server
    fn test_copy_paste_roundtrip() {
        let mut manager = ClipboardManager::new();

        if !manager.is_available() {
            println!("Clipboard not available, skipping test");
            return;
        }

        let test_text = "Hello, Scarab Terminal!";

        // Copy
        let copy_result = manager.copy(test_text, ClipboardType::Standard);
        assert!(copy_result.is_ok(), "Copy failed: {:?}", copy_result);

        // Paste
        let paste_result = manager.paste(ClipboardType::Standard);
        assert!(paste_result.is_ok(), "Paste failed: {:?}", paste_result);

        let pasted = paste_result.unwrap();
        assert_eq!(pasted, test_text, "Pasted text doesn't match");
    }

    #[test]
    #[ignore] // Ignore by default as it requires a display server
    fn test_multiline_copy_paste() {
        let mut manager = ClipboardManager::new();

        if !manager.is_available() {
            println!("Clipboard not available, skipping test");
            return;
        }

        let test_text = "Line 1\nLine 2\nLine 3\nLine 4";

        let copy_result = manager.copy(test_text, ClipboardType::Standard);
        assert!(copy_result.is_ok());

        let paste_result = manager.paste(ClipboardType::Standard);
        assert!(paste_result.is_ok());

        let pasted = paste_result.unwrap();
        assert_eq!(pasted, test_text);
    }
}
