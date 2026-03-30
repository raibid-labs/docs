//! Test Scenario 8: Stress Test
//!
//! Long-running stability tests:
//! - 1-hour continuous operation
//! - Memory leak detection
//! - Performance degradation checks
//! - Crash resistance

use super::harness::E2ETestHarness;
use anyhow::Result;
use std::thread;
use std::time::{Duration, Instant};

#[test]
#[ignore] // Run manually: cargo test stress_1_hour -- --ignored
fn test_stress_1_hour() -> Result<()> {
    println!("\n=== Test: 1-Hour Stress Test ===");
    println!("This test will run for 1 hour. Press Ctrl+C to abort.");

    let mut harness = E2ETestHarness::new()?;

    let start = Instant::now();
    let duration = Duration::from_secs(3600); // 1 hour
    let mut iteration = 0;

    while start.elapsed() < duration {
        iteration += 1;

        // Cycle through various commands
        match iteration % 6 {
            0 => {
                harness.send_input("ls -la /tmp\n")?;
                thread::sleep(Duration::from_millis(500));
            }
            1 => {
                harness.send_input("echo 'Stress test iteration {}'\n")?;
                thread::sleep(Duration::from_millis(500));
            }
            2 => {
                harness.send_input("pwd\n")?;
                thread::sleep(Duration::from_millis(500));
            }
            3 => {
                harness.send_input("seq 1 100\n")?;
                thread::sleep(Duration::from_secs(1));
            }
            4 => {
                harness.send_input("cat /etc/hostname\n")?;
                thread::sleep(Duration::from_millis(500));
            }
            5 => {
                harness.send_input("date\n")?;
                thread::sleep(Duration::from_millis(500));
            }
            _ => unreachable!(),
        }

        // Every 100 iterations, check health
        if iteration % 100 == 0 {
            let elapsed = start.elapsed();
            println!(
                "[{:02}:{:02}:{:02}] Iteration {}: Daemon alive: {}, Elapsed: {:.2}s",
                elapsed.as_secs() / 3600,
                (elapsed.as_secs() % 3600) / 60,
                elapsed.as_secs() % 60,
                iteration,
                harness.daemon_is_alive(),
                elapsed.as_secs_f64()
            );

            // Verify daemon is still alive
            assert!(
                harness.daemon_is_alive(),
                "Daemon died during stress test at iteration {}",
                iteration
            );

            // Check shared state is still accessible
            let state = harness.get_shared_state()?;
            println!("  Sequence number: {}", state.sequence_number);
        }

        // Every 500 iterations, test client reconnection
        if iteration % 500 == 0 && iteration > 0 {
            println!("Testing client reconnection...");
            harness.disconnect_client()?;
            thread::sleep(Duration::from_secs(1));
            harness.reconnect_client()?;
            println!("Reconnection successful");
        }
    }

    println!("\n=== Stress Test Completed Successfully ===");
    println!("Total iterations: {}", iteration);
    println!("Duration: {} seconds", start.elapsed().as_secs());

    Ok(())
}

#[test]
#[ignore] // Run manually: cargo test stress_short -- --ignored
fn test_stress_short() -> Result<()> {
    println!("\n=== Test: Short Stress Test (5 minutes) ===");

    let mut harness = E2ETestHarness::new()?;

    let start = Instant::now();
    let duration = Duration::from_secs(300); // 5 minutes
    let mut iteration = 0;

    while start.elapsed() < duration {
        iteration += 1;

        // Rapid command execution
        harness.send_input(&format!("echo 'Test {}'\n", iteration))?;
        thread::sleep(Duration::from_millis(100));

        if iteration % 50 == 0 {
            println!(
                "Iteration {}, elapsed: {:.1}s",
                iteration,
                start.elapsed().as_secs_f64()
            );
            assert!(
                harness.daemon_is_alive(),
                "Daemon died at iteration {}",
                iteration
            );
        }
    }

    println!("=== Short Stress Test Passed ===");
    println!("Total iterations: {}", iteration);

    Ok(())
}

