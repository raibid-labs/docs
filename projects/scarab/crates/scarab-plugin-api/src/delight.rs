//! Delightful messages, ASCII art, and celebrations for the plugin system
//!
//! Making plugin development and usage more enjoyable, one message at a time.

use rand::Rng;

/// Loading messages to show when plugins are being loaded
pub const LOADING_MESSAGES: &[&str] = &[
    "Summoning plugin spirits...",
    "Warming up the Fusabi engine...",
    "Compiling joy into bytecode...",
    "Sprinkling some plugin magic...",
    "Waking up the terminal wizards...",
    "Brewing fresh plugin goodness...",
    "Initializing awesomeness modules...",
    "Loading super powers...",
    "Charging plugin batteries...",
    "Consulting the plugin oracle...",
    "Calibrating terminal enhancers...",
    "Activating plugin neurons...",
];

/// Success messages for plugin loading
pub const SUCCESS_MESSAGES: &[&str] = &[
    "Plugin loaded successfully!",
    "And we're live!",
    "That's what I'm talking about!",
    "Nailed it!",
    "Plugin is ready to roll!",
    "Successfully deployed!",
    "Good to go!",
    "All systems green!",
    "Plugin successfully summoned!",
    "Welcome aboard!",
];

/// Friendly error message prefixes
pub const ERROR_PREFIXES: &[&str] = &[
    "Oops, something went sideways:",
    "Well, this is awkward:",
    "Houston, we have a problem:",
    "Not quite what we expected:",
    "Hmm, that didn't work:",
    "Plot twist:",
    "Yikes:",
    "Oh no:",
];

/// ASCII art for celebrations
pub const CONFETTI: &str = r#"
    *  .  *    .  *  .    *
  .  *  .  *  .  *  .  *  .
*  .  *  .  *  .  *  .  *  .
  .  *  .  *  .  *  .  *  .
"#;

pub const PARTY_POPPER: &str = r#"
    \  |  /
     \ | /
      \|/
   --- * ---
      /|\
     / | \
    /  |  \
"#;

pub const TROPHY: &str = r#"
       ___
      |   |
      |___|
     /|   |\
    /_|___|_\
      |   |
      |___|
"#;

pub const ROCKET: &str = r#"
      /\
     /  \
    |    |
    | ** |
    |    |
   /|    |\
  / |    | \
 /  |    |  \
    |    |
   /|    |\
  / '--' |
 /________\
"#;

/// Get a random loading message
pub fn random_loading_message() -> &'static str {
    let mut rng = rand::thread_rng();
    LOADING_MESSAGES[rng.gen_range(0..LOADING_MESSAGES.len())]
}

/// Get a random success message
pub fn random_success_message() -> &'static str {
    let mut rng = rand::thread_rng();
    SUCCESS_MESSAGES[rng.gen_range(0..SUCCESS_MESSAGES.len())]
}

/// Get a random error prefix
pub fn random_error_prefix() -> &'static str {
    let mut rng = rand::thread_rng();
    ERROR_PREFIXES[rng.gen_range(0..ERROR_PREFIXES.len())]
}

/// Achievement system for plugin milestones
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Achievement {
    FirstPlugin,
    TenPlugins,
    FiftyPlugins,
    HundredPlugins,
    FirstCustomPlugin,
    PluginDeveloper,
    PluginMaster,
    ZeroFailures,
}

impl Achievement {
    /// Get achievement title
    pub fn title(&self) -> &'static str {
        match self {
            Achievement::FirstPlugin => "Plugin Pioneer",
            Achievement::TenPlugins => "Plugin Enthusiast",
            Achievement::FiftyPlugins => "Plugin Collector",
            Achievement::HundredPlugins => "Plugin Legend",
            Achievement::FirstCustomPlugin => "Plugin Creator",
            Achievement::PluginDeveloper => "Plugin Developer",
            Achievement::PluginMaster => "Plugin Master",
            Achievement::ZeroFailures => "Flawless Execution",
        }
    }

    /// Get achievement description
    pub fn description(&self) -> &'static str {
        match self {
            Achievement::FirstPlugin => "Loaded your very first plugin!",
            Achievement::TenPlugins => "Running 10 plugins like a boss!",
            Achievement::FiftyPlugins => "Wow, 50 plugins! Your terminal is a powerhouse!",
            Achievement::HundredPlugins => "100 plugins!? You're absolutely legendary!",
            Achievement::FirstCustomPlugin => "Created and loaded your own custom plugin!",
            Achievement::PluginDeveloper => "Successfully built 5 custom plugins!",
            Achievement::PluginMaster => "Built 25 custom plugins. You're a plugin wizard!",
            Achievement::ZeroFailures => "Plugin running smoothly with zero failures!",
        }
    }

    /// Get achievement emoji
    pub fn emoji(&self) -> &'static str {
        match self {
            Achievement::FirstPlugin => "🎉",
            Achievement::TenPlugins => "⭐",
            Achievement::FiftyPlugins => "💎",
            Achievement::HundredPlugins => "👑",
            Achievement::FirstCustomPlugin => "🛠️",
            Achievement::PluginDeveloper => "🚀",
            Achievement::PluginMaster => "🧙",
            Achievement::ZeroFailures => "✨",
        }
    }

    /// Format achievement for display
    pub fn format(&self) -> String {
        format!(
            "{} {} - {}",
            self.emoji(),
            self.title(),
            self.description()
        )
    }

    /// Get ASCII art for this achievement
    pub fn ascii_art(&self) -> Option<&'static str> {
        match self {
            Achievement::FirstPlugin => Some(PARTY_POPPER),
            Achievement::HundredPlugins => Some(TROPHY),
            Achievement::PluginMaster => Some(ROCKET),
            _ => None,
        }
    }
}

