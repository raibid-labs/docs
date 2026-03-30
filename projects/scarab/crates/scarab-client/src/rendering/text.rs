// Text rendering system for terminal grid

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};
use scarab_protocol::{Cell, SharedState, GRID_HEIGHT, GRID_WIDTH};
use std::collections::HashSet;

use super::atlas::{AtlasRect, GlyphAtlas, GlyphKey};
use super::config::{color, FontConfig, TextAttributes};

/// Text renderer resource managing fonts and glyph caching
#[derive(Resource)]
pub struct TextRenderer {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub atlas: GlyphAtlas,
    pub config: FontConfig,
    pub cell_width: f32,
    pub cell_height: f32,
}

impl TextRenderer {
    /// Create a new text renderer
    pub fn new(config: FontConfig, images: &mut ResMut<Assets<Image>>) -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let atlas = GlyphAtlas::new(images);

        let (cell_width, cell_height) = config.cell_dimensions();

        Self {
            font_system,
            swash_cache,
            atlas,
            config,
            cell_width,
            cell_height,
        }
    }

    /// Update font size and recalculate cell dimensions
    pub fn set_font_size(&mut self, size: f32) {
        self.config.size = size;
        let (width, height) = self.config.cell_dimensions();
        self.cell_width = width;
        self.cell_height = height;
    }

    /// Get actual font metrics for precise cell sizing
    pub fn update_metrics(&mut self) {
        let mut buffer = Buffer::new(
            &mut self.font_system,
            Metrics::new(self.config.size, self.config.size * self.config.line_height),
        );

        buffer.set_size(&mut self.font_system, 100.0, 100.0);
        buffer.set_text(&mut self.font_system, "M", Attrs::new(), Shaping::Advanced);

        // Get actual glyph dimensions
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                self.cell_width = glyph.w;
                self.cell_height = self.config.size * self.config.line_height;
                return;
            }
        }
    }
}

/// Dirty region tracking for optimized mesh updates
#[derive(Debug, Clone, Default)]
pub struct DirtyRegion {
    /// Set of dirty cell indices
    dirty_cells: HashSet<usize>,
    /// Whether the entire grid is dirty
    full_redraw: bool,
}

impl DirtyRegion {
    pub fn new() -> Self {
        Self {
            dirty_cells: HashSet::new(),
            full_redraw: true, // Start with full redraw
        }
    }

    pub fn mark_dirty(&mut self, index: usize) {
        self.dirty_cells.insert(index);
    }

    pub fn mark_full_redraw(&mut self) {
        self.full_redraw = true;
        self.dirty_cells.clear();
    }

    pub fn is_dirty(&self, index: usize) -> bool {
        self.full_redraw || self.dirty_cells.contains(&index)
    }

    pub fn clear(&mut self) {
        self.dirty_cells.clear();
        self.full_redraw = false;
    }

    pub fn is_empty(&self) -> bool {
        !self.full_redraw && self.dirty_cells.is_empty()
    }
}

/// Terminal mesh component
#[derive(Component)]
pub struct TerminalMesh {
    pub dirty_region: DirtyRegion,
    pub last_sequence: u64,
    pub mesh_handle: Handle<Mesh>, // Store handle in component
}

impl TerminalMesh {
    pub fn new(mesh_handle: Handle<Mesh>) -> Self {
        Self {
            dirty_region: DirtyRegion::new(),
            last_sequence: 0,
            mesh_handle,
        }
    }
}

impl Default for TerminalMesh {
    fn default() -> Self {
        Self {
            dirty_region: DirtyRegion::new(),
            last_sequence: 0,
            mesh_handle: Handle::default(),
        }
    }
}

