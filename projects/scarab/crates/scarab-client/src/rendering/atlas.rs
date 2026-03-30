// GPU texture atlas for glyph caching

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use cosmic_text::{
    CacheKey, CacheKeyFlags, FontSystem, SubpixelBin, SwashCache, SwashContent, SwashImage,
};
use std::collections::HashMap;

/// Maximum atlas texture size (4096x4096 for compatibility)
pub const ATLAS_SIZE: u32 = 4096;

/// Padding between glyphs in the atlas
const GLYPH_PADDING: u32 = 2;

/// Key for identifying unique glyphs in the atlas
/// Matches cosmic-text's glyph identification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_id: cosmic_text::fontdb::ID,
    pub glyph_id: u16,
    pub font_size_bits: u32,
}

impl From<CacheKey> for GlyphKey {
    fn from(cache_key: CacheKey) -> Self {
        Self {
            font_id: cache_key.font_id,
            glyph_id: cache_key.glyph_id,
            font_size_bits: cache_key.font_size_bits,
        }
    }
}

/// Rectangle in the atlas texture (UV coordinates)
#[derive(Debug, Clone, Copy)]
pub struct AtlasRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl AtlasRect {
    /// Get UV coordinates normalized to [0, 1]
    pub fn uv_rect(&self) -> [f32; 4] {
        let atlas_size = ATLAS_SIZE as f32;
        [
            self.x as f32 / atlas_size,
            self.y as f32 / atlas_size,
            (self.x + self.width) as f32 / atlas_size,
            (self.y + self.height) as f32 / atlas_size,
        ]
    }
}

/// Glyph atlas for caching rasterized glyphs
pub struct GlyphAtlas {
    /// Texture handle for the atlas
    pub texture: Handle<Image>,

    /// Map from glyph key to atlas position
    glyph_positions: HashMap<GlyphKey, AtlasRect>,

    /// Current packing position
    current_x: u32,
    current_y: u32,
    current_row_height: u32,

    /// Raw texture data (RGBA8)
    texture_data: Vec<u8>,

    /// Whether the texture needs updating
    dirty: bool,
}