#[test]
#[ignore]
fn test_memory_stability() -> Result<()> {
    println!("\n=== Test: Memory Stability ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Generate large amounts of output to test memory management
    for i in 1..=50 {
        println!("Memory test iteration {}/50", i);

        // Generate 1000 lines
        harness.send_input("seq 1 1000\n")?;
        thread::sleep(Duration::from_secs(2));

        // Clear
        harness.send_input("clear\n")?;
        thread::sleep(Duration::from_millis(500));

        // Verify daemon is still alive
        assert!(harness.daemon_is_alive(), "Daemon died during memory test");

        // Check state is accessible
        let state = harness.get_shared_state()?;
        println!("  Sequence: {}", state.sequence_number);
    }

    println!("=== Memory Stability Test Passed ===");

    Ok(())
}

#[test]
#[ignore]
fn test_rapid_input_stress() -> Result<()> {
    println!("\n=== Test: Rapid Input Stress ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Send massive amounts of input
    for i in 0..1000 {
        harness.send_input("x")?;

        if i % 100 == 0 {
            println!("Sent {} characters", i);
        }
    }

    // Clear the input buffer
    harness.send_input("\x03")?; // Ctrl+C
    thread::sleep(Duration::from_millis(500));

    // Verify terminal is responsive
    harness.send_input("echo 'Rapid input test done'\n")?;

    let found = harness.verify_output_contains("Rapid input test done", Duration::from_secs(3))?;
    assert!(found, "Terminal should recover from rapid input");

    println!("=== Rapid Input Stress Test Passed ===");

    Ok(())
}

#[test]
#[ignore]
fn test_concurrent_commands() -> Result<()> {
    println!("\n=== Test: Concurrent Commands ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Start multiple background processes
    for i in 1..=5 {
        harness.send_input(&format!("sleep {} &\n", i))?;
        thread::sleep(Duration::from_millis(200));
    }

    // Run foreground commands while background jobs are active
    for i in 1..=20 {
        harness.send_input(&format!("echo 'Foreground {}'\n", i))?;
        thread::sleep(Duration::from_millis(300));
    }

    // Check jobs
    harness.send_input("jobs\n")?;
    thread::sleep(Duration::from_secs(1));

    // Wait for background jobs to complete
    thread::sleep(Duration::from_secs(6));

    // Verify terminal is still responsive
    harness.send_input("echo 'Concurrent test done'\n")?;

    let found = harness.verify_output_contains("Concurrent test done", Duration::from_secs(2))?;
    assert!(found, "Should handle concurrent commands");

    println!("=== Concurrent Commands Test Passed ===");

    Ok(())
}

#[test]
#[ignore]
fn test_resize_stress() -> Result<()> {
    println!("\n=== Test: Resize Stress ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Perform many resizes rapidly
    for i in 0..200 {
        let size = 80 + (i % 40) * 2;
        harness.resize(size, 24)?;

        if i % 50 == 0 {
            println!("Resize iteration {}/200", i);
        }

        // Occasionally send commands during resize
        if i % 20 == 0 {
            harness.send_input("echo 'test'\n")?;
            thread::sleep(Duration::from_millis(100));
        }
    }

    // Verify terminal is still functional
    harness.send_input("echo 'Resize stress done'\n")?;

    let found = harness.verify_output_contains("Resize stress done", Duration::from_secs(2))?;
    assert!(found, "Should survive resize stress");

    println!("=== Resize Stress Test Passed ===");

    Ok(())
}

#[test]
#[ignore]
fn test_disconnect_stress() -> Result<()> {
    println!("\n=== Test: Disconnect Stress ===");

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Perform many disconnect/reconnect cycles
    for i in 1..=20 {
        println!("Disconnect cycle {}/20", i);

        // Send command
        harness.send_input(&format!("echo 'Cycle {}'\n", i))?;
        thread::sleep(Duration::from_millis(300));

        // Disconnect and reconnect
        harness.disconnect_client()?;
        thread::sleep(Duration::from_millis(200));
        harness.reconnect_client()?;
        thread::sleep(Duration::from_millis(300));
    }

    // Final verification
    harness.send_input("echo 'Disconnect stress done'\n")?;

    let found = harness.verify_output_contains("Disconnect stress done", Duration::from_secs(2))?;
    assert!(found, "Should survive many disconnect cycles");

    println!("=== Disconnect Stress Test Passed ===");

    Ok(())
}
