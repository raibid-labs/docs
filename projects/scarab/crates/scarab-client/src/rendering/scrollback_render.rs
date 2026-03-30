// Scrollback rendering integration
// Renders historical lines from scrollback buffer instead of live grid

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use scarab_protocol::{Cell, GRID_HEIGHT, GRID_WIDTH};

use crate::terminal::scrollback::{ScrollbackBuffer, ScrollbackState, ScrollbackLine};
use super::atlas::GlyphKey;
use super::config::{color, TextAttributes};
use super::text::TextRenderer;

/// Generate mesh for scrollback view
/// Combines scrollback lines with live view at the bottom
pub fn generate_scrollback_mesh(
    scrollback: &ScrollbackBuffer,
    scrollback_state: &ScrollbackState,
    renderer: &mut TextRenderer,
    images: &mut ResMut<Assets<Image>>,
) -> Mesh {
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();
    let mut vertex_index = 0u32;

    let grid_width = GRID_WIDTH as f32;
    let grid_height = GRID_HEIGHT as f32;

    // Calculate screen positioning (center grid)
    let start_x = -(grid_width * renderer.cell_width) / 2.0;
    let start_y = (grid_height * renderer.cell_height) / 2.0;

    // Get scrollback lines to display
    let visible_lines = scrollback.get_visible_lines(GRID_HEIGHT);

    // Render scrollback lines
    for (row_offset, line) in visible_lines.iter().enumerate() {
        let y_pos = start_y - (row_offset as f32 * renderer.cell_height);

        for (col, cell) in line.cells.iter().enumerate() {
            if col >= GRID_WIDTH {
                break; // Don't overflow grid width
            }

            let x_pos = start_x + (col as f32 * renderer.cell_width);

            // Render cell background
            if cell.bg != 0 {
                add_background_quad(
                    &mut positions,
                    &mut uvs,
                    &mut colors,
                    &mut indices,
                    &mut vertex_index,
                    x_pos,
                    y_pos,
                    renderer.cell_width,
                    renderer.cell_height,
                    cell.bg,
                );
            }

            // Render cell glyph
            if cell.char_codepoint != 0 && cell.char_codepoint != 32 {
                render_scrollback_glyph(
                    cell,
                    renderer,
                    &mut positions,
                    &mut uvs,
                    &mut colors,
                    &mut indices,
                    &mut vertex_index,
                    x_pos,
                    y_pos,
                );
            }
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

/// Add background quad for a cell
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

    positions.extend_from_slice(&[
        [x, y, 0.0],
        [x + width, y, 0.0],
        [x + width, y - height, 0.0],
        [x, y - height, 0.0],
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

/// Render a glyph from scrollback
fn render_scrollback_glyph(
    cell: &Cell,
    renderer: &mut TextRenderer,
    positions: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
    vertex_index: &mut u32,
    x: f32,
    y: f32,
) {
    // Get character from codepoint
    let ch = match char::from_u32(cell.char_codepoint) {
        Some(c) => c,
        None => return,
    };

    // Parse text attributes
    let attrs = TextAttributes::from_flags(cell.flags);

    // Get or cache the glyph
    use cosmic_text::{Attrs, Buffer, Metrics, Shaping};

    let glyph_key = {
        let metrics = Metrics::new(
            renderer.config.size,
            renderer.config.size * renderer.config.line_height,
        );
        let mut buffer = Buffer::new(&mut renderer.font_system, metrics);

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

    let glyph_key = match glyph_key {
        Some(k) => k,
        None => return,
    };

    // Get atlas rect
    let atlas_rect = match renderer.atlas.get_or_cache(
        &mut renderer.font_system,
        glyph_key,
        &mut renderer.swash_cache,
    ) {
        Some(rect) => rect,
        None => return,
    };

    let uv_rect = atlas_rect.uv_rect();

    // Get foreground color
    let mut fg = color::from_rgba(cell.fg);
    if attrs.dim {
        let [r, g, b, a] = fg.to_srgba().to_f32_array();
        fg = Color::srgba(r * 0.5, g * 0.5, b * 0.5, a);
    }
    if attrs.reverse {
        fg = color::from_rgba(cell.bg);
    }

    let fg_array = fg.to_srgba().to_f32_array();

    // Add glyph quad
    let glyph_width = atlas_rect.width as f32;
    let glyph_height = atlas_rect.height as f32;

    positions.extend_from_slice(&[
        [x, y, 0.1],
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
        add_background_quad(
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
        add_background_quad(
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
}
