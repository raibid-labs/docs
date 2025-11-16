//! Test Data Generators

/// Generate a test configuration
pub fn generate_test_config() -> String {
    r#"cluster:
  name: test-cluster

api:
  port: 8080

agents:
  rust:
    enabled: true
"#.to_string()
}

/// Generate a commit hash for testing
pub fn generate_commit_hash() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..40)
        .map(|_| format!("{:x}", rng.gen_range(0..16)))
        .collect()
}
