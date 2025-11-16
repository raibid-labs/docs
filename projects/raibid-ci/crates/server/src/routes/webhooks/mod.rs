//! Webhook route handlers for GitHub and Gitea

mod payloads;
mod signature;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::{error::ServerError, state::AppState};
pub use payloads::{GitHubWebhookPayload, GiteaWebhookPayload};
use signature::{verify_gitea_signature, verify_github_signature};

/// Job metadata for Redis Stream
#[derive(Debug, Serialize, Deserialize)]
pub struct JobMetadata {
    pub job_id: String,
    pub repository: String,
    pub branch: String,
    pub commit: String,
    pub author: String,
    pub event_type: String,
    pub created_at: String,
}

/// Webhook response
#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookResponse {
    pub job_id: String,
    pub message: String,
}

/// Create webhook routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/webhooks/gitea", post(gitea_webhook_handler))
        .route("/webhooks/github", post(github_webhook_handler))
}

/// Gitea webhook handler
async fn gitea_webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, ServerError> {
    info!("Received Gitea webhook request");

    // Verify signature if secret is configured
    if let Some(secret) = state.gitea_webhook_secret() {
        let signature = headers
            .get("X-Gitea-Signature")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                ServerError::Unauthorized("Missing X-Gitea-Signature header".to_string())
            })?;

        if !verify_gitea_signature(&body, signature, secret) {
            warn!("Invalid Gitea webhook signature");
            return Err(ServerError::Unauthorized("Invalid signature".to_string()));
        }
    }

    // Parse webhook payload
    let payload: GiteaWebhookPayload = serde_json::from_str(&body)
        .map_err(|e| ServerError::BadRequest(format!("Invalid webhook payload: {}", e)))?;

    // Extract metadata
    let metadata = JobMetadata {
        job_id: Uuid::new_v4().to_string(),
        repository: payload.repository.full_name,
        branch: payload.ref_name.unwrap_or_else(|| "main".to_string()),
        commit: payload.after.unwrap_or_default(),
        author: payload.pusher.username,
        event_type: "push".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    // Queue job to Redis Streams
    let job_id = queue_job(&state, &metadata).await?;

    info!(
        "Queued job {} for repository {}",
        job_id, metadata.repository
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(WebhookResponse {
            job_id: job_id.clone(),
            message: format!("Job {} queued successfully", job_id),
        }),
    ))
}

/// GitHub webhook handler
async fn github_webhook_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, ServerError> {
    info!("Received GitHub webhook request");

    // Verify signature if secret is configured
    if let Some(secret) = state.github_webhook_secret() {
        let signature = headers
            .get("X-Hub-Signature-256")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                ServerError::Unauthorized("Missing X-Hub-Signature-256 header".to_string())
            })?;

        if !verify_github_signature(&body, signature, secret) {
            warn!("Invalid GitHub webhook signature");
            return Err(ServerError::Unauthorized("Invalid signature".to_string()));
        }
    }

    // Parse webhook payload
    let payload: GitHubWebhookPayload = serde_json::from_str(&body)
        .map_err(|e| ServerError::BadRequest(format!("Invalid webhook payload: {}", e)))?;

    // Extract metadata
    let metadata = JobMetadata {
        job_id: Uuid::new_v4().to_string(),
        repository: payload.repository.full_name,
        branch: payload.ref_name.unwrap_or_else(|| "main".to_string()),
        commit: payload.after.unwrap_or_default(),
        author: payload.pusher.name,
        event_type: "push".to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    // Queue job to Redis Streams
    let job_id = queue_job(&state, &metadata).await?;

    info!(
        "Queued job {} for repository {}",
        job_id, metadata.repository
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(WebhookResponse {
            job_id: job_id.clone(),
            message: format!("Job {} queued successfully", job_id),
        }),
    ))
}

/// Queue a job to Redis Streams
async fn queue_job(state: &AppState, metadata: &JobMetadata) -> Result<String, ServerError> {
    let mut conn = state.redis_connection().await?;

    // Serialize metadata to JSON
    let metadata_json = serde_json::to_string(metadata)?;

    // Add job to Redis Stream using XADD
    let _stream_id: String = conn
        .xadd("ci:jobs", "*", &[("data", metadata_json.as_str())])
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to queue job: {}", e)))?;

    Ok(metadata.job_id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_metadata_serialization() {
        let metadata = JobMetadata {
            job_id: "test-123".to_string(),
            repository: "owner/repo".to_string(),
            branch: "main".to_string(),
            commit: "abc123".to_string(),
            author: "testuser".to_string(),
            event_type: "push".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("test-123"));
        assert!(json.contains("owner/repo"));
    }

    #[test]
    fn test_webhook_response_serialization() {
        let response = WebhookResponse {
            job_id: "job-123".to_string(),
            message: "Job queued".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("job-123"));
        assert!(json.contains("Job queued"));
    }
}
