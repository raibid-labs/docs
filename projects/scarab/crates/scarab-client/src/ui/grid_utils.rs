// Grid coordinate utilities for UI positioning
// Provides conversion functions between terminal grid coordinates and pixel positions

use bevy::prelude::*;
use crate::rendering::text::TextRenderer;
use scarab_protocol::{GRID_WIDTH, GRID_HEIGHT};

/// Convert grid column and row to pixel coordinates
///
/// # Arguments
/// * `col` - Column index (0-indexed, left to right)
/// * `row` - Row index (0-indexed, top to bottom)
/// * `cell_width` - Width of a single cell in pixels
/// * `cell_height` - Height of a single cell in pixels
///
/// # Returns
/// Pixel position (x, y) in Bevy screen space coordinates
/// - X: horizontal position (left to right)
/// - Y: vertical position (Bevy uses inverted Y, so negative is downward)
/// - Origin is centered on the grid
pub fn grid_to_pixel(col: u16, row: u16, cell_width: f32, cell_height: f32) -> Vec2 {
    let grid_width = GRID_WIDTH as f32;
    let grid_height = GRID_HEIGHT as f32;

    // Calculate screen positioning (grid is centered)
    let start_x = -(grid_width * cell_width) / 2.0;
    let start_y = (grid_height * cell_height) / 2.0;

    // Calculate pixel position for this cell
    let x = start_x + (col as f32 * cell_width);
    let y = start_y - (row as f32 * cell_height);

    Vec2::new(x, y)
}

/// Convert grid coordinates to pixel coordinates using TextRenderer metrics
///
/// # Arguments
/// * `col` - Column index (0-indexed)
/// * `row` - Row index (0-indexed)
/// * `renderer` - TextRenderer resource containing font metrics
///
/// # Returns
/// Pixel position Vec2 in screen space
pub fn grid_to_pixel_with_renderer(col: u16, row: u16, renderer: &TextRenderer) -> Vec2 {
    grid_to_pixel(col, row, renderer.cell_width, renderer.cell_height)
}

/// Convert pixel coordinates back to grid coordinates
///
/// # Arguments
/// * `pixel_pos` - Pixel position Vec2
/// * `cell_width` - Width of a single cell in pixels
/// * `cell_height` - Height of a single cell in pixels
///
/// # Returns
/// Grid coordinates (col, row) or None if out of bounds
pub fn pixel_to_grid(pixel_pos: Vec2, cell_width: f32, cell_height: f32) -> Option<(u16, u16)> {
    let grid_width = GRID_WIDTH as f32;
    let grid_height = GRID_HEIGHT as f32;

    let start_x = -(grid_width * cell_width) / 2.0;
    let start_y = (grid_height * cell_height) / 2.0;

    // Calculate grid coordinates
    let col = ((pixel_pos.x - start_x) / cell_width) as i32;
    let row = ((start_y - pixel_pos.y) / cell_height) as i32;

    // Check bounds
    if col < 0 || col >= GRID_WIDTH as i32 || row < 0 || row >= GRID_HEIGHT as i32 {
        return None;
    }

    Some((col as u16, row as u16))
}

/// Get the bounding box for a cell in pixel coordinates
///
/// # Arguments
/// * `col` - Column index
/// * `row` - Row index
/// * `cell_width` - Width of a single cell
/// * `cell_height` - Height of a single cell
///
/// # Returns
/// Rect representing the cell bounds (position and size)
pub fn grid_cell_bounds(col: u16, row: u16, cell_width: f32, cell_height: f32) -> Rect {
    let pos = grid_to_pixel(col, row, cell_width, cell_height);

    Rect {
        min: Vec2::new(pos.x, pos.y - cell_height),
        max: Vec2::new(pos.x + cell_width, pos.y),
    }
}

/// Get the center point of a cell in pixel coordinates
///
/// # Arguments
/// * `col` - Column index
/// * `row` - Row index
/// * `cell_width` - Width of a single cell
/// * `cell_height` - Height of a single cell
///
/// # Returns
/// Vec2 representing the cell's center point
pub fn grid_cell_center(col: u16, row: u16, cell_width: f32, cell_height: f32) -> Vec2 {
    let pos = grid_to_pixel(col, row, cell_width, cell_height);
    Vec2::new(pos.x + cell_width / 2.0, pos.y - cell_height / 2.0)
}