impl GlyphAtlas {
    /// Create a new glyph atlas
    pub fn new(images: &mut ResMut<Assets<Image>>) -> Self {
        // Create initial atlas texture
        let size = Extent3d {
            width: ATLAS_SIZE,
            height: ATLAS_SIZE,
            depth_or_array_layers: 1,
        };

        let mut image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0, 0, 0, 0], // Transparent black
            TextureFormat::Rgba8UnormSrgb,
            bevy::render::render_asset::RenderAssetUsages::default(),
        );

        image.texture_descriptor.usage =
            bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
                | bevy::render::render_resource::TextureUsages::COPY_DST;

        let texture = images.add(image);

        Self {
            texture,
            glyph_positions: HashMap::new(),
            current_x: GLYPH_PADDING,
            current_y: GLYPH_PADDING,
            current_row_height: 0,
            texture_data: vec![0; (ATLAS_SIZE * ATLAS_SIZE * 4) as usize],
            dirty: false,
        }
    }

    /// Get or cache a glyph in the atlas
    pub fn get_or_cache(
        &mut self,
        font_system: &mut FontSystem,
        glyph_key: GlyphKey,
        swash_cache: &mut SwashCache,
    ) -> Option<AtlasRect> {
        // Check if already cached
        if let Some(rect) = self.glyph_positions.get(&glyph_key) {
            return Some(*rect);
        }

        // Rasterize the glyph using cosmic-text
        // Convert GlyphKey back to CacheKey for swash
        let cache_key = CacheKey {
            font_id: glyph_key.font_id,
            glyph_id: glyph_key.glyph_id,
            font_size_bits: glyph_key.font_size_bits,
            x_bin: SubpixelBin::Zero,
            y_bin: SubpixelBin::Zero,
            flags: CacheKeyFlags::empty(),
        };

        let image = swash_cache.get_image(font_system, cache_key).as_ref()?;

        // Check if we have space in the atlas
        let glyph_width = image.placement.width as u32;
        let glyph_height = image.placement.height as u32;

        if !self.can_fit(glyph_width, glyph_height) {
            warn!("Atlas full! Consider implementing dynamic atlas expansion");
            return None;
        }

        // Pack the glyph
        let rect = self.pack_glyph(glyph_width, glyph_height);

        // Copy glyph data to atlas
        self.copy_glyph_data(image, &rect);

        // Cache the position
        self.glyph_positions.insert(glyph_key, rect);
        self.dirty = true;

        Some(rect)
    }

    /// Check if a glyph can fit in the atlas
    fn can_fit(&self, width: u32, height: u32) -> bool {
        let padded_width = width + GLYPH_PADDING * 2;
        let padded_height = height + GLYPH_PADDING * 2;

        // Try current row
        if self.current_x + padded_width <= ATLAS_SIZE {
            return true;
        }

        // Try next row
        let next_y = self.current_y + self.current_row_height + GLYPH_PADDING;
        next_y + padded_height <= ATLAS_SIZE
    }

    /// Pack a glyph into the atlas and return its rect
    fn pack_glyph(&mut self, width: u32, height: u32) -> AtlasRect {
        let padded_width = width + GLYPH_PADDING * 2;
        let padded_height = height + GLYPH_PADDING * 2;

        // Check if we need to move to next row
        if self.current_x + padded_width > ATLAS_SIZE {
            self.current_x = GLYPH_PADDING;
            self.current_y += self.current_row_height + GLYPH_PADDING;
            self.current_row_height = 0;
        }

        let rect = AtlasRect {
            x: self.current_x,
            y: self.current_y,
            width,
            height,
        };

        self.current_x += padded_width;
        self.current_row_height = self.current_row_height.max(padded_height);

        rect
    }

    /// Copy glyph image data to the atlas texture
    fn copy_glyph_data(&mut self, image: &SwashImage, rect: &AtlasRect) {
        let data = &image.data;

        for y in 0..rect.height {
            for x in 0..rect.width {
                let src_idx = (y * rect.width + x) as usize;
                let dst_idx = ((rect.y + y) * ATLAS_SIZE + (rect.x + x)) as usize * 4;

                // Convert alpha mask to RGBA - cosmic-text 0.11 uses enum variants
                let alpha = match image.content {
                    SwashContent::Mask => {
                        if src_idx < data.len() {
                            data[src_idx]
                        } else {
                            0
                        }
                    }
                    SwashContent::Color => {
                        // For colored emoji, copy RGBA directly
                        if src_idx * 4 + 3 < data.len() {
                            let offset = src_idx * 4;
                            self.texture_data[dst_idx] = data[offset];
                            self.texture_data[dst_idx + 1] = data[offset + 1];
                            self.texture_data[dst_idx + 2] = data[offset + 2];
                            data[offset + 3]
                        } else {
                            0
                        }
                    }
                    SwashContent::SubpixelMask => {
                        // Use red channel for subpixel rendering
                        if src_idx * 3 < data.len() {
                            data[src_idx * 3]
                        } else {
                            0
                        }
                    }
                };

                // Set white color with alpha (for monochrome glyphs)
                if matches!(image.content, SwashContent::Mask) {
                    self.texture_data[dst_idx] = 255;
                    self.texture_data[dst_idx + 1] = 255;
                    self.texture_data[dst_idx + 2] = 255;
                }
                self.texture_data[dst_idx + 3] = alpha;
            }
        }
    }

    /// Update the GPU texture if dirty
    pub fn update_texture(&mut self, images: &mut ResMut<Assets<Image>>) {
        if !self.dirty {
            return;
        }

        if let Some(image) = images.get_mut(&self.texture) {
            image.data.copy_from_slice(&self.texture_data);
        }

        self.dirty = false;
    }

    /// Clear the atlas (for debugging/testing)
    pub fn clear(&mut self) {
        self.glyph_positions.clear();
        self.current_x = GLYPH_PADDING;
        self.current_y = GLYPH_PADDING;
        self.current_row_height = 0;
        self.texture_data.fill(0);
        self.dirty = true;
    }

    /// Get atlas statistics
    pub fn stats(&self) -> AtlasStats {
        let used_height = self.current_y + self.current_row_height;
        let total_pixels = ATLAS_SIZE * ATLAS_SIZE;
        let used_pixels = used_height * ATLAS_SIZE;

        AtlasStats {
            glyph_count: self.glyph_positions.len(),
            used_height,
            total_height: ATLAS_SIZE,
            occupancy: used_pixels as f32 / total_pixels as f32,
            memory_mb: (self.texture_data.len() as f32) / (1024.0 * 1024.0),
        }
    }
}

/// Atlas statistics for monitoring
#[derive(Debug, Clone, Copy)]
pub struct AtlasStats {
    pub glyph_count: usize,
    pub used_height: u32,
    pub total_height: u32,
    pub occupancy: f32,
    pub memory_mb: f32,
}
