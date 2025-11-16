//! Example Integration Test Using Helpers
//!
//! Demonstrates the test helper utilities.

use assert_cmd::prelude::*;
use std::process::Command;

// Include helpers module
#[path = "../helpers/mod.rs"]
mod helpers;

use helpers::{TestEnv, generate_test_config, load_fixture};

#[test]
fn test_with_generated_config() {
    let mut env = TestEnv::new();
    let config = generate_test_config();
    let config_path = env.create_config(&config);

    assert!(config_path.exists());
    assert!(config.contains("test-cluster"));
}

#[test]
fn test_with_fixture() {
    let config = load_fixture("sample_config.yaml");
    assert!(!config.is_empty());
    assert!(config.contains("test-cluster"));
}

#[test]
fn test_config_validation_with_helpers() {
    let mut env = TestEnv::new();
    let config = generate_test_config();
    let config_path = env.create_config(&config);

    // This test would validate config with CLI
    // For now, just check file exists
    assert!(config_path.exists());
}
