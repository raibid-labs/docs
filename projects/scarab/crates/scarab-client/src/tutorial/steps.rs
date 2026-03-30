//! Tutorial step definitions
//!
//! Defines the 8-step guided tour through Scarab features

use super::{TerminalContext, TutorialStep};

pub struct TutorialSteps;

impl TutorialSteps {
    /// Create all 8 tutorial steps
    pub fn create_all_steps() -> Vec<TutorialStep> {
        vec![
            Self::step_welcome(),
            Self::step_navigation(),
            Self::step_scrollback(),
            Self::step_link_hints(),
            Self::step_command_palette(),
            Self::step_plugins(),
            Self::step_configuration(),
            Self::step_completion(),
        ]
    }

    /// Step 1: Welcome and introduction
    fn step_welcome() -> TutorialStep {
        TutorialStep {
            id: "welcome".to_string(),
            title: "Welcome to Scarab Terminal!".to_string(),
            description: concat!(
                "Scarab is a next-generation GPU-accelerated terminal emulator ",
                "with F# plugins and zero-copy IPC.\n\n",
                "This quick tutorial will guide you through the key features.\n\n",
                "It will take about 5 minutes to complete."
            )
            .to_string(),
            instruction: "Press SPACE or ENTER to continue".to_string(),
            validation: |_| true, // Always valid, waiting for user input
            hint: Some("You can skip this tutorial anytime by pressing ESC".to_string()),
            visual_demo: Some("assets/demos/welcome.gif".to_string()),
        }
    }

    /// Step 2: Basic navigation and command execution
    fn step_navigation() -> TutorialStep {
        TutorialStep {
            id: "navigation".to_string(),
            title: "Basic Navigation".to_string(),
            description: concat!(
                "Scarab works just like any terminal emulator.\n\n",
                "Type commands, press Enter to execute, and see the output.\n\n",
                "Try typing a simple command to get started."
            )
            .to_string(),
            instruction: "Try running: ls -la".to_string(),
            validation: |ctx: &TerminalContext| {
                // Check if user ran any command
                ctx.last_command.is_some()
            },
            hint: Some("Just type 'ls -la' and press Enter".to_string()),
            visual_demo: None,
        }
    }

    /// Step 3: Scrollback navigation
    fn step_scrollback() -> TutorialStep {
        TutorialStep {
            id: "scrollback".to_string(),
            title: "Scrollback History".to_string(),
            description: concat!(
                "Scarab maintains a scrollback buffer of your terminal output.\n\n",
                "Use your mouse wheel or trackpad to scroll through history.\n\n",
                "This is useful for reviewing long command outputs."
            )
            .to_string(),
            instruction: "Use your mouse wheel to scroll up and down".to_string(),
            validation: |ctx: &TerminalContext| {
                // Check if user scrolled
                ctx.scroll_position != 0
            },
            hint: Some("Try scrolling up with your mouse wheel".to_string()),
            visual_demo: Some("assets/demos/scrollback.gif".to_string()),
        }
    }

    /// Step 4: Link hints feature
    fn step_link_hints() -> TutorialStep {
        TutorialStep {
            id: "link_hints".to_string(),
            title: "Link Hints".to_string(),
            description: concat!(
                "Scarab can detect URLs in your terminal output.\n\n",
                "Press Ctrl+Shift+O to highlight all links with keyboard shortcuts.\n\n",
                "This lets you open links without using your mouse."
            )
            .to_string(),
            instruction: "Press Ctrl+Shift+O to trigger link hints".to_string(),
            validation: |ctx: &TerminalContext| ctx.link_hints_triggered,
            hint: Some("The keyboard shortcut is: Ctrl + Shift + O".to_string()),
            visual_demo: Some("assets/demos/link-hints-demo.gif".to_string()),
        }
    }

    /// Step 5: Command palette
    fn step_command_palette() -> TutorialStep {
        TutorialStep {
            id: "command_palette".to_string(),
            title: "Command Palette".to_string(),
            description: concat!(
                "The command palette gives you quick access to all Scarab features.\n\n",
                "Press Ctrl+Shift+P to open it, then use fuzzy search to find commands.\n\n",
                "It's like VS Code's command palette, but for your terminal!"
            )
            .to_string(),
            instruction: "Press Ctrl+Shift+P to open the command palette".to_string(),
            validation: |ctx: &TerminalContext| ctx.palette_opened,
            hint: Some("Keyboard shortcut: Ctrl + Shift + P".to_string()),
            visual_demo: Some("assets/demos/command-palette.gif".to_string()),
        }
    }

