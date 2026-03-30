// Rendering subsystem for Scarab terminal
// GPU-accelerated text rendering with cosmic-text and glyph atlas caching

pub mod atlas;
pub mod config;
pub mod scrollback_render;
pub mod text;

pub use atlas::{AtlasRect, GlyphAtlas, GlyphKey};
pub use config::{color, FontConfig, TextAttributes};
pub use scrollback_render::generate_scrollback_mesh;
pub use text::{
    generate_terminal_mesh, DirtyRegion, TerminalMesh, TextRenderer,
    update_terminal_mesh_system,
};
