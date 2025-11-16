//! End-to-End Full Workflow Test
//!
//! This test validates the complete user workflow from configuration to deployment.
//!
//! Workflow:
//! 1. Initialize configuration
//! 2. Bootstrap k3s cluster
//! 3. Install Gitea
//! 4. Install Redis
//! 5. Deploy Flux GitOps
//! 6. Configure KEDA autoscaling
//! 7. Mirror a GitHub repository
//! 8. Trigger a build job
//! 9. Verify build agent scales up
//! 10. Verify build completes
//! 11. Verify agent scales down
//! 12. Cleanup resources

use assert_cmd::prelude::*;
use std::process::Command;
use std::time::Duration;
use tempfile::TempDir;

/// Full E2E workflow test
///
/// This test requires external services (k3s, Docker) and is ignored by default.
/// Run with: `TEST_EXTERNAL=1 cargo test --test e2e_full_workflow -- --ignored`
#[test]
#[ignore = "requires external services and takes several minutes"]
fn test_e2e_full_workflow() {
    // Skip if external tests not enabled
    if std::env::var("TEST_EXTERNAL").is_err() {
        eprintln!("Skipping E2E test - set TEST_EXTERNAL=1 to run");
        return;
    }

    // Setup
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let config_path = temp_dir.path().join("raibid.yaml");

    // Step 1: Initialize configuration
    println!("Step 1: Initializing configuration...");
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("config")
        .arg("init")
        .arg("--output")
        .arg(&config_path)
        .assert()
        .success();

    // Step 2: Bootstrap k3s cluster
    println!("Step 2: Bootstrapping k3s cluster...");
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("cluster")
        .arg("bootstrap")
        .arg("--config")
        .arg(&config_path)
        .timeout(Duration::from_secs(300)) // 5 minutes
        .assert()
        .success();

    // Step 3: Install Gitea
    println!("Step 3: Installing Gitea...");
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("gitea")
        .arg("install")
        .arg("--config")
        .arg(&config_path)
        .timeout(Duration::from_secs(180)) // 3 minutes
        .assert()
        .success();

    // Wait for Gitea to be ready
    std::thread::sleep(Duration::from_secs(30));

    // Step 4: Install Redis
    println!("Step 4: Installing Redis...");
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("redis")
        .arg("install")
        .arg("--config")
        .arg(&config_path)
        .assert()
        .success();

    // Step 5: Deploy Flux GitOps
    println!("Step 5: Deploying Flux...");
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("flux")
        .arg("install")
        .arg("--config")
        .arg(&config_path)
        .assert()
        .success();

    // Step 6: Configure KEDA
    println!("Step 6: Configuring KEDA...");
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("keda")
        .arg("install")
        .arg("--config")
        .arg(&config_path)
        .assert()
        .success();

    // Step 7: Mirror a test repository
    println!("Step 7: Mirroring test repository...");
    Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("mirror")
        .arg("add")
        .arg("https://github.com/rust-lang/rustlings")
        .arg("--config")
        .arg(&config_path)
        .assert()
        .success();

    // Step 8: Trigger a build job
    println!("Step 8: Triggering build job...");
    let output = Command::cargo_bin("raibid-cli")
        .unwrap()
        .arg("job")
        .arg("create")
        .arg("--repo")
        .arg("rustlings")
        .arg("--branch")
        .arg("main")
        .arg("--config")
        .arg(&config_path)
        .output()
        .expect("Failed to create job");

    assert!(output.status.success(), "Failed to create build job");

    // Extract job ID from output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let job_id = extract_job_id(&stdout);

    // Step 9: Wait for agent to scale up
    println!("Step 9: Waiting for agent to scale up...");
    wait_for_agent_scale_up(&config_path, Duration::from_secs(120));

    // Step 10: Wait for build to complete
    println!("Step 10: Waiting for build to complete...");
    wait_for_job_completion(&config_path, &job_id, Duration::from_secs(600));

    // Step 11: Wait for agent to scale down
    println!("Step 11: Waiting for agent to scale down...");
    wait_for_agent_scale_down(&config_path, Duration::from_secs(300));

    // Step 12: Cleanup
    println!("Step 12: Cleaning up...");
    if std::env::var("RAIBID_TEST_NO_CLEANUP").is_err() {
        Command::cargo_bin("raibid-cli")
            .unwrap()
            .arg("cluster")
            .arg("destroy")
            .arg("--config")
            .arg(&config_path)
            .arg("--force")
            .assert()
            .success();
    } else {
        println!("Skipping cleanup (RAIBID_TEST_NO_CLEANUP is set)");
    }

    println!("E2E test completed successfully!");
}

/// Extract job ID from command output
fn extract_job_id(output: &str) -> String {
    // Look for pattern like "Job created: job-abc123"
    for line in output.lines() {
        if line.contains("Job created:") || line.contains("job-") {
            if let Some(id) = line.split_whitespace().last() {
                return id.to_string();
            }
        }
    }
    panic!("Could not extract job ID from output: {}", output);
}

/// Wait for agent deployment to scale up
fn wait_for_agent_scale_up(config_path: &std::path::Path, timeout: Duration) {
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for agent to scale up");
        }

        let output = Command::cargo_bin("raibid-cli")
            .unwrap()
            .arg("status")
            .arg("--config")
            .arg(config_path)
            .arg("--format")
            .arg("json")
            .output()
            .expect("Failed to get status");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("\"running\": 1") || stdout.contains("\"running\":1") {
                println!("Agent scaled up!");
                return;
            }
        }

        std::thread::sleep(Duration::from_secs(5));
    }
}

/// Wait for agent deployment to scale down
fn wait_for_agent_scale_down(config_path: &std::path::Path, timeout: Duration) {
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for agent to scale down");
        }

        let output = Command::cargo_bin("raibid-cli")
            .unwrap()
            .arg("status")
            .arg("--config")
            .arg(config_path)
            .arg("--format")
            .arg("json")
            .output()
            .expect("Failed to get status");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("\"running\": 0") || stdout.contains("\"running\":0") {
                println!("Agent scaled down!");
                return;
            }
        }

        std::thread::sleep(Duration::from_secs(5));
    }
}

/// Wait for job to complete
fn wait_for_job_completion(
    config_path: &std::path::Path,
    job_id: &str,
    timeout: Duration,
) {
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            panic!("Timeout waiting for job {} to complete", job_id);
        }

        let output = Command::cargo_bin("raibid-cli")
            .unwrap()
            .arg("job")
            .arg("status")
            .arg(job_id)
            .arg("--config")
            .arg(config_path)
            .output()
            .expect("Failed to get job status");

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("Status: completed")
                || stdout.contains("Status: success")
            {
                println!("Job {} completed!", job_id);
                return;
            } else if stdout.contains("Status: failed") {
                panic!("Job {} failed!", job_id);
            }
        }

        std::thread::sleep(Duration::from_secs(10));
    }
}