/// Generate mesh from terminal grid state
/// Note: Uses separate attribute arrays instead of a vertex struct
/// for better compatibility with Bevy's mesh API
pub fn generate_terminal_mesh(
    state: &SharedState,
    renderer: &mut TextRenderer,
    dirty_region: &DirtyRegion,
    images: &mut ResMut<Assets<Image>>,
) -> Mesh {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let grid_width = GRID_WIDTH as f32;
    let grid_height = GRID_HEIGHT as f32;

    // Calculate screen positioning (center grid)
    let start_x = -(grid_width * renderer.cell_width) / 2.0;
    let start_y = (grid_height * renderer.cell_height) / 2.0;

    let mut vertex_index = 0u32;

    // Iterate through all cells
    for (idx, cell) in state.cells.iter().enumerate() {
        // Skip if not dirty (optimization)
        if !dirty_region.is_empty() && !dirty_region.is_dirty(idx) {
            continue;
        }

        let row = idx / GRID_WIDTH;
        let col = idx % GRID_WIDTH;

        let x = start_x + (col as f32 * renderer.cell_width);
        let y = start_y - (row as f32 * renderer.cell_height);

        // Background quad
        if cell.bg != 0 {
            add_background_quad(
                &mut positions,
                &mut uvs,
                &mut colors,
                &mut indices,
                &mut vertex_index,
                x,
                y,
                renderer.cell_width,
                renderer.cell_height,
                cell.bg,
            );
        }

        // Foreground glyph
        if cell.char_codepoint != 0 && cell.char_codepoint != 32 {
            // Space character, skip rendering
            let _ = render_glyph(
                cell,
                renderer,
                &mut positions,
                &mut uvs,
                &mut colors,
                &mut indices,
                &mut vertex_index,
                x,
                y,
            );
        }
    }

    // Update atlas texture if dirty
    renderer.atlas.update_texture(images);

    // Build mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Add a background quad for a cell
fn add_background_quad(
    positions: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    vertex_index: &mut u32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    bg_color: u32,
) {
    let bg = color::from_rgba(bg_color);
    let color_array = bg.to_srgba().to_f32_array();

    // Add four vertices (quad)
    positions.extend_from_slice(&[
        [x, y, 0.0],
        [x + width, y, 0.0],
        [x + width, y - height, 0.0],
        [x, y - height, 0.0],
    ]);

    // UVs don't matter for solid color
    uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

    // All vertices have same background color
    for _ in 0..4 {
        colors.push(color_array);
    }

    // Two triangles (counter-clockwise winding)
    indices.extend_from_slice(&[
        *vertex_index,
        *vertex_index + 1,
        *vertex_index + 2,
        *vertex_index,
        *vertex_index + 2,
        *vertex_index + 3,
    ]);

    *vertex_index += 4;
}

/// Render a glyph quad
fn render_glyph(
    cell: &Cell,
    renderer: &mut TextRenderer,
    positions: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    vertex_index: &mut u32,
    x: f32,
    y: f32,
) -> Option<AtlasRect> {
    // Get character from codepoint
    let ch = char::from_u32(cell.char_codepoint)?;

    // Parse text attributes
    let attrs = TextAttributes::from_flags(cell.flags);

    // Get the glyph cache key
    // Use a block to limit the scope of the buffer borrow on font_system
    let glyph_key = {
        // Create cosmic-text buffer to get glyph info
        let metrics = Metrics::new(
            renderer.config.size,
            renderer.config.size * renderer.config.line_height,
        );
        let mut buffer = Buffer::new(&mut renderer.font_system, metrics);

        // Build attrs with bold/italic
        let mut cosmic_attrs = Attrs::new();

        if attrs.bold {
            cosmic_attrs = cosmic_attrs.weight(cosmic_text::Weight::BOLD);
        }
        if attrs.italic {
            cosmic_attrs = cosmic_attrs.style(cosmic_text::Style::Italic);
        }

        buffer.set_text(
            &mut renderer.font_system,
            &ch.to_string(),
            cosmic_attrs,
            Shaping::Advanced,
        );

        let mut key = None;
        for run in buffer.layout_runs() {
            for glyph in run.glyphs {
                // Create GlyphKey from glyph info (cosmic-text API changed)
                key = Some(GlyphKey {
                    font_id: glyph.font_id,
                    glyph_id: glyph.glyph_id,
                    font_size_bits: glyph.font_size.to_bits(),
                });
                break;
            }
        }
        key
    };

    let glyph_key = glyph_key?;

    // Get or cache the glyph in atlas
    let atlas_rect = renderer.atlas.get_or_cache(
        &mut renderer.font_system,
        glyph_key,
        &mut renderer.swash_cache,
    )?;

    // Get UV coordinates
    let uv_rect = atlas_rect.uv_rect();

    // Get foreground color (with dim attribute)
    let mut fg = color::from_rgba(cell.fg);
    if attrs.dim {
        let [r, g, b, a] = fg.to_srgba().to_f32_array();
        fg = Color::srgba(r * 0.5, g * 0.5, b * 0.5, a);
    }

    // Handle reverse video
    if attrs.reverse {
        fg = color::from_rgba(cell.bg);
    }

    let fg_array = fg.to_srgba().to_f32_array();

    // Add glyph quad
    let glyph_width = atlas_rect.width as f32;
    let glyph_height = atlas_rect.height as f32;

    positions.extend_from_slice(&[
        [x, y, 0.1], // Slightly above background
        [x + glyph_width, y, 0.1],
        [x + glyph_width, y - glyph_height, 0.1],
        [x, y - glyph_height, 0.1],
    ]);

    uvs.extend_from_slice(&[
        [uv_rect[0], uv_rect[1]],
        [uv_rect[2], uv_rect[1]],
        [uv_rect[2], uv_rect[3]],
        [uv_rect[0], uv_rect[3]],
    ]);

    for _ in 0..4 {
        colors.push(fg_array);
    }

    indices.extend_from_slice(&[
        *vertex_index,
        *vertex_index + 1,
        *vertex_index + 2,
        *vertex_index,
        *vertex_index + 2,
        *vertex_index + 3,
    ]);

    *vertex_index += 4;

    // Handle underline
    if attrs.underline {
        add_underline_quad(
            positions,
            uvs,
            colors,
            indices,
            vertex_index,
            x,
            y - renderer.cell_height + 2.0,
            renderer.cell_width,
            1.0,
            cell.fg,
        );
    }

    // Handle strikethrough
    if attrs.strikethrough {
        add_underline_quad(
            positions,
            uvs,
            colors,
            indices,
            vertex_index,
            x,
            y - renderer.cell_height / 2.0,
            renderer.cell_width,
            1.0,
            cell.fg,
        );
    }

    Some(atlas_rect)
}

/// Add underline/strikethrough line
fn add_underline_quad(
    positions: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    vertex_index: &mut u32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color_u32: u32,
) {
    let color = color::from_rgba(color_u32);
    let color_array = color.to_srgba().to_f32_array();

    positions.extend_from_slice(&[
        [x, y, 0.15], // Above glyph
        [x + width, y, 0.15],
        [x + width, y - height, 0.15],
        [x, y - height, 0.15],
    ]);

    uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

    for _ in 0..4 {
        colors.push(color_array);
    }

    indices.extend_from_slice(&[
        *vertex_index,
        *vertex_index + 1,
        *vertex_index + 2,
        *vertex_index,
        *vertex_index + 2,
        *vertex_index + 3,
    ]);

    *vertex_index += 4;
}

/// System to update terminal mesh when state changes
pub fn update_terminal_mesh_system(
    mut renderer: ResMut<TextRenderer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut query: Query<&mut TerminalMesh>,
    state_reader: Res<crate::integration::SharedMemoryReader>,
) {
    let state = unsafe { &*(state_reader.shmem.0.as_ptr() as *const SharedState) };

    for mut terminal_mesh in query.iter_mut() {
        // Check if state changed
        let current_seq = state.sequence_number;
        if current_seq != terminal_mesh.last_sequence {
            terminal_mesh.dirty_region.mark_full_redraw();
            terminal_mesh.last_sequence = current_seq;
        }

        // Skip if nothing to update
        if terminal_mesh.dirty_region.is_empty() {
            continue;
        }

        // Generate new mesh
        let new_mesh = generate_terminal_mesh(
            state,
            &mut renderer,
            &terminal_mesh.dirty_region,
            &mut images,
        );

        // Update mesh asset using the handle stored in the component
        if let Some(mesh) = meshes.get_mut(&terminal_mesh.mesh_handle) {
            *mesh = new_mesh;
        }

        // Clear dirty region
        terminal_mesh.dirty_region.clear();
    }
}
