//! BRP (Bevy Remote Protocol) integration module
//!
//! This module provides custom BRP methods for AI-controlled entity spawning
//! and manipulation. Standard BRP methods cannot spawn entities with meshes/materials
//! because asset handles aren't serializable. These custom methods handle asset
//! creation internally.

pub mod config;
pub mod tools;

pub use config::BrpConfig;
pub use tools::CustomBrpPlugin;
