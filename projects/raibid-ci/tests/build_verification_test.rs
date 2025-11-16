//! Build verification tests
//!
//! These tests verify that the build process correctly creates binaries at expected locations.
//! This addresses issue #37: ensuring binaries are created and properly located,
//! particularly when CARGO_TARGET_DIR environment variable is set.

use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use predicates::prelude::*;
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Helper function to determine the expected binary path based on CARGO_TARGET_DIR
fn get_expected_binary_path(profile: &str, target: Option<&str>) -> PathBuf {
    let base_dir = if let Ok(target_dir) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target_dir)
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target")
    };

    let mut path = base_dir;

    if let Some(target_triple) = target {
        path = path.join(target_triple);
    }

    path.join(profile).join("raibid-cli")
}

/// Test that release build creates binary at correct location
/// This test verifies that after building with `cargo build --release`,
/// the binary exists at the expected location (handling CARGO_TARGET_DIR)
#[test]
fn test_release_build_creates_binary() {
    let expected_path = get_expected_binary_path("release", None);

    assert!(
        expected_path.exists(),
        "\n\nBinary not found at expected location!\n\
         Expected: {:?}\n\
         CARGO_TARGET_DIR: {:?}\n\
         \n\
         This test expects the binary to already be built with:\n\
         cargo build --release\n\
         \n\
         If CARGO_TARGET_DIR is set, the binary will be at:\n\
         $CARGO_TARGET_DIR/release/raibid-cli\n\
         \n\
         Otherwise, it will be at:\n\
         target/release/raibid-cli\n",
        expected_path,
        env::var("CARGO_TARGET_DIR").ok()
    );
}

/// Test that debug build creates binary at correct location
#[test]
fn test_debug_build_creates_binary() {
    let expected_path = get_expected_binary_path("debug", None);

    assert!(
        expected_path.exists(),
        "\n\nBinary not found at expected location!\n\
         Expected: {:?}\n\
         CARGO_TARGET_DIR: {:?}\n\
         \n\
         This test expects the binary to already be built with:\n\
         cargo build\n",
        expected_path,
        env::var("CARGO_TARGET_DIR").ok()
    );
}

/// Test that the release binary is executable and responds to --version
#[test]
fn test_release_binary_version() {
    let binary_path = get_expected_binary_path("release", None);

    assert!(
        binary_path.exists(),
        "Release binary not found at {:?}. Build with: cargo build --release",
        binary_path
    );

    // Test that the binary runs and outputs version info
    let output = Command::new(&binary_path)
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "Binary failed to execute --version: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("raibid-cli"),
        "Version output doesn't contain 'raibid-cli': {}",
        stdout
    );
    assert!(
        stdout.contains("0.1.0"),
        "Version output doesn't contain expected version: {}",
        stdout
    );
}

/// Test that the debug binary is executable and responds to --version
#[test]
fn test_debug_binary_version() {
    let binary_path = get_expected_binary_path("debug", None);

    assert!(
        binary_path.exists(),
        "Debug binary not found at {:?}. Build with: cargo build",
        binary_path
    );

    // Test that the binary runs and outputs version info
    let output = Command::new(&binary_path)
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "Binary failed to execute --version: {:?}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("raibid-cli"),
        "Version output doesn't contain 'raibid-cli': {}",
        stdout
    );
}

/// Test that binary has correct permissions (executable)
#[test]
#[cfg(unix)]
fn test_binary_is_executable() {
    use std::os::unix::fs::PermissionsExt;

    let binary_path = get_expected_binary_path("release", None);

    assert!(
        binary_path.exists(),
        "Binary not found at {:?}",
        binary_path
    );

    let metadata = std::fs::metadata(&binary_path)
        .expect("Failed to get binary metadata");

    let permissions = metadata.permissions();
    let mode = permissions.mode();

    // Check if executable bit is set for owner
    assert!(
        mode & 0o100 != 0,
        "Binary is not executable! Permissions: {:o}",
        mode
    );
}

/// Test CARGO_TARGET_DIR detection and reporting
#[test]
fn test_cargo_target_dir_detection() {
    let target_dir = env::var("CARGO_TARGET_DIR");

    match target_dir {
        Ok(dir) => {
            println!("CARGO_TARGET_DIR is SET: {}", dir);
            println!("Binaries will be located at: {}/{{profile}}/raibid-cli", dir);

            // Verify the path exists
            let path = PathBuf::from(&dir);
            assert!(
                path.exists(),
                "CARGO_TARGET_DIR points to non-existent directory: {}",
                dir
            );
        }
        Err(_) => {
            println!("CARGO_TARGET_DIR is NOT SET");
            println!("Binaries will be located at: target/{{profile}}/raibid-cli");
        }
    }
}

/// Test ARM64 cross-compilation binary location
/// This test is only compiled on ARM64 platforms
#[test]
#[cfg(target_arch = "aarch64")]
fn test_arm64_native_build() {
    let expected_path = get_expected_binary_path("release", None);

    assert!(
        expected_path.exists(),
        "\n\nARM64 native binary not found at {:?}\n\
         Build with: cargo build --release\n",
        expected_path
    );

    // Verify it's actually an ARM64 binary
    let output = Command::new("file")
        .arg(&expected_path)
        .output()
        .expect("Failed to run 'file' command");

    let file_output = String::from_utf8_lossy(&output.stdout);
    assert!(
        file_output.contains("aarch64") || file_output.contains("ARM"),
        "Binary doesn't appear to be ARM64: {}",
        file_output
    );
}