    /// Step 6: Plugin system overview
    fn step_plugins() -> TutorialStep {
        TutorialStep {
            id: "plugins".to_string(),
            title: "Plugin System".to_string(),
            description: concat!(
                "Scarab uses Fusabi (F# for Rust) for plugins.\n\n",
                "You can write plugins in F# to:\n",
                "  - Filter terminal output\n",
                "  - Add custom keybindings\n",
                "  - Create UI overlays\n",
                "  - Integrate with external tools\n\n",
                "Plugins are stored in: ~/.config/scarab/plugins/"
            )
            .to_string(),
            instruction: "Press SPACE to learn about creating your first plugin".to_string(),
            validation: |_| true, // Always valid
            hint: Some(
                "We'll show you how to create a simple plugin in the next tutorial".to_string(),
            ),
            visual_demo: Some("assets/demos/plugin-install.gif".to_string()),
        }
    }

    /// Step 7: Configuration
    fn step_configuration() -> TutorialStep {
        TutorialStep {
            id: "configuration".to_string(),
            title: "Configuration".to_string(),
            description: concat!(
                "Scarab is highly customizable via TOML configuration.\n\n",
                "Config file location: ~/.config/scarab/config.toml\n\n",
                "You can customize:\n",
                "  - Colors and themes\n",
                "  - Fonts and rendering\n",
                "  - Keybindings\n",
                "  - Plugin settings\n",
                "  - Performance options"
            )
            .to_string(),
            instruction: "Press SPACE to continue".to_string(),
            validation: |_| true, // Always valid
            hint: Some("Check docs/tutorials/02-customization.md for examples".to_string()),
            visual_demo: Some("assets/demos/theme-switch.gif".to_string()),
        }
    }

    /// Step 8: Completion and next steps
    fn step_completion() -> TutorialStep {
        TutorialStep {
            id: "completion".to_string(),
            title: "Tutorial Complete!".to_string(),
            description: concat!(
                "Congratulations! You've completed the Scarab tutorial.\n\n",
                "Next steps:\n",
                "  1. Read the documentation in docs/tutorials/\n",
                "  2. Create your first plugin (see examples/plugins/)\n",
                "  3. Customize your config (~/.config/scarab/config.toml)\n",
                "  4. Join our community on GitHub Discussions\n\n",
                "You can replay this tutorial anytime with: scarab --tutorial"
            )
            .to_string(),
            instruction: "Press SPACE to start using Scarab!".to_string(),
            validation: |_| true, // Always valid
            hint: Some("Happy hacking! 🚀".to_string()),
            visual_demo: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_steps_created() {
        let steps = TutorialSteps::create_all_steps();
        assert_eq!(steps.len(), 8, "Should have exactly 8 tutorial steps");
    }

    #[test]
    fn test_step_ids_unique() {
        let steps = TutorialSteps::create_all_steps();
        let mut ids: Vec<String> = steps.iter().map(|s| s.id.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(
            ids.len(),
            8,
            "All step IDs should be unique"
        );
    }

    #[test]
    fn test_welcome_step() {
        let step = TutorialSteps::step_welcome();
        assert_eq!(step.id, "welcome");
        assert!(step.title.contains("Welcome"));
    }

    #[test]
    fn test_navigation_validation() {
        let step = TutorialSteps::step_navigation();

        let ctx_no_command = TerminalContext {
            last_command: None,
            scroll_position: 0,
            palette_opened: false,
            link_hints_triggered: false,
        };
        assert!(!( step.validation)(&ctx_no_command));

        let ctx_with_command = TerminalContext {
            last_command: Some("ls".to_string()),
            scroll_position: 0,
            palette_opened: false,
            link_hints_triggered: false,
        };
        assert!((step.validation)(&ctx_with_command));
    }

    #[test]
    fn test_scrollback_validation() {
        let step = TutorialSteps::step_scrollback();

        let ctx_no_scroll = TerminalContext {
            last_command: None,
            scroll_position: 0,
            palette_opened: false,
            link_hints_triggered: false,
        };
        assert!(!(step.validation)(&ctx_no_scroll));

        let ctx_scrolled = TerminalContext {
            last_command: None,
            scroll_position: -10,
            palette_opened: false,
            link_hints_triggered: false,
        };
        assert!((step.validation)(&ctx_scrolled));
    }
}
