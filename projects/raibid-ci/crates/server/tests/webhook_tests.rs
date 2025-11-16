//! Integration tests for webhook routes
//!
//! Note: Tests that require Redis will be skipped if Redis is not available.
//! To run all tests, ensure Redis is running on localhost:6379.

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use hmac::{Hmac, Mac};
use raibid_server::AppState;
use sha2::Sha256;
use tower::ServiceExt;

type HmacSha256 = Hmac<Sha256>;

/// Helper to create a test app state with Redis
fn create_test_state(gitea_secret: Option<String>, github_secret: Option<String>) -> AppState {
    // Use Redis URL from environment or default
    let redis_url =
        std::env::var("RAIBID_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    AppState::with_config(&redis_url, gitea_secret, github_secret)
        .unwrap_or_else(|_| AppState::new())
}

/// Check if Redis is available
async fn is_redis_available() -> bool {
    let redis_url =
        std::env::var("RAIBID_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    match redis::Client::open(redis_url.as_str()) {
        Ok(client) => (client.get_multiplexed_async_connection().await).is_ok(),
        Err(_) => false,
    }
}

/// Generate Gitea HMAC signature
fn generate_gitea_signature(payload: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Generate GitHub HMAC signature
fn generate_github_signature(payload: &str, secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    format!("sha256={}", hex::encode(mac.finalize().into_bytes()))
}

#[tokio::test]
async fn test_gitea_webhook_without_signature() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let state = create_test_state(None, None);
    let app = raibid_server::routes::webhooks::routes().with_state(std::sync::Arc::new(state));

    let payload = r#"{
        "ref": "refs/heads/main",
        "before": "abc123",
        "after": "def456",
        "repository": {
            "id": 1,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "owner": {
                "id": 1,
                "login": "owner",
                "avatar_url": "https://example.com/avatar.png"
            },
            "html_url": "https://git.example.com/owner/test-repo",
            "clone_url": "https://git.example.com/owner/test-repo.git",
            "ssh_url": "git@git.example.com:owner/test-repo.git",
            "default_branch": "main"
        },
        "pusher": {
            "id": 1,
            "username": "testuser",
            "email": "test@example.com",
            "avatar_url": "https://example.com/avatar.png"
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/gitea")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should accept without signature when no secret is configured
    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_gitea_webhook_with_valid_signature() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let secret = "test-secret";
    let state = create_test_state(Some(secret.to_string()), None);
    let app = raibid_server::routes::webhooks::routes().with_state(std::sync::Arc::new(state));

    let payload = r#"{
        "ref": "refs/heads/main",
        "before": "abc123",
        "after": "def456",
        "repository": {
            "id": 1,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "owner": {
                "id": 1,
                "login": "owner",
                "avatar_url": "https://example.com/avatar.png"
            },
            "html_url": "https://git.example.com/owner/test-repo",
            "clone_url": "https://git.example.com/owner/test-repo.git",
            "ssh_url": "git@git.example.com:owner/test-repo.git",
            "default_branch": "main"
        },
        "pusher": {
            "id": 1,
            "username": "testuser",
            "email": "test@example.com",
            "avatar_url": "https://example.com/avatar.png"
        }
    }"#;

    let signature = generate_gitea_signature(payload, secret);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/gitea")
                .header("content-type", "application/json")
                .header("X-Gitea-Signature", signature)
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_gitea_webhook_with_invalid_signature() {
    let secret = "test-secret";
    let state = create_test_state(Some(secret.to_string()), None);
    let app = raibid_server::routes::webhooks::routes().with_state(std::sync::Arc::new(state));

    let payload = r#"{
        "ref": "refs/heads/main",
        "repository": {
            "id": 1,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "owner": {"id": 1, "login": "owner"},
            "html_url": "https://git.example.com/owner/test-repo",
            "clone_url": "https://git.example.com/owner/test-repo.git",
            "ssh_url": "git@git.example.com:owner/test-repo.git",
            "default_branch": "main"
        },
        "pusher": {"id": 1, "username": "testuser"}
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/gitea")
                .header("content-type", "application/json")
                .header("X-Gitea-Signature", "invalid-signature")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_gitea_webhook_missing_signature_when_required() {
    let secret = "test-secret";
    let state = create_test_state(Some(secret.to_string()), None);
    let app = raibid_server::routes::webhooks::routes().with_state(std::sync::Arc::new(state));

    let payload = r#"{
        "ref": "refs/heads/main",
        "repository": {
            "id": 1,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "owner": {"id": 1, "login": "owner"},
            "html_url": "https://git.example.com/owner/test-repo",
            "clone_url": "https://git.example.com/owner/test-repo.git",
            "ssh_url": "git@git.example.com:owner/test-repo.git",
            "default_branch": "main"
        },
        "pusher": {"id": 1, "username": "testuser"}
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/gitea")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_github_webhook_without_signature() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let state = create_test_state(None, None);
    let app = raibid_server::routes::webhooks::routes().with_state(std::sync::Arc::new(state));

    let payload = r#"{
        "ref": "refs/heads/main",
        "before": "abc123",
        "after": "def456",
        "repository": {
            "id": 1,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "owner": {
                "id": 1,
                "login": "owner",
                "avatar_url": "https://example.com/avatar.png"
            },
            "html_url": "https://github.com/owner/test-repo",
            "clone_url": "https://github.com/owner/test-repo.git",
            "ssh_url": "git@github.com:owner/test-repo.git",
            "default_branch": "main"
        },
        "pusher": {
            "name": "testuser",
            "email": "test@example.com"
        }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_github_webhook_with_valid_signature() {
    if !is_redis_available().await {
        eprintln!("Skipping test: Redis not available");
        return;
    }

    let secret = "test-github-secret";
    let state = create_test_state(None, Some(secret.to_string()));
    let app = raibid_server::routes::webhooks::routes().with_state(std::sync::Arc::new(state));

    let payload = r#"{
        "ref": "refs/heads/main",
        "before": "abc123",
        "after": "def456",
        "repository": {
            "id": 1,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "owner": {
                "id": 1,
                "login": "owner",
                "avatar_url": "https://example.com/avatar.png"
            },
            "html_url": "https://github.com/owner/test-repo",
            "clone_url": "https://github.com/owner/test-repo.git",
            "ssh_url": "git@github.com:owner/test-repo.git",
            "default_branch": "main"
        },
        "pusher": {
            "name": "testuser",
            "email": "test@example.com"
        }
    }"#;

    let signature = generate_github_signature(payload, secret);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header("content-type", "application/json")
                .header("X-Hub-Signature-256", signature)
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

#[tokio::test]
async fn test_github_webhook_with_invalid_signature() {
    let secret = "test-github-secret";
    let state = create_test_state(None, Some(secret.to_string()));
    let app = raibid_server::routes::webhooks::routes().with_state(std::sync::Arc::new(state));

    let payload = r#"{
        "ref": "refs/heads/main",
        "repository": {
            "id": 1,
            "name": "test-repo",
            "full_name": "owner/test-repo",
            "owner": {"id": 1, "login": "owner"},
            "html_url": "https://github.com/owner/test-repo",
            "clone_url": "https://github.com/owner/test-repo.git",
            "ssh_url": "git@github.com:owner/test-repo.git",
            "default_branch": "main"
        },
        "pusher": {"name": "testuser"}
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/github")
                .header("content-type", "application/json")
                .header("X-Hub-Signature-256", "sha256=invalid")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_invalid_json_payload() {
    let state = create_test_state(None, None);
    let app = raibid_server::routes::webhooks::routes().with_state(std::sync::Arc::new(state));

    let payload = r#"{"invalid": "json"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/webhooks/gitea")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
