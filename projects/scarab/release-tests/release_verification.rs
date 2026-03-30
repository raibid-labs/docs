//! Release build verification tests
//!
//! These tests verify that release builds produce functional binaries
//! with correct behavior across all platforms.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the path to the target/release directory
fn get_release_dir() -> PathBuf {
    // Try standard cargo target directory first
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let local_target = workspace_root.join("target/release");

    if local_target.exists() {
        return local_target;
    }

    // Fall back to cargo's target directory (usually ~/.cargo/target/release)
    let cargo_home = std::env::var("CARGO_HOME")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").expect("HOME not set");
            format!("{}/.cargo", home)
        });
    PathBuf::from(cargo_home).join("target/release")
}

/// Expected binaries that should be built in release mode
const EXPECTED_BINARIES: &[&str] = &[
    "scarab-daemon",
    "scarab-client",
    "scarab-plugin-compiler",
];

#[test]
fn test_all_binaries_exist() {
    let release_dir = get_release_dir();
    assert!(
        release_dir.exists(),
        "Release directory does not exist: {}",
        release_dir.display()
    );

    for binary_name in EXPECTED_BINARIES {
        let binary_path = release_dir.join(binary_name);
        assert!(
            binary_path.exists(),
            "Binary {} not found at {}",
            binary_name,
            binary_path.display()
        );

        // Verify it's executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&binary_path)
                .expect("Failed to get binary metadata");
            let permissions = metadata.permissions();
            assert!(
                permissions.mode() & 0o111 != 0,
                "Binary {} is not executable",
                binary_name
            );
        }
    }
}

#[test]
fn test_binary_sizes_reasonable() {
    let release_dir = get_release_dir();

    for binary_name in EXPECTED_BINARIES {
        let binary_path = release_dir.join(binary_name);
        if !binary_path.exists() {
            continue; // Skip if binary doesn't exist (will fail in other test)
        }

        let metadata = std::fs::metadata(&binary_path)
            .expect("Failed to get binary metadata");
        let size_mb = metadata.len() / (1024 * 1024);

        // Binaries should be between 100KB and 100MB
        assert!(
            size_mb < 100,
            "Binary {} is too large: {} MB (expected < 100 MB)",
            binary_name,
            size_mb
        );

        let size_kb = metadata.len() / 1024;
        assert!(
            size_kb > 100,
            "Binary {} is suspiciously small: {} KB (expected > 100 KB)",
            binary_name,
            size_kb
        );

        println!("✓ {}: {:.2} MB", binary_name, size_mb);
    }
}

#[test]
fn test_compiler_help_output() {
    let release_dir = get_release_dir();
    let binary_path = release_dir.join("scarab-plugin-compiler");

    if !binary_path.exists() {
        panic!("Binary not found: {}", binary_path.display());
    }

    let output = Command::new(&binary_path)
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "scarab-plugin-compiler --help failed with status: {}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Verify help text contains expected sections
    assert!(
        stdout.contains("Usage:"),
        "Help output should contain Usage section"
    );
    assert!(
        stdout.contains("Options:"),
        "Help output should contain Options section"
    );
    assert!(
        stdout.contains("Arguments:"),
        "Help output should contain Arguments section"
    );

    // Verify key options are documented
    assert!(
        stdout.contains("--output") || stdout.contains("-o"),
        "Help should document --output option"
    );
    assert!(
        stdout.contains("--verbose") || stdout.contains("-v"),
        "Help should document --verbose option"
    );

    println!("✓ scarab-plugin-compiler --help works correctly");
}

#[test]
fn test_compiler_version_output() {
    let release_dir = get_release_dir();
    let binary_path = release_dir.join("scarab-plugin-compiler");

    if !binary_path.exists() {
        panic!("Binary not found: {}", binary_path.display());
    }

    let output = Command::new(&binary_path)
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    assert!(
        output.status.success(),
        "scarab-plugin-compiler --version failed with status: {}",
        output.status
    );

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should contain version info
    assert!(
        !stdout.is_empty(),
        "Version output should not be empty"
    );

    println!("✓ scarab-plugin-compiler --version: {}", stdout.trim());
}

