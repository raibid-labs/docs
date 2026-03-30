//! Test Scenario 1: Basic Workflow
//!
//! Tests basic terminal operations:
//! - Echo command
//! - Simple text input
//! - Output verification

use super::harness::E2ETestHarness;
use anyhow::Result;
use std::thread;
use std::time::Duration;

#[test]
fn test_basic_echo() -> Result<()> {
    println!("\n=== Test: Basic Echo ===");

    let mut harness = E2ETestHarness::new()?;
    harness.start_client()?;

    // Wait for shell prompt to appear
    thread::sleep(Duration::from_secs(1));

    // Send echo command
    println!("Sending: echo 'Hello, World!'");
    harness.send_input("echo 'Hello, World!'\n")?;

    // Verify output appears
    let found = harness.verify_output_contains("Hello, World!", Duration::from_secs(3))?;
    assert!(found, "Expected output 'Hello, World!' not found");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_multiple_commands() -> Result<()> {
    println!("\n=== Test: Multiple Commands ===");

    let mut harness = E2ETestHarness::new()?;
    harness.start_client()?;

    thread::sleep(Duration::from_secs(1));

    // Send multiple commands
    harness.send_input("echo 'First'\n")?;
    thread::sleep(Duration::from_millis(500));

    harness.send_input("echo 'Second'\n")?;
    thread::sleep(Duration::from_millis(500));

    harness.send_input("echo 'Third'\n")?;
    thread::sleep(Duration::from_millis(500));

    // Verify all outputs
    let output = harness.get_output(Duration::from_millis(100))?;
    assert!(output.contains("First"), "First command output missing");
    assert!(output.contains("Second"), "Second command output missing");
    assert!(output.contains("Third"), "Third command output missing");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_pwd_command() -> Result<()> {
    println!("\n=== Test: PWD Command ===");

    let mut harness = E2ETestHarness::new()?;
    harness.start_client()?;

    thread::sleep(Duration::from_secs(1));

    // Send pwd command
    harness.send_input("pwd\n")?;

    // Verify output contains a path
    let found = harness.verify_output_contains("/", Duration::from_secs(2))?;
    assert!(found, "PWD output should contain a path");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_environment_variables() -> Result<()> {
    println!("\n=== Test: Environment Variables ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Set and retrieve environment variable
    harness.send_input("export TEST_VAR='test_value'\n")?;
    thread::sleep(Duration::from_millis(500));

    harness.send_input("echo $TEST_VAR\n")?;

    let found = harness.verify_output_contains("test_value", Duration::from_secs(2))?;
    assert!(found, "Environment variable not found");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_multiline_input() -> Result<()> {
    println!("\n=== Test: Multiline Input ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Send multiline command
    harness.send_input("for i in 1 2 3; do echo \"Number $i\"; done\n")?;

    // Verify output
    thread::sleep(Duration::from_secs(1));
    let output = harness.get_output(Duration::from_millis(100))?;

    assert!(output.contains("Number 1"), "First iteration missing");
    assert!(output.contains("Number 2"), "Second iteration missing");
    assert!(output.contains("Number 3"), "Third iteration missing");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_clear_screen() -> Result<()> {
    println!("\n=== Test: Clear Screen ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Fill screen with output
    harness.send_input("echo 'Before clear'\n")?;
    thread::sleep(Duration::from_millis(500));

    // Get initial state
    let state_before = harness.get_shared_state()?;
    let seq_before = state_before.sequence_number;

    // Clear screen
    harness.send_input("clear\n")?;
    thread::sleep(Duration::from_millis(500));

    // Verify sequence number changed (screen was updated)
    let state_after = harness.get_shared_state()?;
    assert!(
        state_after.sequence_number > seq_before,
        "Sequence number should increment after clear"
    );

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_backspace_handling() -> Result<()> {
    println!("\n=== Test: Backspace Handling ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Type text with backspaces
    harness.send_input("echo 'mistake")?;
    thread::sleep(Duration::from_millis(200));

    // Send backspaces (0x7F)
    for _ in 0..7 {
        harness.send_input("\x7F")?;
        thread::sleep(Duration::from_millis(50));
    }

    harness.send_input("test'\n")?;

    let found = harness.verify_output_contains("test", Duration::from_secs(2))?;
    assert!(found, "Backspace correction not applied");

    println!("=== Test Passed ===\n");
    Ok(())
}
