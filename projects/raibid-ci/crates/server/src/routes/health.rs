//! Health check routes

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::state::AppState;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_seconds: i64,
    pub requests_total: u64,
    pub active_connections: u64,
    pub timestamp: String,
}

/// Detailed health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedHealthResponse {
    #[serde(flatten)]
    pub health: HealthResponse,
    pub checks: HealthChecks,
}

/// Health check details
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthChecks {
    pub database: CheckStatus,
    pub redis: CheckStatus,
    pub kubernetes: CheckStatus,
}

/// Individual check status
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckStatus {
    pub healthy: bool,
    pub message: String,
}

/// Create health check routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/health", get(health_handler))
        .route("/health/ready", get(readiness_handler))
        .route("/health/live", get(liveness_handler))
}

async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let now = chrono::Utc::now();
    let uptime = (now - state.start_time()).num_seconds();

    let response = HealthResponse {
        status: "ok".to_string(),
        uptime_seconds: uptime,
        requests_total: state.request_count(),
        active_connections: state.active_connections(),
        timestamp: now.to_rfc3339(),
    };

    (StatusCode::OK, Json(response))
}

async fn readiness_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let now = chrono::Utc::now();
    let uptime = (now - state.start_time()).num_seconds();
    let health_status = state.health_status().await;

    let response = DetailedHealthResponse {
        health: HealthResponse {
            status: if health_status.healthy {
                "ready".to_string()
            } else {
                "not_ready".to_string()
            },
            uptime_seconds: uptime,
            requests_total: state.request_count(),
            active_connections: state.active_connections(),
            timestamp: now.to_rfc3339(),
        },
        checks: HealthChecks {
            database: CheckStatus {
                healthy: true,
                message: "Not implemented yet".to_string(),
            },
            redis: CheckStatus {
                healthy: true,
                message: "Not implemented yet".to_string(),
            },
            kubernetes: CheckStatus {
                healthy: true,
                message: "Not implemented yet".to_string(),
            },
        },
    };

    let status = if health_status.healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, Json(response))
}

async fn liveness_handler() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "alive",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health_endpoint() {
        let state = Arc::new(AppState::new());
        let app = routes().with_state(state.clone());

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: HealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health.status, "ok");
    }

    #[tokio::test]
    async fn test_liveness_endpoint() {
        let state = Arc::new(AppState::new());
        let app = routes().with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health/live")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readiness_endpoint() {
        let state = Arc::new(AppState::new());
        let app = routes().with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let health: DetailedHealthResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(health.health.status, "ready");
    }
}
