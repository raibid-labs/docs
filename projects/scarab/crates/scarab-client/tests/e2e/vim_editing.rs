//! Test Scenario 2: Vim Editing Session
//!
//! Tests interaction with vim editor:
//! - Opening vim
//! - Insert mode
//! - Text input
//! - Saving and quitting
//! - File verification

use super::harness::E2ETestHarness;
use anyhow::Result;
use std::fs;
use std::thread;
use std::time::Duration;

#[test]
#[ignore] // Vim may not be available in all test environments
fn test_vim_basic_editing() -> Result<()> {
    println!("\n=== Test: Vim Basic Editing ===");

    // Check if vim is available
    let vim_check = std::process::Command::new("which").arg("vim").output()?;

    if !vim_check.status.success() {
        println!("Vim not available, skipping test");
        return Ok(());
    }

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    // Create temp file path
    let temp_file = "/tmp/scarab_test_vim.txt";

    // Remove file if it exists
    let _ = fs::remove_file(temp_file);

    // Open vim
    println!("Opening vim...");
    harness.send_input(&format!("vim {}\n", temp_file))?;
    thread::sleep(Duration::from_secs(1));

    // Enter insert mode
    println!("Entering insert mode...");
    harness.send_input("i")?;
    thread::sleep(Duration::from_millis(200));

    // Type text
    println!("Typing text...");
    harness.send_input("Hello from Scarab\n")?;
    harness.send_input("This is a test file\n")?;
    thread::sleep(Duration::from_millis(500));

    // Exit insert mode and save
    println!("Saving and quitting...");
    harness.send_input("\x1b")?; // ESC
    thread::sleep(Duration::from_millis(200));
    harness.send_input(":wq\n")?;

    // Wait for vim to exit
    thread::sleep(Duration::from_secs(1));

    // Verify file was created and contains expected content
    let contents = fs::read_to_string(temp_file).expect("File should have been created");

    assert!(contents.contains("Hello from Scarab"), "First line missing");
    assert!(
        contents.contains("This is a test file"),
        "Second line missing"
    );

    // Cleanup
    fs::remove_file(temp_file)?;

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
#[ignore]
fn test_vim_quit_without_saving() -> Result<()> {
    println!("\n=== Test: Vim Quit Without Saving ===");

    let vim_check = std::process::Command::new("which").arg("vim").output()?;

    if !vim_check.status.success() {
        println!("Vim not available, skipping test");
        return Ok(());
    }

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    let temp_file = "/tmp/scarab_test_vim_nosave.txt";
    let _ = fs::remove_file(temp_file);

    // Open vim
    harness.send_input(&format!("vim {}\n", temp_file))?;
    thread::sleep(Duration::from_secs(1));

    // Enter insert mode and type
    harness.send_input("i")?;
    thread::sleep(Duration::from_millis(200));
    harness.send_input("This should not be saved")?;
    thread::sleep(Duration::from_millis(500));

    // Quit without saving
    harness.send_input("\x1b")?; // ESC
    thread::sleep(Duration::from_millis(200));
    harness.send_input(":q!\n")?;

    thread::sleep(Duration::from_secs(1));

    // Verify file was NOT created
    assert!(
        !std::path::Path::new(temp_file).exists(),
        "File should not exist"
    );

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
#[ignore]
fn test_vim_navigation() -> Result<()> {
    println!("\n=== Test: Vim Navigation ===");

    let vim_check = std::process::Command::new("which").arg("vim").output()?;

    if !vim_check.status.success() {
        println!("Vim not available, skipping test");
        return Ok(());
    }

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    let temp_file = "/tmp/scarab_test_vim_nav.txt";
    let _ = fs::remove_file(temp_file);

    // Create a test file with content
    fs::write(temp_file, "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\n")?;

    // Open vim
    harness.send_input(&format!("vim {}\n", temp_file))?;
    thread::sleep(Duration::from_secs(1));

    // Navigate down (j key)
    harness.send_input("jjj")?; // Move down 3 lines
    thread::sleep(Duration::from_millis(200));

    // Go to end of line ($)
    harness.send_input("$")?;
    thread::sleep(Duration::from_millis(200));

    // Go to beginning of file (gg)
    harness.send_input("gg")?;
    thread::sleep(Duration::from_millis(200));

    // Go to end of file (G)
    harness.send_input("G")?;
    thread::sleep(Duration::from_millis(200));

    // Quit without changes
    harness.send_input(":q\n")?;

    thread::sleep(Duration::from_secs(1));

    // Cleanup
    fs::remove_file(temp_file)?;

    println!("=== Test Passed ===\n");
    Ok(())
}

#[test]
#[ignore]
fn test_vim_search() -> Result<()> {
    println!("\n=== Test: Vim Search ===");

    let vim_check = std::process::Command::new("which").arg("vim").output()?;

    if !vim_check.status.success() {
        println!("Vim not available, skipping test");
        return Ok(());
    }

    let mut harness = E2ETestHarness::new()?;

    thread::sleep(Duration::from_secs(1));

    let temp_file = "/tmp/scarab_test_vim_search.txt";
    let _ = fs::remove_file(temp_file);

    // Create test file
    fs::write(
        temp_file,
        "The quick brown fox\njumps over the lazy dog\nfox and dog are friends\n",
    )?;

    // Open vim
    harness.send_input(&format!("vim {}\n", temp_file))?;
    thread::sleep(Duration::from_secs(1));

    // Search for "fox"
    harness.send_input("/fox\n")?;
    thread::sleep(Duration::from_millis(500));

    // Find next occurrence
    harness.send_input("n")?;
    thread::sleep(Duration::from_millis(200));

    // Quit
    harness.send_input(":q\n")?;
    thread::sleep(Duration::from_secs(1));

    // Cleanup
    fs::remove_file(temp_file)?;

    println!("=== Test Passed ===\n");
    Ok(())
}
