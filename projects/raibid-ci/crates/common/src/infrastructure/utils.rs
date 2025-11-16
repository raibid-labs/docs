//! Utility functions for infrastructure installation
//!
//! This module provides helper functions for permission checking, PATH detection,
//! and generating helpful error messages.

use anyhow::{anyhow, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Check if a directory is writable by attempting to create a test file
pub fn check_directory_writable(dir: &Path) -> Result<()> {
    debug!("Checking if directory is writable: {:?}", dir);

    // If directory doesn't exist, try to create it
    if !dir.exists() {
        fs::create_dir_all(dir).with_context(|| {
            format!(
                "Failed to create install directory: {}\n\n{}",
                dir.display(),
                permission_denied_help(dir)
            )
        })?;
    }

    // Try to create a test file
    let test_file = dir.join(".raibid-write-test");
    match fs::write(&test_file, b"test") {
        Ok(_) => {
            // Clean up test file
            let _ = fs::remove_file(&test_file);
            debug!("Directory is writable: {:?}", dir);
            Ok(())
        }
        Err(e) => Err(anyhow!(
            "Cannot write to installation directory: {}\n\n{}",
            dir.display(),
            permission_denied_help(dir)
        ))
        .context(format!("Permission check failed: {}", e)),
    }
}

/// Check if a directory is in the PATH environment variable
pub fn is_directory_in_path(dir: &Path) -> bool {
    if let Ok(path_env) = std::env::var("PATH") {
        let canonical_dir = dir.canonicalize().ok();

        for path_entry in path_env.split(':') {
            let entry_path = PathBuf::from(path_entry);

            // Try to canonicalize the entry to handle symlinks
            if let Ok(canonical_entry) = entry_path.canonicalize() {
                if canonical_dir.as_ref() == Some(&canonical_entry) {
                    return true;
                }
            }

            // Also check direct match
            if entry_path == dir {
                return true;
            }
        }
    }

    false
}

/// Generate a helpful error message for permission denied errors
pub fn permission_denied_help(dir: &Path) -> String {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/user"));
    let user_local = home.join(".local").join("bin");

    format!(
        r#"Permission denied when trying to write to: {}

SOLUTIONS:

1. Use user-local installation (recommended, no sudo required):
   Set install_dir to: {}

   This directory will be created automatically if it doesn't exist.
   You may need to add it to your PATH:

   For bash/zsh, add to your ~/.bashrc or ~/.zshrc:
   export PATH="$HOME/.local/bin:$PATH"

2. Install to system directory with sudo:
   If you need system-wide installation, you can:
   - Run the entire command with sudo (not recommended)
   - Or manually copy the binaries after installation

3. Use a custom directory:
   Create a custom directory you own and configure raibid-cli to use it.

For more information, see the troubleshooting section in the README.
"#,
        dir.display(),
        user_local.display()
    )
}

/// Get the default user-local bin directory
#[allow(dead_code)]
pub fn get_user_local_bin() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/root"));
    home.join(".local").join("bin")
}

/// Warn if install directory is not in PATH
pub fn warn_if_not_in_path(dir: &Path) {
    if !is_directory_in_path(dir) {
        warn!(
            "\nInstallation directory is not in your PATH: {}\n\
             \n\
             To use the installed binaries, add the directory to your PATH:\n\
             \n\
             For bash/zsh, add to your ~/.bashrc or ~/.zshrc:\n\
             export PATH=\"{}:$PATH\"\n\
             \n\
             Then reload your shell:\n\
             source ~/.bashrc  # or source ~/.zshrc\n",
            dir.display(),
            dir.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn test_check_directory_writable_temp() {
        let temp_dir = std::env::temp_dir();
        let result = check_directory_writable(&temp_dir);
        assert!(result.is_ok(), "Temp directory should be writable");
    }

    #[test]
    fn test_check_directory_writable_new_dir() {
        let test_dir = std::env::temp_dir().join("raibid-test-utils-new");
        let _ = fs::remove_dir_all(&test_dir);

        let result = check_directory_writable(&test_dir);
        assert!(
            result.is_ok(),
            "Should be able to create and write to new directory"
        );

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }

    #[test]
    #[cfg(unix)]
    fn test_check_directory_writable_readonly() {
        let test_dir = std::env::temp_dir().join("raibid-test-utils-readonly");
        fs::create_dir_all(&test_dir).expect("Should create test directory");

        // Make it read-only
        let mut perms = fs::metadata(&test_dir).unwrap().permissions();
        perms.set_mode(0o444);
        fs::set_permissions(&test_dir, perms).expect("Should set permissions");

        let result = check_directory_writable(&test_dir);

        // Restore permissions
        let mut perms = fs::metadata(&test_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_dir, perms).expect("Should restore permissions");

        // Clean up
        let _ = fs::remove_dir_all(&test_dir);

        assert!(result.is_err(), "Should detect read-only directory");
    }

    #[test]
    fn test_is_directory_in_path() {
        let path_env = std::env::var("PATH").expect("PATH should be set");
        let first_dir = path_env
            .split(':')
            .next()
            .expect("PATH should have entries");

        let result = is_directory_in_path(&PathBuf::from(first_dir));
        assert!(result, "First PATH directory should be detected");
    }

    #[test]
    fn test_is_directory_not_in_path() {
        let random_dir = PathBuf::from("/tmp/not-in-path-12345");
        let result = is_directory_in_path(&random_dir);
        assert!(!result, "Random directory should not be in PATH");
    }

    #[test]
    fn test_permission_denied_help() {
        let dir = PathBuf::from("/usr/local/bin");
        let message = permission_denied_help(&dir);

        assert!(message.contains("Permission denied"));
        assert!(message.contains(".local/bin"));
        assert!(message.contains("PATH"));
    }

    #[test]
    fn test_get_user_local_bin() {
        let user_local = get_user_local_bin();
        assert!(user_local.to_string_lossy().contains(".local"));
        assert!(user_local.to_string_lossy().ends_with("bin"));
    }
}