/// Plugin mood indicator based on performance
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginMood {
    Happy,
    Content,
    Worried,
    Struggling,
    Disabled,
}

impl PluginMood {
    /// Determine mood from failure count
    pub fn from_failure_count(failures: u32, max_failures: u32, enabled: bool) -> Self {
        if !enabled {
            return PluginMood::Disabled;
        }

        match failures {
            0 => PluginMood::Happy,
            1 => PluginMood::Content,
            f if f < max_failures - 1 => PluginMood::Worried,
            _ => PluginMood::Struggling,
        }
    }

    /// Get emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            PluginMood::Happy => "😊",
            PluginMood::Content => "🙂",
            PluginMood::Worried => "😟",
            PluginMood::Struggling => "😰",
            PluginMood::Disabled => "💤",
        }
    }

    /// Get descriptive text
    pub fn description(&self) -> &'static str {
        match self {
            PluginMood::Happy => "Running perfectly!",
            PluginMood::Content => "Doing well",
            PluginMood::Worried => "Having some trouble",
            PluginMood::Struggling => "Needs attention",
            PluginMood::Disabled => "Taking a break",
        }
    }
}

/// Tips and tricks for plugin developers
pub const DEVELOPER_TIPS: &[&str] = &[
    "Pro tip: Use ctx.log() to debug your plugin hooks!",
    "Did you know? Plugins can communicate via shared state in PluginContext.",
    "Hot reload tip: .fsx scripts reload automatically, no rebuild needed!",
    "Performance tip: Keep your hooks lightweight for snappy terminal response.",
    "Debug trick: Check plugin.failure_count to see if your hooks are erroring.",
    "Cool feature: Plugins can define custom emojis and colors for personality!",
    "Remember: on_output runs for every line, so optimize for speed!",
    "Quick win: Add a catchphrase to your plugin metadata for extra charm.",
    "Testing tip: Use on_attach to show debug overlays when clients connect.",
    "Style guide: Choose a unique emoji that represents your plugin's purpose.",
];

/// Get a random developer tip
pub fn random_developer_tip() -> &'static str {
    let mut rng = rand::thread_rng();
    DEVELOPER_TIPS[rng.gen_range(0..DEVELOPER_TIPS.len())]
}

/// Format a friendly error message with suggestions
pub fn friendly_error(error: &str, suggestions: Vec<&str>) -> String {
    let mut msg = format!("{} {}\n", random_error_prefix(), error);

    if !suggestions.is_empty() {
        msg.push_str("\n💡 Suggestions:\n");
        for (i, suggestion) in suggestions.iter().enumerate() {
            msg.push_str(&format!("  {}. {}\n", i + 1, suggestion));
        }
    }

    msg
}

/// Format a celebration message for special occasions
pub fn celebration_message(occasion: &str, ascii_art: Option<&str>) -> String {
    let mut msg = format!("\n🎊 {} 🎊\n", occasion);

    if let Some(art) = ascii_art {
        msg.push_str(art);
        msg.push('\n');
    }

    msg
}

/// Check if today is a special date and return a message
pub fn special_date_message() -> Option<String> {
    use chrono::{Datelike, Local};

    let today = Local::now();
    let month = today.month();
    let day = today.day();

    match (month, day) {
        (1, 1) => Some("🎆 Happy New Year! May your plugins be bug-free! 🎆".to_string()),
        (3, 14) => Some("🥧 Happy Pi Day! 3.14159... plugins loaded! 🥧".to_string()),
        (4, 1) => Some("🤡 April Fools! Just kidding, your plugins are real. 🤡".to_string()),
        (5, 4) => Some("⭐ May the 4th be with you! Plugin Force activated! ⭐".to_string()),
        (10, 31) => Some("🎃 Happy Halloween! Your plugins are spooktacular! 🎃".to_string()),
        (12, 25) => Some("🎄 Merry Christmas! Unwrapping plugin presents! 🎄".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_mood() {
        assert_eq!(PluginMood::from_failure_count(0, 3, true), PluginMood::Happy);
        assert_eq!(PluginMood::from_failure_count(1, 3, true), PluginMood::Content);
        assert_eq!(PluginMood::from_failure_count(2, 3, true), PluginMood::Struggling);
        assert_eq!(PluginMood::from_failure_count(0, 3, false), PluginMood::Disabled);
    }

    #[test]
    fn test_achievement_formatting() {
        let achievement = Achievement::FirstPlugin;
        let formatted = achievement.format();
        assert!(formatted.contains("Plugin Pioneer"));
        assert!(formatted.contains("🎉"));
    }

    #[test]
    fn test_friendly_error() {
        let error = friendly_error(
            "Plugin failed to load",
            vec!["Check the file path", "Verify plugin format"],
        );
        assert!(error.contains("Plugin failed to load"));
        assert!(error.contains("Check the file path"));
    }
}
