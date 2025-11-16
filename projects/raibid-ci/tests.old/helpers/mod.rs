//! Test Helpers Module
//!
//! Common utilities and helper functions shared across integration and E2E tests.
//!
//! This module provides:
//! - Test environment setup/teardown
//! - Mock builders for k3s, Gitea, Redis
//! - Test data generators
//! - Common assertion helpers

pub mod test_env;
pub mod mock_builders;
pub mod generators;
pub mod assertions;

// Re-export commonly used items
pub use test_env::{TestEnv, setup_test_env, teardown_test_env};
pub use mock_builders::{MockCluster, MockGitea, MockRedis};
pub use generators::{generate_test_config, generate_test_manifest};
pub use assertions::{assert_success_output, assert_error_contains};

use std::path::PathBuf;

/// Get the path to a fixture file
///
/// # Examples
/// ```
/// let config_path = fixture_path("sample_config.yaml");
/// ```
pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

/// Load fixture file contents as string
///
/// # Panics
/// Panics if the fixture file doesn't exist or can't be read
pub fn load_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture_path(name))
        .unwrap_or_else(|_| panic!("Failed to load fixture: {}", name))
}

/// Check if external tests are enabled via TEST_EXTERNAL env var
pub fn external_tests_enabled() -> bool {
    std::env::var("TEST_EXTERNAL").is_ok()
}

/// Check if cleanup should be skipped (for debugging)
pub fn skip_cleanup() -> bool {
    std::env::var("RAIBID_TEST_NO_CLEANUP").is_ok()
}
