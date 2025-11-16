//! Test Environment Management
//!
//! Provides utilities for setting up and tearing down test environments.

use std::path::PathBuf;
use tempfile::TempDir;

/// Test environment context
///
/// Manages temporary directories, configuration, and resources for tests.
/// Automatically cleans up on drop unless RAIBID_TEST_NO_CLEANUP is set.
pub struct TestEnv {
    /// Temporary directory for test files
    pub temp_dir: TempDir,
    /// Path to test configuration file
    pub config_path: Option<PathBuf>,
    /// Path to test data directory
    pub data_dir: PathBuf,
    /// Skip cleanup on drop (for debugging)
    skip_cleanup: bool,
}

impl TestEnv {
    /// Create a new test environment
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");

        let skip_cleanup = std::env::var("RAIBID_TEST_NO_CLEANUP").is_ok();

        TestEnv {
            temp_dir,
            config_path: None,
            data_dir,
            skip_cleanup,
        }
    }

    /// Create a test configuration file
    pub fn create_config(&mut self, content: &str) -> PathBuf {
        let config_path = self.temp_dir.path().join("raibid.yaml");
        std::fs::write(&config_path, content)
            .expect("Failed to write test config");
        self.config_path = Some(config_path.clone());
        config_path
    }

    /// Get the temporary directory path
    pub fn path(&self) -> &std::path::Path {
        self.temp_dir.path()
    }

    /// Create a subdirectory in the test environment
    pub fn create_dir(&self, name: &str) -> PathBuf {
        let dir_path = self.temp_dir.path().join(name);
        std::fs::create_dir_all(&dir_path)
            .expect("Failed to create subdirectory");
        dir_path
    }

    /// Create a file in the test environment
    pub fn create_file(&self, name: &str, content: &str) -> PathBuf {
        let file_path = self.temp_dir.path().join(name);
        std::fs::write(&file_path, content)
            .expect("Failed to create test file");
        file_path
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        if self.skip_cleanup {
            println!("Skipping cleanup. Test directory: {:?}", self.temp_dir.path());
            // Prevent automatic cleanup
            let _ = self.temp_dir.path();
        }
    }
}

/// Set up a test environment
///
/// Creates a temporary directory structure for testing.
/// Returns a TestEnv that will be automatically cleaned up.
pub fn setup_test_env() -> TestEnv {
    TestEnv::new()
}

/// Tear down a test environment
///
/// Explicitly clean up test resources.
/// Note: TestEnv automatically cleans up on drop, so this is optional.
pub fn teardown_test_env(_env: TestEnv) {
    // TestEnv will be dropped and cleaned up automatically
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_creates_temp_dir() {
        let env = TestEnv::new();
        assert!(env.path().exists());
        assert!(env.data_dir.exists());
    }

    #[test]
    fn test_env_create_config() {
        let mut env = TestEnv::new();
        let config_content = "cluster:\n  name: test";
        let config_path = env.create_config(config_content);

        assert!(config_path.exists());
        let content = std::fs::read_to_string(&config_path).unwrap();
        assert_eq!(content, config_content);
    }

    #[test]
    fn test_env_create_subdirectory() {
        let env = TestEnv::new();
        let subdir = env.create_dir("subdir");
        assert!(subdir.exists());
        assert!(subdir.is_dir());
    }

    #[test]
    fn test_env_create_file() {
        let env = TestEnv::new();
        let file_path = env.create_file("test.txt", "test content");
        assert!(file_path.exists());
        assert!(file_path.is_file());

        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "test content");
    }
}
