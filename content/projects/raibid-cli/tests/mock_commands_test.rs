//! Integration tests for CLI-002: Mock Infrastructure Commands
//!
//! These tests verify the mock implementations of setup, teardown, and status commands.

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use predicates::prelude::*;
use std::process::Command;

// ============================================================================
// SETUP COMMAND TESTS
// ============================================================================

/// Test setup command with k3s component
#[test]
fn test_setup_k3s() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("setup")
        .arg("k3s")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setting up k3s"))
        .stdout(predicate::str::contains("pre-flight checks"))
        .stdout(predicate::str::contains("success").or(predicate::str::contains("✓")));
}

/// Test setup command with gitea component
#[test]
fn test_setup_gitea() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("setup")
        .arg("gitea")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setting up gitea"))
        .stdout(predicate::str::contains("k3s"));
}

/// Test setup command with redis component
#[test]
fn test_setup_redis() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("setup")
        .arg("redis")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setting up redis"));
}

/// Test setup command with keda component
#[test]
fn test_setup_keda() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("setup")
        .arg("keda")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setting up keda"));
}

/// Test setup command with flux component
#[test]
fn test_setup_flux() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("setup")
        .arg("flux")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setting up flux"));
}

/// Test setup command with all components
#[test]
fn test_setup_all() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("setup")
        .arg("all")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setting up all components"))
        .stdout(predicate::str::contains("k3s"))
        .stdout(predicate::str::contains("gitea"))
        .stdout(predicate::str::contains("redis"))
        .stdout(predicate::str::contains("keda"))
        .stdout(predicate::str::contains("flux"));
}

/// Test setup command without component argument shows error
#[test]
fn test_setup_no_component() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("setup")
        .assert()
        .failure();
}

/// Test setup command shows dependency information
#[test]
fn test_setup_shows_dependencies() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("setup")
        .arg("gitea")
        .assert()
        .success()
        .stdout(predicate::str::contains("requires").or(predicate::str::contains("depends")));
}

/// Test setup command shows pre-flight checks
#[test]
fn test_setup_preflight_checks() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("setup")
        .arg("k3s")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("disk")
                .or(predicate::str::contains("memory"))
                .or(predicate::str::contains("CPU")),
        );
}

/// Test setup command with verbose flag
#[test]
fn test_setup_verbose() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("--verbose")
        .arg("setup")
        .arg("k3s")
        .assert()
        .success();
}

// ============================================================================
// TEARDOWN COMMAND TESTS
// ============================================================================

/// Test teardown command with k3s component
#[test]
fn test_teardown_k3s() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("teardown")
        .arg("k3s")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tearing down k3s"))
        .stdout(predicate::str::contains("success").or(predicate::str::contains("✓")));
}

/// Test teardown command with gitea component
#[test]
fn test_teardown_gitea() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("teardown")
        .arg("gitea")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tearing down gitea"));
}

/// Test teardown command with redis component
#[test]
fn test_teardown_redis() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("teardown")
        .arg("redis")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tearing down redis"));
}

/// Test teardown command with keda component
#[test]
fn test_teardown_keda() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("teardown")
        .arg("keda")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tearing down keda"));
}

/// Test teardown command with flux component
#[test]
fn test_teardown_flux() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("teardown")
        .arg("flux")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tearing down flux"));
}

/// Test teardown command with all components
#[test]
fn test_teardown_all() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("teardown")
        .arg("all")
        .assert()
        .success()
        .stdout(predicate::str::contains("Tearing down all components"))
        .stdout(predicate::str::contains("k3s"))
        .stdout(predicate::str::contains("gitea"))
        .stdout(predicate::str::contains("redis"))
        .stdout(predicate::str::contains("keda"))
        .stdout(predicate::str::contains("flux"));
}

/// Test teardown command without component argument shows error
#[test]
fn test_teardown_no_component() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("teardown")
        .assert()
        .failure();
}

/// Test teardown command shows what will be removed
#[test]
fn test_teardown_shows_removal_info() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("teardown")
        .arg("k3s")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("Removing")
                .or(predicate::str::contains("Cleaning"))
                .or(predicate::str::contains("remove")),
        );
}

/// Test teardown command with verbose flag
#[test]
fn test_teardown_verbose() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("--verbose")
        .arg("teardown")
        .arg("k3s")
        .assert()
        .success();
}

// ============================================================================
// STATUS COMMAND TESTS
// ============================================================================

/// Test status command with k3s component
#[test]
fn test_status_k3s() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("status")
        .arg("k3s")
        .assert()
        .success()
        .stdout(predicate::str::contains("k3s"));
}

/// Test status command with gitea component
#[test]
fn test_status_gitea() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("status")
        .arg("gitea")
        .assert()
        .success()
        .stdout(predicate::str::contains("gitea"));
}

/// Test status command with redis component
#[test]
fn test_status_redis() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("status")
        .arg("redis")
        .assert()
        .success()
        .stdout(predicate::str::contains("redis"));
}

/// Test status command with keda component
#[test]
fn test_status_keda() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("status")
        .arg("keda")
        .assert()
        .success()
        .stdout(predicate::str::contains("keda"));
}

/// Test status command with flux component
#[test]
fn test_status_flux() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("status")
        .arg("flux")
        .assert()
        .success()
        .stdout(predicate::str::contains("flux"));
}

/// Test status command with all components
#[test]
fn test_status_all() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("status")
        .arg("all")
        .assert()
        .success()
        .stdout(predicate::str::contains("k3s"))
        .stdout(predicate::str::contains("gitea"))
        .stdout(predicate::str::contains("redis"))
        .stdout(predicate::str::contains("keda"))
        .stdout(predicate::str::contains("flux"));
}

/// Test status command without component shows all
#[test]
fn test_status_no_component_shows_all() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("k3s"))
        .stdout(predicate::str::contains("gitea"));
}

/// Test status command shows version information
#[test]
fn test_status_shows_version() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("status")
        .arg("k3s")
        .assert()
        .success()
        .stdout(predicate::str::contains("version").or(predicate::str::contains("v1")));
}

/// Test status command shows state information
#[test]
fn test_status_shows_state() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("status")
        .arg("k3s")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("running")
                .or(predicate::str::contains("stopped"))
                .or(predicate::str::contains("state")),
        );
}

/// Test status command shows resource usage
#[test]
fn test_status_shows_resource_usage() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("status")
        .arg("k3s")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("CPU")
                .or(predicate::str::contains("Memory"))
                .or(predicate::str::contains("memory")),
        );
}

/// Test status command with verbose flag
#[test]
fn test_status_verbose() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("--verbose")
        .arg("status")
        .arg("k3s")
        .assert()
        .success();
}

// ============================================================================
// HELP AND ERROR TESTS
// ============================================================================

/// Test setup command help
#[test]
fn test_setup_help() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("setup")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Setup infrastructure component"));
}

/// Test teardown command help
#[test]
fn test_teardown_help() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("teardown")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Teardown infrastructure component",
        ));
}

/// Test status command help
#[test]
fn test_status_help() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("status")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Show status of infrastructure component",
        ));
}