/// Calculate pixel dimensions for a rectangular grid region
///
/// # Arguments
/// * `start_col` - Starting column
/// * `start_row` - Starting row
/// * `end_col` - Ending column (inclusive)
/// * `end_row` - Ending row (inclusive)
/// * `cell_width` - Width of a single cell
/// * `cell_height` - Height of a single cell
///
/// # Returns
/// (position, size) tuple for the region
pub fn grid_region_bounds(
    start_col: u16,
    start_row: u16,
    end_col: u16,
    end_row: u16,
    cell_width: f32,
    cell_height: f32,
) -> (Vec2, Vec2) {
    let pos = grid_to_pixel(start_col, start_row, cell_width, cell_height);

    let width = (end_col - start_col + 1) as f32 * cell_width;
    let height = (end_row - start_row + 1) as f32 * cell_height;

    (pos, Vec2::new(width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_to_pixel() {
        let cell_width = 8.0;
        let cell_height = 16.0;

        // Test origin (0,0)
        let pos = grid_to_pixel(0, 0, cell_width, cell_height);
        let expected_x = -(GRID_WIDTH as f32 * cell_width) / 2.0;
        let expected_y = (GRID_HEIGHT as f32 * cell_height) / 2.0;
        assert_eq!(pos.x, expected_x);
        assert_eq!(pos.y, expected_y);

        // Test middle cell
        let mid_col = (GRID_WIDTH / 2) as u16;
        let mid_row = (GRID_HEIGHT / 2) as u16;
        let pos = grid_to_pixel(mid_col, mid_row, cell_width, cell_height);
        assert!(pos.x.abs() < cell_width); // Near center X
        assert!(pos.y.abs() < cell_height); // Near center Y
    }

    #[test]
    fn test_pixel_to_grid_roundtrip() {
        let cell_width = 8.0;
        let cell_height = 16.0;

        let col = 10u16;
        let row = 20u16;

        let pixel_pos = grid_to_pixel(col, row, cell_width, cell_height);
        let (new_col, new_row) = pixel_to_grid(pixel_pos, cell_width, cell_height).unwrap();

        assert_eq!(col, new_col);
        assert_eq!(row, new_row);
    }

    #[test]
    fn test_grid_cell_bounds() {
        let cell_width = 8.0;
        let cell_height = 16.0;

        let bounds = grid_cell_bounds(0, 0, cell_width, cell_height);
        assert_eq!(bounds.width(), cell_width);
        assert_eq!(bounds.height(), cell_height);

        // Verify the bounds are correct
        let pos = grid_to_pixel(0, 0, cell_width, cell_height);
        assert_eq!(bounds.min.x, pos.x);
        assert_eq!(bounds.max.x, pos.x + cell_width);
        assert_eq!(bounds.max.y, pos.y);
        assert_eq!(bounds.min.y, pos.y - cell_height);
    }

    #[test]
    fn test_grid_cell_center() {
        let cell_width = 8.0;
        let cell_height = 16.0;

        let pos = grid_to_pixel(5, 5, cell_width, cell_height);
        let center = grid_cell_center(5, 5, cell_width, cell_height);

        assert_eq!(center.x, pos.x + cell_width / 2.0);
        assert_eq!(center.y, pos.y - cell_height / 2.0);
    }

    #[test]
    fn test_grid_region_bounds() {
        let cell_width = 8.0;
        let cell_height = 16.0;

        let (pos, size) = grid_region_bounds(0, 0, 9, 4, cell_width, cell_height);

        assert_eq!(size.x, 10.0 * cell_width); // 10 columns (0-9 inclusive)
        assert_eq!(size.y, 5.0 * cell_height); // 5 rows (0-4 inclusive)

        let expected_pos = grid_to_pixel(0, 0, cell_width, cell_height);
        assert_eq!(pos, expected_pos);
    }

    #[test]
    fn test_pixel_to_grid_out_of_bounds() {
        let cell_width = 8.0;
        let cell_height = 16.0;

        // Way outside grid
        let far_pos = Vec2::new(100000.0, 100000.0);
        assert!(pixel_to_grid(far_pos, cell_width, cell_height).is_none());

        let far_neg = Vec2::new(-100000.0, -100000.0);
        assert!(pixel_to_grid(far_neg, cell_width, cell_height).is_none());
    }
}
