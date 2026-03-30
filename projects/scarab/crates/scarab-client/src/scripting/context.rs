//! Runtime context for script execution
//!
//! Provides access to Bevy resources and state

use super::api::{ColorContext, FontContext, ScriptContext, TerminalContext, WindowContext};
use bevy::prelude::*;
use scarab_config::ScarabConfig;

/// Bevy resource that provides context to scripts
#[derive(Resource, Clone)]
pub struct RuntimeContext {
    context: ScriptContext,
}

impl RuntimeContext {
    /// Create a new runtime context from Bevy resources
    pub fn from_resources(config: &ScarabConfig, window: &Window) -> Self {
        Self {
            context: ScriptContext {
                colors: Self::build_color_context(config),
                fonts: Self::build_font_context(config),
                window: Self::build_window_context(window),
                terminal: Self::build_terminal_context(config),
            },
        }
    }

    /// Get a reference to the script context
    pub fn context(&self) -> &ScriptContext {
        &self.context
    }

    /// Update the context (called when resources change)
    pub fn update(&mut self, config: &ScarabConfig, window: &Window) {
        self.context.colors = Self::build_color_context(config);
        self.context.fonts = Self::build_font_context(config);
        self.context.window = Self::build_window_context(window);
        self.context.terminal = Self::build_terminal_context(config);
    }

    fn build_color_context(config: &ScarabConfig) -> ColorContext {
        let palette = vec![
            parse_color(&config.colors.palette.black),
            parse_color(&config.colors.palette.red),
            parse_color(&config.colors.palette.green),
            parse_color(&config.colors.palette.yellow),
            parse_color(&config.colors.palette.blue),
            parse_color(&config.colors.palette.magenta),
            parse_color(&config.colors.palette.cyan),
            parse_color(&config.colors.palette.white),
            parse_color(&config.colors.palette.bright_black),
            parse_color(&config.colors.palette.bright_red),
            parse_color(&config.colors.palette.bright_green),
            parse_color(&config.colors.palette.bright_yellow),
            parse_color(&config.colors.palette.bright_blue),
            parse_color(&config.colors.palette.bright_magenta),
            parse_color(&config.colors.palette.bright_cyan),
            parse_color(&config.colors.palette.bright_white),
        ];

        ColorContext {
            foreground: config
                .colors
                .foreground
                .as_ref()
                .map(|s| parse_color(s))
                .unwrap_or(Color::srgb(0.97, 0.97, 0.95)),
            background: config
                .colors
                .background
                .as_ref()
                .map(|s| parse_color(s))
                .unwrap_or(Color::srgb(0.13, 0.13, 0.17)),
            cursor: config
                .colors
                .cursor
                .as_ref()
                .map(|s| parse_color(s))
                .unwrap_or(Color::srgb(0.97, 0.97, 0.95)),
            selection_bg: config
                .colors
                .selection_background
                .as_ref()
                .map(|s| parse_color(s))
                .unwrap_or(Color::srgba(0.38, 0.45, 0.64, 0.3)),
            selection_fg: config
                .colors
                .selection_foreground
                .as_ref()
                .map(|s| parse_color(s))
                .unwrap_or(Color::srgb(0.97, 0.97, 0.95)),
            palette,
        }
    }

    fn build_font_context(config: &ScarabConfig) -> FontContext {
        FontContext {
            family: config.font.family.clone(),
            size: config.font.size,
            line_height: config.font.line_height,
        }
    }

    fn build_window_context(window: &Window) -> WindowContext {
        WindowContext {
            width: window.width(),
            height: window.height(),
            scale_factor: window.scale_factor(),
            title: window.title.clone(),
        }
    }

    fn build_terminal_context(config: &ScarabConfig) -> TerminalContext {
        TerminalContext {
            rows: config.terminal.rows,
            cols: config.terminal.columns,
            scrollback_lines: config.terminal.scrollback_lines,
        }
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        // Create a default context (used when resources aren't available yet)
        Self {
            context: ScriptContext {
                colors: ColorContext {
                    foreground: Color::srgb(0.97, 0.97, 0.95),
                    background: Color::srgb(0.13, 0.13, 0.17),
                    cursor: Color::srgb(0.97, 0.97, 0.95),
                    selection_bg: Color::srgba(0.38, 0.45, 0.64, 0.3),
                    selection_fg: Color::srgb(0.97, 0.97, 0.95),
                    palette: vec![Color::BLACK; 16],
                },
                fonts: FontContext {
                    family: "JetBrains Mono".to_string(),
                    size: 14.0,
                    line_height: 1.2,
                },
                window: WindowContext {
                    width: 800.0,
                    height: 600.0,
                    scale_factor: 1.0,
                    title: "Scarab Terminal".to_string(),
                },
                terminal: TerminalContext {
                    rows: 24,
                    cols: 80,
                    scrollback_lines: 10000,
                },
            },
        }
    }
}

/// Parse a hex color string to Bevy Color
fn parse_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 && hex.len() != 8 {
        warn!("Invalid hex color: #{}, using fallback", hex);
        return Color::WHITE;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    let a = if hex.len() == 8 {
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(255)
    } else {
        255
    };

    Color::srgba_u8(r, g, b, a)
}

/// System to initialize the runtime context
pub fn initialize_context(
    mut commands: Commands,
    config: Res<ScarabConfig>,
    window: Query<&Window>,
) {
    if let Ok(window) = window.get_single() {
        let context = RuntimeContext::from_resources(&config, window);
        commands.insert_resource(context);
        info!("Script runtime context initialized");
    }
}

/// System to update the runtime context when resources change
pub fn update_context(
    mut context: ResMut<RuntimeContext>,
    config: Res<ScarabConfig>,
    window: Query<&Window>,
) {
    // Only update if config changed
    if !config.is_changed() {
        return;
    }

    if let Ok(window) = window.get_single() {
        context.update(&config, window);
        debug!("Script runtime context updated");
    }
}
