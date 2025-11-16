//! Integration tests for CLI-006: Additional Commands (job, agent, mirror)
//!
//! These tests verify the mock implementations of job, agent, and mirror commands.

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use predicates::prelude::*;
use std::process::Command;

// ============================================================================
// JOB COMMAND TESTS
// ============================================================================

/// Test job list command
#[test]
fn test_job_list() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("REPOSITORY"))
        .stdout(predicate::str::contains("STATUS"));
}

/// Test job list with status filter
#[test]
fn test_job_list_status_filter() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("list")
        .arg("--status")
        .arg("running")
        .assert()
        .success();
}

/// Test job list with repo filter
#[test]
fn test_job_list_repo_filter() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("list")
        .arg("--repo")
        .arg("raibid/core")
        .assert()
        .success();
}

/// Test job list with limit
#[test]
fn test_job_list_limit() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("list")
        .arg("--limit")
        .arg("2")
        .assert()
        .success();
}

/// Test job list JSON output
#[test]
fn test_job_list_json() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("list")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("repository"));
}

/// Test job show command
#[test]
fn test_job_show() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("show")
        .arg("a1b2c3")
        .assert()
        .success()
        .stdout(predicate::str::contains("Job Details"))
        .stdout(predicate::str::contains("a1b2c3"));
}

/// Test job show with invalid ID
#[test]
fn test_job_show_invalid_id() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("show")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test job show JSON output
#[test]
fn test_job_show_json() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("show")
        .arg("a1b2c3")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("repository"))
        .stdout(predicate::str::contains("branch"));
}

/// Test job cancel with force flag
#[test]
fn test_job_cancel_force() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("cancel")
        .arg("a1b2c3")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Cancelled"));
}

/// Test job retry command
#[test]
fn test_job_retry() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("retry")
        .arg("g7h8i9")
        .assert()
        .success()
        .stdout(predicate::str::contains("Retrying"))
        .stdout(predicate::str::contains("retry"));
}

/// Test job retry with invalid ID
#[test]
fn test_job_retry_invalid_id() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("retry")
        .arg("invalid")
        .assert()
        .failure();
}

/// Test job command help
#[test]
fn test_job_help() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage CI jobs"));
}

// ============================================================================
// AGENT COMMAND TESTS
// ============================================================================

/// Test agent list command
#[test]
fn test_agent_list() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("ID"))
        .stdout(predicate::str::contains("STATUS"));
}

/// Test agent list with status filter
#[test]
fn test_agent_list_status_filter() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("list")
        .arg("--status")
        .arg("running")
        .assert()
        .success();
}

/// Test agent list JSON output
#[test]
fn test_agent_list_json() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("list")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("id"));
}

/// Test agent show command
#[test]
fn test_agent_show() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("show")
        .arg("rust-builder-1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Agent Details"))
        .stdout(predicate::str::contains("rust-builder-1"));
}

/// Test agent show with invalid ID
#[test]
fn test_agent_show_invalid_id() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("show")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test agent show JSON output
#[test]
fn test_agent_show_json() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("show")
        .arg("rust-builder-1")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("cpu_usage"));
}

/// Test agent restart with force flag
#[test]
fn test_agent_restart_force() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("restart")
        .arg("rust-builder-1")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Restarted"));
}

/// Test agent restart with invalid ID
#[test]
fn test_agent_restart_invalid_id() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("restart")
        .arg("invalid")
        .arg("--force")
        .assert()
        .failure();
}

/// Test agent scale command
#[test]
fn test_agent_scale() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("scale")
        .arg("--count")
        .arg("5")
        .assert()
        .success()
        .stdout(predicate::str::contains("Scaled"));
}

/// Test agent scale with min and max
#[test]
fn test_agent_scale_min_max() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("scale")
        .arg("--count")
        .arg("3")
        .arg("--min")
        .arg("2")
        .arg("--max")
        .arg("8")
        .assert()
        .success();
}

/// Test agent scale with count below minimum
#[test]
fn test_agent_scale_below_minimum() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("scale")
        .arg("--count")
        .arg("1")
        .arg("--min")
        .arg("2")
        .assert()
        .failure()
        .stderr(predicate::str::contains("less than minimum"));
}

/// Test agent scale with count above maximum
#[test]
fn test_agent_scale_above_maximum() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("scale")
        .arg("--count")
        .arg("11")
        .arg("--max")
        .arg("10")
        .assert()
        .failure()
        .stderr(predicate::str::contains("greater than maximum"));
}

/// Test agent command help
#[test]
fn test_agent_help() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("agent")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage CI agents"));
}

// ============================================================================
// MIRROR COMMAND TESTS
// ============================================================================

/// Test mirror add command
#[test]
fn test_mirror_add() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("add")
        .arg("github.com/test/repo")
        .assert()
        .success()
        .stdout(predicate::str::contains("Added mirror"));
}

/// Test mirror add with custom name
#[test]
fn test_mirror_add_with_name() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("add")
        .arg("github.com/test/repo")
        .arg("--name")
        .arg("test-repo")
        .assert()
        .success()
        .stdout(predicate::str::contains("test-repo"));
}

/// Test mirror add with sync interval
#[test]
fn test_mirror_add_with_interval() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("add")
        .arg("github.com/test/repo")
        .arg("--sync-interval")
        .arg("30")
        .assert()
        .success()
        .stdout(predicate::str::contains("30 minutes"));
}

/// Test mirror list command
#[test]
fn test_mirror_list() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("REPOSITORY"))
        .stdout(predicate::str::contains("STATUS"));
}

/// Test mirror list JSON output
#[test]
fn test_mirror_list_json() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("list")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("["))
        .stdout(predicate::str::contains("repository"));
}

/// Test mirror sync command
#[test]
fn test_mirror_sync() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("sync")
        .arg("github.com/raibid/core")
        .assert()
        .success()
        .stdout(predicate::str::contains("Syncing"));
}

/// Test mirror sync with force flag
#[test]
fn test_mirror_sync_force() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("sync")
        .arg("github.com/raibid/core")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Force syncing"));
}

/// Test mirror sync with invalid repo
#[test]
fn test_mirror_sync_invalid_repo() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("sync")
        .arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test mirror remove with force flag
#[test]
fn test_mirror_remove_force() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("remove")
        .arg("github.com/raibid/core")
        .arg("--force")
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed"));
}

/// Test mirror remove with invalid repo
#[test]
fn test_mirror_remove_invalid_repo() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("remove")
        .arg("invalid")
        .arg("--force")
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test mirror command help
#[test]
fn test_mirror_help() {
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage repository mirrors"));
}
