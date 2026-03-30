// Font configuration and management

use serde::{Deserialize, Serialize};

/// Font configuration for the text renderer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    /// Primary font family name
    pub family: String,

    /// Font size in points
    pub size: f32,

    /// Fallback font families (in order of preference)
    pub fallback: Vec<String>,

    /// Line height multiplier
    pub line_height: f32,

    /// Letter spacing adjustment
    pub letter_spacing: f32,

    /// Enable font hinting
    pub hinting: bool,

    /// Enable subpixel positioning
    pub subpixel: bool,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "JetBrains Mono".to_string(),
            size: 14.0,
            fallback: vec![
                "Fira Code".to_string(),
                "Cascadia Code".to_string(),
                "DejaVu Sans Mono".to_string(),
                "Noto Sans Mono".to_string(),
            ],
            line_height: 1.2,
            letter_spacing: 0.0,
            hinting: true,
            subpixel: true,
        }
    }
}

impl FontConfig {
    /// Create a new font configuration
    pub fn new(family: impl Into<String>, size: f32) -> Self {
        Self {
            family: family.into(),
            size,
            ..Default::default()
        }
    }

    /// Get all font families (primary + fallbacks)
    pub fn all_families(&self) -> Vec<&str> {
        let mut families = vec![self.family.as_str()];
        families.extend(self.fallback.iter().map(|s| s.as_str()));
        families
    }

    /// Calculate cell dimensions based on font metrics
    pub fn cell_dimensions(&self) -> (f32, f32) {
        // Approximate cell dimensions (will be refined with actual font metrics)
        let width = self.size * 0.6; // Monospace width ratio
        let height = self.size * self.line_height;
        (width, height)
    }
}

/// Text attributes flags (matches SharedState Cell flags)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextAttributes {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim: bool,
    pub reverse: bool,
}

impl TextAttributes {
    pub fn from_flags(flags: u8) -> Self {
        Self {
            bold: flags & 0x01 != 0,
            italic: flags & 0x02 != 0,
            underline: flags & 0x04 != 0,
            strikethrough: flags & 0x08 != 0,
            dim: flags & 0x10 != 0,
            reverse: flags & 0x20 != 0,
        }
    }

    pub fn to_flags(&self) -> u8 {
        let mut flags = 0u8;
        if self.bold {
            flags |= 0x01;
        }
        if self.italic {
            flags |= 0x02;
        }
        if self.underline {
            flags |= 0x04;
        }
        if self.strikethrough {
            flags |= 0x08;
        }
        if self.dim {
            flags |= 0x10;
        }
        if self.reverse {
            flags |= 0x20;
        }
        flags
    }
}

impl Default for TextAttributes {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            dim: false,
            reverse: false,
        }
    }
}

/// Color helper functions for converting between formats
pub mod color {
    use bevy::prelude::{Color, ColorToComponents};

    /// Convert u32 RGBA to Bevy Color
    pub fn from_rgba(rgba: u32) -> Color {
        let r = ((rgba >> 24) & 0xFF) as f32 / 255.0;
        let g = ((rgba >> 16) & 0xFF) as f32 / 255.0;
        let b = ((rgba >> 8) & 0xFF) as f32 / 255.0;
        let a = (rgba & 0xFF) as f32 / 255.0;
        Color::srgba(r, g, b, a)
    }

    /// Convert Bevy Color to u32 RGBA
    pub fn to_rgba(color: Color) -> u32 {
        let [r, g, b, a] = color.to_srgba().to_f32_array();
        let r = (r * 255.0) as u32;
        let g = (g * 255.0) as u32;
        let b = (b * 255.0) as u32;
        let a = (a * 255.0) as u32;
        (r << 24) | (g << 16) | (b << 8) | a
    }
}
