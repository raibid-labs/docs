//! Common types used throughout the plugin API

pub use scarab_protocol::{ModalItem, OverlayStyle};
use serde::{Deserialize, Serialize};

/// Command sent from plugin to daemon
#[derive(Debug, Clone)]
pub enum RemoteCommand {
    DrawOverlay {
        id: u64,
        x: u16,
        y: u16,
        text: String,
        style: OverlayStyle,
    },
    ClearOverlays {
        id: Option<u64>,
    },
    ShowModal {
        title: String,
        items: Vec<ModalItem>,
    },
    PluginLog {
        plugin_name: String,
        level: crate::context::LogLevel,
        message: String,
    },
    PluginNotify {
        title: String,
        body: String,
        level: crate::context::NotifyLevel,
    },
}

/// Action that a plugin hook can return
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Continue processing with next plugin
    Continue,
    /// Stop processing, don't call remaining plugins
    Stop,
    /// Modify the data and continue
    Modify(Vec<u8>),
}

impl Action {
    /// Check if this action modifies data
    pub fn is_modify(&self) -> bool {
        matches!(self, Action::Modify(_))
    }

    /// Check if this action stops processing
    pub fn is_stop(&self) -> bool {
        matches!(self, Action::Stop)
    }
}

/// Type of hook being executed
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookType {
    /// Before output is displayed
    PreOutput,
    /// After input is received
    PostInput,
    /// Before command is executed
    PreCommand,
    /// After command completes
    PostCommand,
    /// Terminal resize event
    OnResize,
    /// Client attached
    OnAttach,
    /// Client detached
    OnDetach,
}

impl HookType {
    /// Get all hook types
    pub fn all() -> &'static [HookType] {
        &[
            HookType::PreOutput,
            HookType::PostInput,
            HookType::PreCommand,
            HookType::PostCommand,
            HookType::OnResize,
            HookType::OnAttach,
            HookType::OnDetach,
        ]
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            HookType::PreOutput => "pre-output",
            HookType::PostInput => "post-input",
            HookType::PreCommand => "pre-command",
            HookType::PostCommand => "post-command",
            HookType::OnResize => "on-resize",
            HookType::OnAttach => "on-attach",
            HookType::OnDetach => "on-detach",
        }
    }
}

/// Information about a loaded plugin with personality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Plugin homepage URL
    pub homepage: Option<String>,
    /// API version required
    pub api_version: String,
    /// Minimum Scarab version
    pub min_scarab_version: String,
    /// Whether plugin is currently enabled
    pub enabled: bool,
    /// Number of failures
    pub failure_count: u32,
    /// Plugin emoji (for display)
    #[serde(default)]
    pub emoji: Option<String>,
    /// Plugin color (hex code)
    #[serde(default)]
    pub color: Option<String>,
    /// Plugin catchphrase
    #[serde(default)]
    pub catchphrase: Option<String>,
}

impl PluginInfo {
    /// Create new plugin info
    pub fn new(
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
        author: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: author.into(),
            homepage: None,
            api_version: crate::API_VERSION.to_string(),
            min_scarab_version: "0.1.0".to_string(),
            enabled: true,
            failure_count: 0,
            emoji: None,
            color: None,
            catchphrase: None,
        }
    }

    /// Get display name with emoji if available
    pub fn display_name(&self) -> String {
        if let Some(emoji) = &self.emoji {
            format!("{} {}", emoji, self.name)
        } else {
            self.name.clone()
        }
    }

    /// Get plugin mood based on failure count
    pub fn mood(&self) -> crate::delight::PluginMood {
        crate::delight::PluginMood::from_failure_count(self.failure_count, 3, self.enabled)
    }
}

/// Terminal cell representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    /// Character content
    pub c: char,
    /// Foreground color (RGB)
    pub fg: (u8, u8, u8),
    /// Background color (RGB)
    pub bg: (u8, u8, u8),
    /// Bold flag
    pub bold: bool,
    /// Italic flag
    pub italic: bool,
    /// Underline flag
    pub underline: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            fg: (255, 255, 255),
            bg: (0, 0, 0),
            bold: false,
            italic: false,
            underline: false,
        }
    }
}
