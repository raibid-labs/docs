//! Integration tests for the CLI
//!
//! These tests verify that the CLI binary can be built and executed
//! with various command-line arguments.

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use predicates::prelude::*;
use std::process::Command;

/// Test that the binary can be invoked
#[test]
fn test_cli_binary_exists() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"));
    cmd.assert();
}

/// Test --version flag
#[test]
fn test_version_flag() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("raibid-cli"));
}

/// Test short version flag
#[test]
fn test_version_flag_short() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("raibid-cli"));
}

/// Test --help flag
#[test]
fn test_help_flag() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Options:"));
}

/// Test short help flag
#[test]
fn test_help_flag_short() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

/// Test running without arguments shows helpful message
#[test]
fn test_no_arguments() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .assert()
        .failure()
        .stdout(predicate::str::contains("No command specified"));
}

/// Test verbose flag is recognized
#[test]
fn test_verbose_flag() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("--verbose")
        .assert()
        .failure(); // Should fail because no command specified, but flag should be recognized
}

/// Test that help shows information about the tool
#[test]
fn test_help_shows_description() {
    Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"))
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("DGX Spark"));
}
