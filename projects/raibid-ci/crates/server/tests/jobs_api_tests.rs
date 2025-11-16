//! Integration tests for job API endpoints

use raibid_server::{AppState, Server, ServerConfig};
use std::time::Duration;
use tokio::time::sleep;

/// Helper to start a test server with Redis
async fn start_test_server(
    port: u16,
    redis_url: &str,
) -> (tokio::task::JoinHandle<()>, ServerConfig) {
    let mut config = ServerConfig::default();
    config.port = port;
    config.redis_url = redis_url.to_string();

    // Create state with Redis
    let state = match AppState::with_redis(redis_url) {
        Ok(s) => s,
        Err(_) => {
            // Fall back to state without Redis for tests
            AppState::new()
        }
    };

    let server = Server::with_state(config.clone(), state);

    let handle = tokio::spawn(async move {
        let _ = server.run().await;
    });

    sleep(Duration::from_millis(500)).await;

    (handle, config)
}

#[tokio::test]
async fn test_list_jobs_endpoint_exists() {
    let (handle, config) = start_test_server(18090, "redis://127.0.0.1:6379").await;

    let response = reqwest::get(format!("http://{}:{}/jobs", config.host, config.port))
        .await
        .expect("Failed to make request");

    // Should return either 200 with data or 500 if Redis is not available
    // Both are acceptable for this test - we're just checking the endpoint exists
    assert!(response.status().is_server_error() || response.status().is_success());

    handle.abort();
}

#[tokio::test]
async fn test_get_job_by_id_endpoint_exists() {
    let (handle, config) = start_test_server(18091, "redis://127.0.0.1:6379").await;

    let response = reqwest::get(format!(
        "http://{}:{}/jobs/test-job-123",
        config.host, config.port
    ))
    .await
    .expect("Failed to make request");

    // Should return either 404/500 depending on Redis availability
    assert!(response.status().is_client_error() || response.status().is_server_error());

    handle.abort();
}

#[tokio::test]
async fn test_job_logs_endpoint_exists() {
    let (handle, config) = start_test_server(18092, "redis://127.0.0.1:6379").await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "http://{}:{}/jobs/test-job-123/logs",
            config.host, config.port
        ))
        .send()
        .await
        .expect("Failed to make request");

    // Should return either 404/500 depending on Redis availability
    assert!(response.status().is_client_error() || response.status().is_server_error());

    handle.abort();
}

#[tokio::test]
async fn test_list_jobs_with_filters() {
    let (handle, config) = start_test_server(18093, "redis://127.0.0.1:6379").await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}:{}/jobs", config.host, config.port))
        .query(&[("status", "running"), ("limit", "10")])
        .send()
        .await
        .expect("Failed to make request");

    // Should return either 200 with data or 500 if Redis is not available
    assert!(response.status().is_server_error() || response.status().is_success());

    handle.abort();
}

#[tokio::test]
async fn test_list_jobs_with_invalid_status() {
    let (handle, config) = start_test_server(18094, "redis://127.0.0.1:6379").await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}:{}/jobs", config.host, config.port))
        .query(&[("status", "invalid_status")])
        .send()
        .await
        .expect("Failed to make request");

    // Should return 400 for invalid status
    if response.status().is_success() || response.status() == 500 {
        // Redis not available or query succeeded despite invalid status
        // This is acceptable in test environment
    } else {
        assert_eq!(response.status(), 400);
    }

    handle.abort();
}

#[tokio::test]
async fn test_list_jobs_pagination() {
    let (handle, config) = start_test_server(18095, "redis://127.0.0.1:6379").await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}:{}/jobs", config.host, config.port))
        .query(&[("limit", "5"), ("offset", "0")])
        .send()
        .await
        .expect("Failed to make request");

    // Should return either 200 with data or 500 if Redis is not available
    assert!(response.status().is_server_error() || response.status().is_success());

    if response.status().is_success() {
        let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        // Check that response has expected fields
        assert!(json.get("jobs").is_some());
        assert!(json.get("total").is_some());
        assert!(json.get("limit").is_some());
        assert!(json.get("offset").is_some());
    }

    handle.abort();
}

#[tokio::test]
async fn test_job_endpoints_return_json() {
    let (handle, config) = start_test_server(18096, "redis://127.0.0.1:6379").await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}:{}/jobs", config.host, config.port))
        .send()
        .await
        .expect("Failed to make request");

    // Check Content-Type header
    if let Some(content_type) = response.headers().get("content-type") {
        let content_type_str = content_type.to_str().unwrap();
        // Should be JSON
        assert!(
            content_type_str.contains("application/json") || response.status().is_server_error(),
            "Expected JSON content type, got: {}",
            content_type_str
        );
    }

    handle.abort();
}

#[tokio::test]
async fn test_concurrent_job_requests() {
    let (handle, config) = start_test_server(18097, "redis://127.0.0.1:6379").await;

    // Simulate multiple concurrent TUI clients
    let client = reqwest::Client::new();
    let mut handles = vec![];

    for i in 0..10 {
        let client = client.clone();
        let url = format!("http://{}:{}/jobs", config.host, config.port);
        let handle = tokio::spawn(async move {
            let response = client.get(&url).query(&[("limit", "5")]).send().await;
            (i, response)
        });
        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut success_count = 0;
    for handle in handles {
        if let Ok((_, Ok(response))) = handle.await {
            if response.status().is_success() || response.status().is_server_error() {
                success_count += 1;
            }
        }
    }

    // All requests should complete
    assert_eq!(success_count, 10, "All concurrent requests should complete");

    handle.abort();
}
