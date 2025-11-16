// Shared modules
pub mod commands;

// Library entry point for mobile platforms
#[cfg(mobile)]
mod mobile;

#[cfg(mobile)]
pub use mobile::*;