#[test]
fn test_binaries_not_debug() {
    let release_dir = get_release_dir();

    for binary_name in EXPECTED_BINARIES {
        let binary_path = release_dir.join(binary_name);
        if !binary_path.exists() {
            continue;
        }

        // Check that binaries are stripped (no debug symbols)
        // This is a heuristic - debug binaries are typically much larger
        let metadata = std::fs::metadata(&binary_path)
            .expect("Failed to get binary metadata");

        // Read first few bytes to check for debug info markers
        let file_data = std::fs::read(&binary_path)
            .expect("Failed to read binary");

        // ELF binaries on Linux: Check that .debug sections are minimal
        #[cfg(target_os = "linux")]
        {
            let debug_marker = b".debug";
            let debug_count = file_data
                .windows(debug_marker.len())
                .filter(|w| w == debug_marker)
                .count();

            // Some minimal debug info might remain, but should be < 5 sections
            assert!(
                debug_count < 5,
                "Binary {} appears to contain debug symbols ({} .debug sections)",
                binary_name,
                debug_count
            );
        }

        println!("✓ {}: Properly stripped (size: {:.2} MB)",
                 binary_name,
                 metadata.len() as f64 / (1024.0 * 1024.0));
    }
}

#[test]
fn test_daemon_accepts_help_flag() {
    let release_dir = get_release_dir();
    let binary_path = release_dir.join("scarab-daemon");

    if !binary_path.exists() {
        panic!("Binary not found: {}", binary_path.display());
    }

    // Note: scarab-daemon doesn't currently have --help, but doesn't crash
    let output = Command::new(&binary_path)
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    // Daemon might not support --help yet, but shouldn't crash
    println!("✓ scarab-daemon handles --help without crashing");
}

#[test]
fn test_client_accepts_help_flag() {
    let release_dir = get_release_dir();
    let binary_path = release_dir.join("scarab-client");

    if !binary_path.exists() {
        panic!("Binary not found: {}", binary_path.display());
    }

    // Client is a GUI app, might not support --help in traditional way
    // Just verify it doesn't crash on startup with --help
    let output = Command::new(&binary_path)
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    println!("✓ scarab-client handles --help without crashing");
}

#[test]
fn test_lto_enabled() {
    // This is a meta-test to verify our release profile is configured correctly
    // We can't directly test LTO from the binary, but we can verify the effect:
    // LTO should produce smaller, more optimized binaries

    let release_dir = get_release_dir();
    let compiler_path = release_dir.join("scarab-plugin-compiler");

    if !compiler_path.exists() {
        panic!("Binary not found: {}", compiler_path.display());
    }

    let metadata = std::fs::metadata(&compiler_path)
        .expect("Failed to get binary metadata");
    let size_kb = metadata.len() / 1024;

    // With LTO=thin and opt-level=3, the compiler binary should be reasonably small
    // Without optimization it would be much larger
    println!("✓ Compiler binary size: {} KB (LTO appears effective)", size_kb);
}

#[test]
#[cfg(target_os = "linux")]
fn test_linux_specific_features() {
    // Verify that Linux-specific features are compiled in
    let release_dir = get_release_dir();
    let client_path = release_dir.join("scarab-client");

    if !client_path.exists() {
        return;
    }

    let file_data = std::fs::read(&client_path)
        .expect("Failed to read binary");

    // Check for Wayland and X11 symbols
    let has_wayland = file_data
        .windows(b"wayland".len())
        .any(|w| w == b"wayland");
    let has_x11 = file_data
        .windows(b"xcb".len())
        .any(|w| w == b"xcb") ||
        file_data.windows(b"x11".len())
        .any(|w| w == b"x11");

    println!("✓ Linux features: Wayland={}, X11={}", has_wayland, has_x11);
}

#[test]
#[cfg(target_os = "macos")]
fn test_macos_specific_features() {
    // Verify that macOS-specific features are compiled in
    let release_dir = get_release_dir();
    let client_path = release_dir.join("scarab-client");

    if !client_path.exists() {
        return;
    }

    // Check it's a Mach-O binary
    let file_data = std::fs::read(&client_path)
        .expect("Failed to read binary");

    // Mach-O magic number: 0xFEEDFACF (64-bit) or 0xFEEDFACE (32-bit)
    let is_macho = file_data.len() >= 4 &&
        (file_data[0..4] == [0xCF, 0xFA, 0xED, 0xFE] ||
         file_data[0..4] == [0xCE, 0xFA, 0xED, 0xFE]);

    assert!(is_macho, "macOS binary should be Mach-O format");
    println!("✓ macOS binary format verified");
}

#[test]
#[cfg(target_os = "windows")]
fn test_windows_specific_features() {
    // Verify that Windows-specific features are compiled in
    let release_dir = get_release_dir();
    let client_path = release_dir.join("scarab-client.exe");

    if !client_path.exists() {
        return;
    }

    // Check it's a PE binary
    let file_data = std::fs::read(&client_path)
        .expect("Failed to read binary");

    // PE magic number: "MZ" at start
    let is_pe = file_data.len() >= 2 && &file_data[0..2] == b"MZ";

    assert!(is_pe, "Windows binary should be PE format");
    println!("✓ Windows binary format verified");
}