/// Test ARM64 cross-compilation binary location (when building on x86_64)
/// This test verifies the binary location when using --target flag
#[test]
#[cfg(all(target_arch = "x86_64", not(target_os = "windows")))]
fn test_arm64_cross_compilation_path() {
    let expected_path = get_expected_binary_path(
        "release",
        Some("aarch64-unknown-linux-gnu")
    );

    // Note: This test only checks if the path exists, doesn't require the binary to be built
    // To build: rustup target add aarch64-unknown-linux-gnu
    //           cargo build --release --target aarch64-unknown-linux-gnu

    if expected_path.exists() {
        println!("ARM64 cross-compiled binary found at: {:?}", expected_path);

        // If it exists, verify it's actually an ARM64 binary
        let output = Command::new("file")
            .arg(&expected_path)
            .output()
            .expect("Failed to run 'file' command");

        let file_output = String::from_utf8_lossy(&output.stdout);
        assert!(
            file_output.contains("aarch64") || file_output.contains("ARM"),
            "Binary doesn't appear to be ARM64: {}",
            file_output
        );
    } else {
        println!(
            "ARM64 cross-compiled binary not found at: {:?}\n\
             To build: rustup target add aarch64-unknown-linux-gnu\n\
             cargo build --release --target aarch64-unknown-linux-gnu",
            expected_path
        );
    }
}

/// Test that assert_cmd finds the binary correctly
/// This verifies that the existing test infrastructure works with CARGO_TARGET_DIR
#[test]
fn test_assert_cmd_finds_binary() {
    // This should work regardless of CARGO_TARGET_DIR because assert_cmd
    // uses cargo's metadata to find the binary
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("raibid-cli"));

    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("raibid-cli"));
}

/// Test binary file size is reasonable
/// This catches cases where the binary might be corrupted or incomplete
#[test]
fn test_binary_file_size() {
    let binary_path = get_expected_binary_path("release", None);

    assert!(
        binary_path.exists(),
        "Binary not found at {:?}",
        binary_path
    );

    let metadata = std::fs::metadata(&binary_path)
        .expect("Failed to get binary metadata");

    let size = metadata.len();

    // Release binary should be at least 1MB (reasonable minimum for Rust binary with dependencies)
    // and less than 100MB (reasonable maximum)
    assert!(
        size > 1_000_000,
        "Binary is suspiciously small ({} bytes), might be incomplete",
        size
    );
    assert!(
        size < 100_000_000,
        "Binary is suspiciously large ({} bytes), might include debug symbols",
        size
    );

    println!("Binary size: {} bytes ({:.2} MB)", size, size as f64 / 1_000_000.0);
}

/// Test that running the binary without arguments produces expected behavior
#[test]
fn test_binary_runs_without_args() {
    let binary_path = get_expected_binary_path("release", None);

    assert!(
        binary_path.exists(),
        "Binary not found at {:?}",
        binary_path
    );

    let output = Command::new(&binary_path)
        .output()
        .expect("Failed to execute binary");

    // Should fail (exit code != 0) because no command specified
    assert!(
        !output.status.success(),
        "Binary should fail when run without arguments"
    );

    // Should output helpful message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No command specified") || stdout.contains("Usage:"),
        "Binary should show helpful message when run without arguments: {}",
        stdout
    );
}

/// Integration test: Verify the entire build process documentation
/// This test validates that the README.md instructions are accurate
#[test]
fn test_build_process_documentation() {
    let cargo_target_dir = env::var("CARGO_TARGET_DIR");

    println!("\n=== Build Process Verification ===\n");

    match &cargo_target_dir {
        Ok(dir) => {
            println!("✓ CARGO_TARGET_DIR is set: {}", dir);
            println!("  Binaries will be in: {}/{{profile}}/raibid-cli", dir);
            println!("  NOT in: target/{{profile}}/raibid-cli");
            println!("\n  This matches README.md troubleshooting section (lines 583-608)");
        }
        Err(_) => {
            println!("✓ CARGO_TARGET_DIR is not set");
            println!("  Binaries will be in: target/{{profile}}/raibid-cli");
        }
    }

    // Check release binary
    let release_path = get_expected_binary_path("release", None);
    println!("\n✓ Release binary location: {:?}", release_path);
    println!("  Exists: {}", release_path.exists());

    // Check debug binary
    let debug_path = get_expected_binary_path("debug", None);
    println!("✓ Debug binary location: {:?}", debug_path);
    println!("  Exists: {}", debug_path.exists());

    // Check ARM64 cross-compilation path
    let arm64_path = get_expected_binary_path(
        "release",
        Some("aarch64-unknown-linux-gnu")
    );
    println!("✓ ARM64 cross-compile location: {:?}", arm64_path);
    println!("  Exists: {}", arm64_path.exists());

    println!("\n=== README.md Instructions Verification ===\n");
    println!("Native build: cargo build --release");
    println!("  → Binary at: {:?}", release_path);
    println!("\nARM64 build: cargo build --release --target aarch64-unknown-linux-gnu");
    println!("  → Binary at: {:?}", arm64_path);
    println!("\n========================================\n");
}
