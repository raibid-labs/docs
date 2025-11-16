//! Integration tests for raibid-server

use raibid_server::{Server, ServerConfig};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_server_starts_and_responds() {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 18080,
        log_format: "text".to_string(),
        cors_enabled: false,
        max_body_size: 1024 * 1024,
        redis_url: "redis://127.0.0.1:6379".to_string(),
        gitea_webhook_secret: None,
        github_webhook_secret: None,
        rate_limit_rpm: 100,
    };

    let server = Server::new(config.clone());

    let server_handle = tokio::spawn(async move { server.run().await });

    sleep(Duration::from_millis(500)).await;

    let response = reqwest::get(format!("http://{}:{}/health", config.host, config.port))
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);

    let body = response.text().await.expect("Failed to read response body");
    assert!(body.contains("ok"));

    server_handle.abort();
}

#[tokio::test]
async fn test_health_endpoints_return_json() {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 18081,
        log_format: "text".to_string(),
        cors_enabled: false,
        max_body_size: 1024 * 1024,
        redis_url: "redis://127.0.0.1:6379".to_string(),
        gitea_webhook_secret: None,
        github_webhook_secret: None,
        rate_limit_rpm: 100,
    };

    let server = Server::new(config.clone());

    let server_handle = tokio::spawn(async move { server.run().await });

    sleep(Duration::from_millis(500)).await;

    let response = reqwest::get(format!("http://{}:{}/health", config.host, config.port))
        .await
        .expect("Failed to make request");

    assert_eq!(response.status(), 200);
    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(json["status"], "ok");
    assert!(json.get("uptime_seconds").is_some());

    let response = reqwest::get(format!(
        "http://{}:{}/health/live",
        config.host, config.port
    ))
    .await
    .expect("Failed to make request");

    assert_eq!(response.status(), 200);
    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(json["status"], "alive");

    let response = reqwest::get(format!(
        "http://{}:{}/health/ready",
        config.host, config.port
    ))
    .await
    .expect("Failed to make request");

    assert_eq!(response.status(), 200);
    let json: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(json["status"], "ready");
    assert!(json.get("checks").is_some());

    server_handle.abort();
}

#[tokio::test]
async fn test_request_id_header() {
    let config = ServerConfig {
        host: "127.0.0.1".to_string(),
        port: 18082,
        log_format: "text".to_string(),
        cors_enabled: false,
        max_body_size: 1024 * 1024,
        redis_url: "redis://127.0.0.1:6379".to_string(),
        gitea_webhook_secret: None,
        github_webhook_secret: None,
        rate_limit_rpm: 100,
    };

    let server = Server::new(config.clone());

    let server_handle = tokio::spawn(async move { server.run().await });

    sleep(Duration::from_millis(500)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("http://{}:{}/health", config.host, config.port))
        .send()
        .await
        .expect("Failed to make request");

    assert!(response.headers().contains_key("x-request-id"));

    server_handle.abort();
}
