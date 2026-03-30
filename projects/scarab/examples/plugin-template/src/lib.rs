//! Scarab Plugin Template - Your Gateway to Terminal Awesomeness!
//!
//! This is your starting point for creating amazing Scarab plugins.
//! Copy this directory, unleash your creativity, and make something awesome!
//!
//! Pro tips:
//! - Give your plugin personality with an emoji and catchphrase
//! - Keep hooks lightweight for snappy terminal performance
//! - Use ctx.log() liberally while developing
//! - Check out other plugins for inspiration
//!
//! Happy hacking! 🚀

use scarab_plugin_api::{
    Action, Plugin, PluginContext, PluginMetadata, Result,
};
use async_trait::async_trait;

/// Your custom plugin struct
///
/// Add any state your plugin needs here. Think of this as your plugin's brain!
pub struct ExamplePlugin {
    metadata: PluginMetadata,
    // Add your plugin state here - get creative!
    notification_count: u32,
    times_saved_the_day: u32,
}

impl ExamplePlugin {
    /// Create a new instance of your plugin
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                "example-plugin",
                "0.1.0",
                "A delightful example plugin that shows you the ropes",
                "Your Name <your.email@example.com>",
            )
            .with_homepage("https://github.com/yourusername/scarab-plugin-example")
            .with_emoji("🎯")  // Pick an emoji that represents your plugin!
            .with_color("#FF6B6B")  // Choose your plugin's signature color
            .with_catchphrase("Making terminals awesome, one line at a time!"),
            notification_count: 0,
            times_saved_the_day: 0,
        }
    }
}

impl Default for ExamplePlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for ExamplePlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// Called when the plugin is loaded - Your plugin's grand entrance!
    async fn on_load(&mut self, ctx: &mut PluginContext) -> Result<()> {
        ctx.log(
            scarab_plugin_api::context::LogLevel::Info,
            "🎉 Example plugin loaded and ready to rock!",
        );

        // Initialize plugin state here
        self.notification_count = 0;
        self.times_saved_the_day = 0;

        // Fun fact: You can queue commands to the UI right from on_load!

        Ok(())
    }

    /// Called when the plugin is being unloaded - Time to say goodbye
    async fn on_unload(&mut self) -> Result<()> {
        log::info!(
            "👋 Example plugin signing off. Stats: {} notifications sent, {} times saved the day.",
            self.notification_count,
            self.times_saved_the_day
        );
        Ok(())
    }

    /// Hook called before output is displayed - Your chance to be a hero!
    ///
    /// This runs for EVERY line of terminal output, so keep it fast and focused.
    async fn on_output(&mut self, line: &str, ctx: &PluginContext) -> Result<Action> {
        // Example: Detect error keywords and send notification
        let error_keywords = ["ERROR", "FAIL", "PANIC", "FATAL", "DIED"];

        if error_keywords.iter().any(|&kw| line.contains(kw)) {
            ctx.notify(&format!("🚨 Error detected: {}", line));
            self.notification_count += 1;
            self.times_saved_the_day += 1;

            // You have three action options:
            // 1. Action::Continue - Let other plugins see this line too (most common)
            // 2. Action::Stop - Don't call remaining plugins (use sparingly!)
            // 3. Action::Modify(vec![...]) - Transform the line before display

            return Ok(Action::Continue);
        }

        // Celebrate success messages too!
        if line.contains("SUCCESS") || line.contains("PASSED") {
            ctx.log(
                scarab_plugin_api::context::LogLevel::Debug,
                &format!("✨ Yay! Success detected: {}", line),
            );
        }

        // Most of the time, just pass through unchanged
        Ok(Action::Continue)
    }

    /// Hook called after input is received - Intercept keystrokes like a ninja
    async fn on_input(&mut self, input: &[u8], _ctx: &PluginContext) -> Result<Action> {
        // Example: You could intercept certain key combinations here
        // Or auto-correct common typos, or add syntax highlighting hints

        // For now, just pass through
        Ok(Action::Continue)
    }

    /// Hook called before a command is executed - Know what's about to happen
    async fn on_pre_command(&mut self, command: &str, ctx: &PluginContext) -> Result<Action> {
        // Example: Log all commands (great for debugging!)
        ctx.log(
            scarab_plugin_api::context::LogLevel::Debug,
            &format!("🎯 Executing command: {}", command),
        );

        // You could also:
        // - Warn about dangerous commands (rm -rf, etc.)
        // - Add aliases or shortcuts
        // - Track command usage statistics

        Ok(Action::Continue)
    }

    /// Hook called after a command completes - Learn from the outcome
    async fn on_post_command(
        &mut self,
        command: &str,
        exit_code: i32,
        ctx: &PluginContext,
    ) -> Result<()> {
        // Example: Notify on command failures
        if exit_code != 0 {
            ctx.notify(&format!("💥 Command '{}' failed with exit code {}", command, exit_code));
            self.notification_count += 1;
        } else {
            // Celebrate the wins!
            ctx.log(
                scarab_plugin_api::context::LogLevel::Debug,
                &format!("✅ Command '{}' succeeded!", command),
            );
        }
        Ok(())
    }

    /// Hook called when terminal is resized - Adapt to change
    async fn on_resize(&mut self, cols: u16, rows: u16, ctx: &PluginContext) -> Result<()> {
        ctx.log(
            scarab_plugin_api::context::LogLevel::Debug,
            &format!("📏 Terminal resized to {}x{} - looking good!", cols, rows),
        );

        // You might want to:
        // - Adjust overlay positions
        // - Recalculate layout-dependent features
        // - Update your plugin's UI elements

        Ok(())
    }

    /// Hook called when a client attaches - Welcome new friends!
    async fn on_attach(&mut self, client_id: u64, ctx: &PluginContext) -> Result<()> {
        ctx.notify(&format!("👋 Client {} joined the party!", client_id));

        // This is a great place to:
        // - Show welcome messages
        // - Initialize client-specific state
        // - Draw custom UI elements

        Ok(())
    }

    /// Hook called when a client detaches - Say goodbye gracefully
    async fn on_detach(&mut self, client_id: u64, ctx: &PluginContext) -> Result<()> {
        ctx.notify(&format!("👋 Client {} left. Come back soon!", client_id));

        // Clean up any client-specific resources here

        Ok(())
    }
}

// Export the plugin creation function - Don't forget this!
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(ExamplePlugin::new())
}

// 🎉 Congratulations! You've got a working plugin template!
//
// Next steps:
// 1. Rename "ExamplePlugin" to something awesome
// 2. Update the metadata with your info
// 3. Pick a cool emoji and catchphrase
// 4. Implement the hooks you need
// 5. Test it out: cargo build && scarab-daemon --plugin path/to/your/plugin.so
// 6. Share your creation with the community!
//
// Remember: The best plugins are the ones that spark joy. Have fun! ✨
