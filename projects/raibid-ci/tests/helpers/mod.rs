//! Test Helpers Module
pub mod test_env;
pub mod generators;

pub use test_env::TestEnv;
pub use generators::*;

use std::path::PathBuf;

pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

pub fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|_| panic!("Failed to load fixture: {}", name))
}
