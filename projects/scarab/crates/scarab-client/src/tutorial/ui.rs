//! Tutorial UI rendering
//!
//! Renders the tutorial overlay with beautiful ASCII art borders

use super::{TutorialStep, TutorialSystem};
use bevy::prelude::*;

pub struct TutorialUI;

impl TutorialUI {
    /// Render the current tutorial step as an overlay
    pub fn render_step(
        tutorial: &TutorialSystem,
        step: &TutorialStep,
    ) {
        // In a real implementation, this would render to the Bevy UI
        // For now, we'll print to console for demo purposes
        Self::print_step_overlay(tutorial, step);
    }

    /// Print tutorial overlay to console (placeholder for Bevy UI)
    fn print_step_overlay(tutorial: &TutorialSystem, step: &TutorialStep) {
        let width = 65;
        let step_num = tutorial.current_step + 1;
        let total_steps = tutorial.steps.len();
        let progress = tutorial.progress_percentage();

        println!("\n{}", "┌".to_string() + &"─".repeat(width - 2) + "┐");
        println!("{}", Self::center_text("SCARAB TUTORIAL", width));
        println!(
            "{}",
            Self::center_text(&format!("Step {} of {}", step_num, total_steps), width)
        );
        println!("{}", "├".to_string() + &"─".repeat(width - 2) + "┤");
        println!("│{:^width$}│", "", width = width - 2);
        println!(
            "{}",
            Self::center_text(&format!("  {}  ", step.title), width)
        );
        println!("│{:^width$}│", "", width = width - 2);

        // Description (word-wrapped)
        for line in Self::wrap_text(&step.description, width - 6) {
            println!("│  {:<width$}  │", line, width = width - 6);
        }

        println!("│{:^width$}│", "", width = width - 2);

        // Instruction (highlighted)
        println!(
            "{}",
            Self::center_text(&format!("▶ {}", step.instruction), width)
        );

        println!("│{:^width$}│", "", width = width - 2);

        // Hint (if available)
        if let Some(hint) = &step.hint {
            println!("│  {:<width$}  │", format!("💡 Hint: {}", hint), width = width - 6);
            println!("│{:^width$}│", "", width = width - 2);
        }

        // Progress bar
        Self::print_progress_bar(progress, width - 6);

        // Controls
        println!("│{:^width$}│", "", width = width - 2);
        println!(
            "│  {:<width$}  │",
            "[ESC: Skip]  [BACKSPACE: Back]  [SPACE/ENTER: Next →]",
            width = width - 6
        );
        println!("{}", "└".to_string() + &"─".repeat(width - 2) + "┘");
    }

    /// Center text within a box of given width
    fn center_text(text: &str, width: usize) -> String {
        let text_len = text.len();
        if text_len >= width - 2 {
            return format!("│{}│", &text[..width - 2]);
        }

        let padding = (width - 2 - text_len) / 2;
        let right_padding = width - 2 - text_len - padding;
        format!(
            "│{}{:^width$}{}│",
            " ".repeat(padding),
            text,
            " ".repeat(right_padding),
            width = text_len
        )
    }

    /// Wrap text to fit within specified width
    fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();

        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + word.len() + 1 <= max_width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        lines
    }

    /// Print a progress bar
    fn print_progress_bar(percentage: f32, width: usize) {
        let filled = ((percentage / 100.0) * width as f32) as usize;
        let empty = width.saturating_sub(filled);

        let bar = format!(
            "│  {}{}  │",
            "█".repeat(filled),
            "░".repeat(empty)
        );
        println!("{}", bar);
        println!(
            "│  {:<width$}  │",
            format!("{}% complete", percentage as u32),
            width = width
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text() {
        let text = "This is a long text that needs to be wrapped into multiple lines";
        let wrapped = TutorialUI::wrap_text(text, 20);

        assert!(wrapped.len() > 1, "Text should be wrapped");
        for line in &wrapped {
            assert!(line.len() <= 20, "Each line should be <= 20 chars");
        }
    }

    #[test]
    fn test_wrap_text_short() {
        let text = "Short text";
        let wrapped = TutorialUI::wrap_text(text, 50);

        assert_eq!(wrapped.len(), 1, "Short text should not wrap");
        assert_eq!(wrapped[0], "Short text");
    }

    #[test]
    fn test_center_text() {
        let centered = TutorialUI::center_text("Hello", 20);
        assert!(centered.starts_with('│'));
        assert!(centered.ends_with('│'));
        assert!(centered.contains("Hello"));
    }
}
