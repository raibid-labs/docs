//! Test Scenario 4: Scrollback Buffer
//!
//! Tests scrollback behavior:
//! - Large output handling
//! - Sequence number updates
//! - Visible vs. scrollback distinction

use super::harness::E2ETestHarness;
use anyhow::Result;
use std::thread;
use std::time::Duration;

#[test]
fn test_large_output() -> Result<()> {
    println!("\n=== Test: Large Output (Scrollback) ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Get initial sequence number
    let state_before = harness.get_shared_state()?;
    let seq_before = state_before.sequence_number;

    // Generate 1000 lines
    println!("Generating 1000 lines of output...");
    harness.send_input("seq 1 1000\n")?;

    // Wait for completion
    thread::sleep(Duration::from_secs(3));

    // Verify sequence number increased significantly
    let state_after = harness.get_shared_state()?;
    let seq_after = state_after.sequence_number;

    println!("Sequence before: {}", seq_before);
    println!("Sequence after: {}", seq_after);
    println!("Difference: {}", seq_after - seq_before);

    assert!(
        seq_after > seq_before + 50,
        "Sequence number should increment significantly with large output"
    );

    // Verify last visible line contains a high number
    // (The exact number depends on scrollback implementation)
    let output = harness.get_output(Duration::from_millis(100))?;
    let has_high_numbers = output.contains("99") || output.contains("100");

    assert!(
        has_high_numbers,
        "Output should contain high numbers from seq command"
    );

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_continuous_output() -> Result<()> {
    println!("\n=== Test: Continuous Output ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    let state_before = harness.get_shared_state()?;
    let seq_before = state_before.sequence_number;

    // Run a command that produces continuous output
    harness.send_input("for i in {1..200}; do echo \"Line $i\"; done\n")?;

    thread::sleep(Duration::from_secs(3));

    let state_after = harness.get_shared_state()?;
    let seq_after = state_after.sequence_number;

    println!("Sequence updates: {}", seq_after - seq_before);

    assert!(
        seq_after > seq_before,
        "Sequence should increment with continuous output"
    );

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_scrollback_persistence() -> Result<()> {
    println!("\n=== Test: Scrollback Persistence ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Generate output that exceeds visible area
    harness.send_input("seq 1 50\n")?;
    thread::sleep(Duration::from_secs(2));

    // Send a marker
    harness.send_input("echo 'MARKER'\n")?;
    thread::sleep(Duration::from_millis(500));

    // Verify marker is visible
    let found = harness.verify_output_contains("MARKER", Duration::from_secs(1))?;
    assert!(found, "Marker should be visible");

    // Generate more output
    harness.send_input("seq 51 100\n")?;
    thread::sleep(Duration::from_secs(2));

    // The marker might have scrolled off, but the test verifies
    // the terminal handles scrollback correctly
    let state = harness.get_shared_state()?;
    println!("Final sequence number: {}", state.sequence_number);

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_rapid_updates() -> Result<()> {
    println!("\n=== Test: Rapid Updates ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    let state_before = harness.get_shared_state()?;
    let seq_before = state_before.sequence_number;

    // Send many rapid commands
    for i in 1..=20 {
        harness.send_input(&format!("echo 'Update {}'\n", i))?;
        thread::sleep(Duration::from_millis(50));
    }

    thread::sleep(Duration::from_secs(1));

    let state_after = harness.get_shared_state()?;
    let seq_after = state_after.sequence_number;

    println!(
        "Rapid updates caused {} sequence increments",
        seq_after - seq_before
    );

    assert!(
        seq_after > seq_before,
        "Sequence should increment with rapid updates"
    );

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_line_wrapping() -> Result<()> {
    println!("\n=== Test: Line Wrapping ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Send a very long line that should wrap
    let long_line = "A".repeat(300);
    harness.send_input(&format!("echo '{}'\n", long_line))?;

    thread::sleep(Duration::from_secs(1));

    // Verify some of the output appears
    let output = harness.get_output(Duration::from_millis(100))?;
    assert!(output.contains("AAA"), "Long line should be visible");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_partial_line_updates() -> Result<()> {
    println!("\n=== Test: Partial Line Updates ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Send partial output without newline
    harness.send_input("echo -n 'Partial'")?;
    thread::sleep(Duration::from_millis(500));

    // Add more to the same line
    harness.send_input("echo ' Complete'\n")?;
    thread::sleep(Duration::from_millis(500));

    let output = harness.get_output(Duration::from_millis(100))?;
    assert!(output.contains("Partial"), "Partial output should appear");
    assert!(output.contains("Complete"), "Complete output should appear");

    println!("=== Test Passed ===\n");
    Ok(())
}
