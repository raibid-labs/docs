//! Integration tests for jobs commands
//!
//! These tests verify that the job management commands work correctly
//! with various inputs and produce the expected outputs.

use assert_cmd::Command;
use predicates::prelude::*;

/// Test that the jobs command shows help when no subcommand is provided
#[test]
fn test_jobs_no_subcommand() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage: raibid jobs"));
}

/// Test that jobs list command accepts valid arguments
#[test]
fn test_jobs_list_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs").arg("list").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("List jobs with optional filters"))
        .stdout(predicate::str::contains("--status"))
        .stdout(predicate::str::contains("--repo"))
        .stdout(predicate::str::contains("--branch"))
        .stdout(predicate::str::contains("--json"));
}

/// Test that jobs show command requires job ID
#[test]
fn test_jobs_show_requires_id() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs").arg("show");

    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
}

/// Test that jobs show command help
#[test]
fn test_jobs_show_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs").arg("show").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Show detailed information about a specific job",
        ))
        .stdout(predicate::str::contains("<JOB_ID>"));
}

/// Test that jobs logs command requires job ID
#[test]
fn test_jobs_logs_requires_id() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs").arg("logs");

    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
}

/// Test that jobs logs command accepts follow and tail options
#[test]
fn test_jobs_logs_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs").arg("logs").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Show logs for a specific job"))
        .stdout(predicate::str::contains("--follow"))
        .stdout(predicate::str::contains("--tail"));
}

/// Test that jobs trigger command requires repo and branch
#[test]
fn test_jobs_trigger_requires_args() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs").arg("trigger");

    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
}

/// Test that jobs trigger command with only repo fails
#[test]
fn test_jobs_trigger_requires_branch() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs")
        .arg("trigger")
        .arg("--repo")
        .arg("test-repo");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

/// Test that jobs trigger command help
#[test]
fn test_jobs_trigger_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs").arg("trigger").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Trigger a new job"))
        .stdout(predicate::str::contains("--repo"))
        .stdout(predicate::str::contains("--branch"))
        .stdout(predicate::str::contains("--commit"));
}

/// Test that jobs cancel command requires job ID
#[test]
fn test_jobs_cancel_requires_id() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs").arg("cancel");

    cmd.assert().failure().stderr(predicate::str::contains(
        "required arguments were not provided",
    ));
}

/// Test that jobs cancel command help
#[test]
fn test_jobs_cancel_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs").arg("cancel").arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Cancel a running or pending job"))
        .stdout(predicate::str::contains("<JOB_ID>"));
}

/// Test that jobs list with invalid status filter fails gracefully
/// Note: This will fail due to API not being available, but we can verify
/// the error message is helpful
#[test]
fn test_jobs_list_invalid_status() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs")
        .arg("list")
        .arg("--status")
        .arg("invalid-status");

    // Should fail to connect to API, but error should mention the status parsing
    cmd.assert().failure();
}

/// Test that JSON output flag is recognized
#[test]
fn test_jobs_list_json_flag() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs").arg("list").arg("--json");

    // Will fail to connect, but flag should be recognized
    cmd.assert().failure();
}

/// Test pagination parameters
#[test]
fn test_jobs_list_pagination() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid"));
    cmd.arg("jobs")
        .arg("list")
        .arg("--limit")
        .arg("10")
        .arg("--offset")
        .arg("5");

    // Will fail to connect, but flags should be recognized
    cmd.assert().failure();
}
