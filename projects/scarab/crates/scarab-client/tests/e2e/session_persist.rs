//! Test Scenario 5: Session Persistence
//!
//! Tests daemon persistence when client disconnects:
//! - Client disconnect/reconnect
//! - Session state preservation
//! - PTY persistence

use super::harness::E2ETestHarness;
use anyhow::Result;
use std::thread;
use std::time::Duration;

#[test]
fn test_client_disconnect_reconnect() -> Result<()> {
    println!("\n=== Test: Client Disconnect/Reconnect ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Type a command but don't execute it
    harness.send_input("echo 'Test persistence'")?;
    thread::sleep(Duration::from_millis(500));

    // Verify the text appears
    let found =
        harness.verify_output_contains("echo 'Test persistence'", Duration::from_secs(1))?;
    assert!(found, "Typed command should be visible");

    // Now execute it
    harness.send_input("\n")?;
    thread::sleep(Duration::from_millis(500));

    // Verify output
    let output_before = harness.get_output(Duration::from_millis(100))?;
    println!("Output before disconnect:\n{}", output_before);

    // Kill client (daemon should stay alive)
    harness.disconnect_client()?;

    // Wait a bit
    thread::sleep(Duration::from_secs(1));

    // Verify daemon is still alive
    assert!(harness.daemon_is_alive(), "Daemon should still be running");

    // Reconnect client
    harness.reconnect_client()?;

    // Verify we can still interact with the terminal
    harness.send_input("echo 'After reconnect'\n")?;

    let found = harness.verify_output_contains("After reconnect", Duration::from_secs(2))?;
    assert!(found, "Should be able to send commands after reconnect");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_daemon_survives_client_crash() -> Result<()> {
    println!("\n=== Test: Daemon Survives Client Crash ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Start client
    harness.start_client()?;
    thread::sleep(Duration::from_millis(500));

    // Verify client is running
    assert!(harness.client_is_alive(), "Client should be running");

    // Send some commands
    harness.send_input("echo 'Before crash'\n")?;
    thread::sleep(Duration::from_millis(500));

    // Kill client abruptly
    harness.disconnect_client()?;

    // Verify daemon survived
    thread::sleep(Duration::from_millis(500));
    assert!(
        harness.daemon_is_alive(),
        "Daemon should survive client crash"
    );

    // Terminal should still be functional
    harness.send_input("echo 'After crash'\n")?;
    thread::sleep(Duration::from_millis(500));

    let found = harness.verify_output_contains("After crash", Duration::from_secs(2))?;
    assert!(found, "Terminal should still work after client crash");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_state_preservation() -> Result<()> {
    println!("\n=== Test: State Preservation ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Create some terminal state
    harness.send_input("export TEST_VAR='preserved'\n")?;
    thread::sleep(Duration::from_millis(500));

    harness.send_input("cd /tmp\n")?;
    thread::sleep(Duration::from_millis(500));

    // Get sequence number before disconnect
    let state_before = harness.get_shared_state()?;
    let seq_before = state_before.sequence_number;

    // Disconnect client
    harness.disconnect_client()?;
    thread::sleep(Duration::from_secs(1));

    // Reconnect
    harness.reconnect_client()?;
    thread::sleep(Duration::from_millis(500));

    // Verify state is preserved
    harness.send_input("echo $TEST_VAR\n")?;
    let found = harness.verify_output_contains("preserved", Duration::from_secs(2))?;
    assert!(found, "Environment variable should be preserved");

    harness.send_input("pwd\n")?;
    let found = harness.verify_output_contains("/tmp", Duration::from_secs(2))?;
    assert!(found, "Working directory should be preserved");

    // Sequence should have continued incrementing
    let state_after = harness.get_shared_state()?;
    assert!(
        state_after.sequence_number >= seq_before,
        "Sequence number should be preserved or incremented"
    );

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_multiple_disconnect_cycles() -> Result<()> {
    println!("\n=== Test: Multiple Disconnect Cycles ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Perform multiple disconnect/reconnect cycles
    for i in 1..=3 {
        println!("Cycle {}/3", i);

        // Send a command
        harness.send_input(&format!("echo 'Cycle {}'\n", i))?;
        thread::sleep(Duration::from_millis(500));

        // Disconnect
        harness.disconnect_client()?;
        thread::sleep(Duration::from_millis(500));

        // Verify daemon is alive
        assert!(
            harness.daemon_is_alive(),
            "Daemon should be alive in cycle {}",
            i
        );

        // Reconnect
        harness.reconnect_client()?;
        thread::sleep(Duration::from_millis(500));
    }

    // Final verification
    harness.send_input("echo 'All cycles complete'\n")?;
    let found = harness.verify_output_contains("All cycles complete", Duration::from_secs(2))?;
    assert!(found, "Should work after multiple cycles");

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
fn test_long_running_process() -> Result<()> {
    println!("\n=== Test: Long Running Process ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Start a long-running process
    harness.send_input("sleep 10 &\n")?;
    thread::sleep(Duration::from_millis(500));

    harness.send_input("echo 'Process started'\n")?;
    thread::sleep(Duration::from_millis(500));

    // Disconnect client
    harness.disconnect_client()?;

    // Wait a bit (but not the full 10 seconds)
    thread::sleep(Duration::from_secs(2));

    // Reconnect
    harness.reconnect_client()?;

    // Verify terminal is responsive
    harness.send_input("echo 'Still responsive'\n")?;
    let found = harness.verify_output_contains("Still responsive", Duration::from_secs(2))?;
    assert!(
        found,
        "Terminal should be responsive with background process"
    );

    println!("=== Test Passed ===\n");
    Ok(())
}
