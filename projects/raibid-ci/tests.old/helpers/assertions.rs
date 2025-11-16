//! Custom Assertion Helpers
//!
//! Provides custom assertions for common test patterns.

use std::process::Output;

/// Assert that a command output was successful and contains expected text
///
/// # Examples
/// ```no_run
/// use std::process::Command;
/// let output = Command::new("raibid-cli").arg("--version").output().unwrap();
/// assert_success_output(&output, "raibid-cli");
/// ```
pub fn assert_success_output(output: &Output, expected_text: &str) {
    assert!(
        output.status.success(),
        "Command failed with status: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(expected_text),
        "Output does not contain expected text.\nExpected: {}\nActual output: {}",
        expected_text,
        stdout
    );
}

/// Assert that a command output failed and contains expected error text
///
/// # Examples
/// ```no_run
/// use std::process::Command;
/// let output = Command::new("raibid-cli").arg("invalid").output().unwrap();
/// assert_error_contains(&output, "Unknown command");
/// ```
pub fn assert_error_contains(output: &Output, expected_error: &str) {
    assert!(
        !output.status.success(),
        "Command succeeded but was expected to fail.\nstdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{}{}", stderr, stdout);

    assert!(
        combined.contains(expected_error),
        "Error output does not contain expected text.\nExpected: {}\nstderr: {}\nstdout: {}",
        expected_error,
        stderr,
        stdout
    );
}

/// Assert that a file exists and contains expected content
pub fn assert_file_contains(path: &std::path::Path, expected_content: &str) {
    assert!(
        path.exists(),
        "File does not exist: {:?}",
        path
    );

    let content = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read file: {:?}", path));

    assert!(
        content.contains(expected_content),
        "File does not contain expected content.\nFile: {:?}\nExpected: {}\nActual: {}",
        path,
        expected_content,
        content
    );
}

/// Assert that a directory exists and is not empty
pub fn assert_dir_not_empty(path: &std::path::Path) {
    assert!(
        path.exists(),
        "Directory does not exist: {:?}",
        path
    );

    assert!(
        path.is_dir(),
        "Path is not a directory: {:?}",
        path
    );

    let entries = std::fs::read_dir(path)
        .unwrap_or_else(|_| panic!("Failed to read directory: {:?}", path));

    let count = entries.count();
    assert!(
        count > 0,
        "Directory is empty: {:?}",
        path
    );
}

/// Assert that YAML is valid and can be parsed
pub fn assert_valid_yaml(yaml_str: &str) {
    serde_yaml::from_str::<serde_yaml::Value>(yaml_str)
        .expect("Invalid YAML");
}

/// Assert that JSON is valid and can be parsed
pub fn assert_valid_json(json_str: &str) {
    serde_json::from_str::<serde_json::Value>(json_str)
        .expect("Invalid JSON");
}

/// Assert that a string matches a regex pattern
pub fn assert_matches_regex(text: &str, pattern: &str) {
    let regex = regex::Regex::new(pattern)
        .unwrap_or_else(|_| panic!("Invalid regex pattern: {}", pattern));

    assert!(
        regex.is_match(text),
        "Text does not match regex pattern.\nPattern: {}\nText: {}",
        pattern,
        text
    );
}

/// Assert that two durations are approximately equal (within tolerance)
pub fn assert_duration_approx_eq(
    actual: std::time::Duration,
    expected: std::time::Duration,
    tolerance_ms: u64,
) {
    let diff = if actual > expected {
        actual - expected
    } else {
        expected - actual
    };

    assert!(
        diff.as_millis() <= tolerance_ms as u128,
        "Durations not approximately equal.\nExpected: {:?}\nActual: {:?}\nDifference: {:?}\nTolerance: {}ms",
        expected,
        actual,
        diff,
        tolerance_ms
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Output};

    fn mock_success_output(stdout: &str) -> Output {
        Output {
            status: std::process::ExitStatus::default(),
            stdout: stdout.as_bytes().to_vec(),
            stderr: vec![],
        }
    }

    #[test]
    fn test_assert_valid_yaml() {
        let valid_yaml = "key: value\nlist:\n  - item1\n  - item2";
        assert_valid_yaml(valid_yaml);
    }

    #[test]
    #[should_panic(expected = "Invalid YAML")]
    fn test_assert_valid_yaml_fails() {
        let invalid_yaml = "key: value: invalid: [";
        assert_valid_yaml(invalid_yaml);
    }

    #[test]
    fn test_assert_valid_json() {
        let valid_json = r#"{"key": "value", "list": [1, 2, 3]}"#;
        assert_valid_json(valid_json);
    }

    #[test]
    #[should_panic(expected = "Invalid JSON")]
    fn test_assert_valid_json_fails() {
        let invalid_json = r#"{"key": "value",}"#;
        assert_valid_json(invalid_json);
    }

    #[test]
    fn test_assert_matches_regex() {
        assert_matches_regex("version 1.2.3", r"version \d+\.\d+\.\d+");
    }

    #[test]
    #[should_panic]
    fn test_assert_matches_regex_fails() {
        assert_matches_regex("no version here", r"version \d+\.\d+\.\d+");
    }

    #[test]
    fn test_assert_duration_approx_eq() {
        use std::time::Duration;

        let d1 = Duration::from_millis(100);
        let d2 = Duration::from_millis(105);

        assert_duration_approx_eq(d1, d2, 10);
    }

    #[test]
    #[should_panic]
    fn test_assert_duration_approx_eq_fails() {
        use std::time::Duration;

        let d1 = Duration::from_millis(100);
        let d2 = Duration::from_millis(200);

        assert_duration_approx_eq(d1, d2, 10);
    }
}
