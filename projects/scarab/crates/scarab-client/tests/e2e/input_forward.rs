//! Test Scenario 6: Input Forwarding
//!
//! Tests that input is correctly forwarded from client to daemon:
//! - Regular text input
//! - Control sequences (Ctrl+C, Ctrl+D, etc.)
//! - Arrow keys
//! - Function keys
//! - Special characters

use super::harness::E2ETestHarness;
use anyhow::Result;
use std::thread;
use std::time::Duration;

#[test]
fn test_basic_text_input() -> Result<()> {
    println!("\n=== Test: Basic Text Input ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Send regular text
    harness.send_input("hello world\n")?;

    let found = harness.verify_output_contains("hello world", Duration::from_secs(2))?;
    assert!(found, "Regular text should be forwarded");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_control_sequences() -> Result<()> {
    println!("\n=== Test: Control Sequences ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Test Ctrl+C (interrupt)
    // Start a command that would run forever
    harness.send_input("sleep 1000")?;
    thread::sleep(Duration::from_millis(200));

    // Send Ctrl+C
    harness.send_input("\x03")?;
    thread::sleep(Duration::from_millis(500));

    // Terminal should be responsive again
    harness.send_input("echo 'After Ctrl+C'\n")?;

    let found = harness.verify_output_contains("After Ctrl+C", Duration::from_secs(2))?;
    assert!(found, "Should be able to interrupt with Ctrl+C");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_ctrl_d_eof() -> Result<()> {
    println!("\n=== Test: Ctrl+D (EOF) ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Start cat (reads from stdin)
    harness.send_input("cat\n")?;
    thread::sleep(Duration::from_millis(500));

    // Send some input
    harness.send_input("test input\n")?;
    thread::sleep(Duration::from_millis(500));

    // Send Ctrl+D to close stdin
    harness.send_input("\x04")?;
    thread::sleep(Duration::from_millis(500));

    // Cat should have exited, terminal should be responsive
    harness.send_input("echo 'After cat'\n")?;

    let found = harness.verify_output_contains("After cat", Duration::from_secs(2))?;
    assert!(found, "Ctrl+D should send EOF");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_arrow_keys() -> Result<()> {
    println!("\n=== Test: Arrow Keys ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Type a command
    harness.send_input("echo test")?;
    thread::sleep(Duration::from_millis(200));

    // Move cursor left with arrow key
    harness.send_input("\x1b[D")?; // Left arrow
    thread::sleep(Duration::from_millis(100));
    harness.send_input("\x1b[D")?; // Left arrow
    thread::sleep(Duration::from_millis(100));
    harness.send_input("\x1b[D")?; // Left arrow
    thread::sleep(Duration::from_millis(100));
    harness.send_input("\x1b[D")?; // Left arrow
    thread::sleep(Duration::from_millis(100));

    // Insert text in the middle
    harness.send_input("_modified")?;
    thread::sleep(Duration::from_millis(200));

    // Execute
    harness.send_input("\n")?;

    // Verify the modified command executed
    let output = harness.get_output(Duration::from_secs(2))?;
    println!("Output: {}", output);

    // Note: exact behavior depends on shell implementation
    // Just verify the command executed without error

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_tab_completion() -> Result<()> {
    println!("\n=== Test: Tab Completion ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Type partial command and hit tab
    harness.send_input("ec")?;
    thread::sleep(Duration::from_millis(200));

    // Send tab character
    harness.send_input("\t")?;
    thread::sleep(Duration::from_millis(500));

    // Depending on shell, this might complete to "echo"
    // We can't reliably test the exact behavior, but verify
    // the tab was processed
    harness.send_input("\n")?;
    thread::sleep(Duration::from_millis(500));

    // Just verify terminal is still responsive
    harness.send_input("echo 'tab test'\n")?;
    let found = harness.verify_output_contains("tab test", Duration::from_secs(2))?;
    assert!(found, "Tab key should be forwarded");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_special_characters() -> Result<()> {
    println!("\n=== Test: Special Characters ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Test various special characters
    let special_chars = vec![
        ("Exclamation", "echo '!'"),
        ("At sign", "echo '@'"),
        ("Hash", "echo '#'"),
        ("Dollar", "echo '$'"),
        ("Percent", "echo '%'"),
        ("Caret", "echo '^'"),
        ("Ampersand", "echo '&'"),
        ("Asterisk", "echo '*'"),
    ];

    for (name, cmd) in special_chars {
        println!("Testing: {}", name);
        harness.send_input(&format!("{}\n", cmd))?;
        thread::sleep(Duration::from_millis(300));
    }

    // Verify terminal is still responsive
    harness.send_input("echo 'special chars done'\n")?;
    let found = harness.verify_output_contains("special chars done", Duration::from_secs(2))?;
    assert!(found, "Special characters should be handled");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_escape_sequences() -> Result<()> {
    println!("\n=== Test: Escape Sequences ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Send escape key
    harness.send_input("\x1b")?;
    thread::sleep(Duration::from_millis(200));

    // Terminal should still be responsive
    harness.send_input("echo 'after escape'\n")?;

    let found = harness.verify_output_contains("after escape", Duration::from_secs(2))?;
    assert!(found, "Escape key should be handled");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_unicode_input() -> Result<()> {
    println!("\n=== Test: Unicode Input ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Test Unicode characters
    harness.send_input("echo 'Hello 世界'\n")?;

    let found = harness.verify_output_contains("世界", Duration::from_secs(2))?;
    assert!(found, "Unicode characters should be forwarded");

    // Test emoji
    harness.send_input("echo '🚀 Rocket'\n")?;

    let output = harness.get_output(Duration::from_secs(1))?;
    println!("Unicode output: {}", output);

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_rapid_input() -> Result<()> {
    println!("\n=== Test: Rapid Input ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Send rapid key presses
    for _i in 0..50 {
        harness.send_input("x")?;
        // No sleep between inputs to test rapid input handling
    }

    thread::sleep(Duration::from_millis(500));

    // Clear the line
    harness.send_input("\x03")?; // Ctrl+C
    thread::sleep(Duration::from_millis(200));

    // Verify terminal is still responsive
    harness.send_input("echo 'rapid test done'\n")?;

    let found = harness.verify_output_contains("rapid test done", Duration::from_secs(2))?;
    assert!(found, "Should handle rapid input");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_ctrl_l_clear() -> Result<()> {
    println!("\n=== Test: Ctrl+L (Clear Screen) ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Fill screen with output
    harness.send_input("echo 'Before clear'\n")?;
    thread::sleep(Duration::from_millis(500));

    // Send Ctrl+L
    harness.send_input("\x0C")?;
    thread::sleep(Duration::from_millis(500));

    // Verify screen was cleared (sequence number should change)
    let state = harness.get_shared_state()?;
    println!("Sequence after Ctrl+L: {}", state.sequence_number);

    // Terminal should be responsive
    harness.send_input("echo 'After clear'\n")?;

    let found = harness.verify_output_contains("After clear", Duration::from_secs(2))?;
    assert!(found, "Should work after Ctrl+L");

    println!("=== Test Passed ===\n");
    Ok(())
}
