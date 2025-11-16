//! Job status and management routes

use axum::{
    extract::{Path, Query, State},
    response::{
        sse::{Event, KeepAlive},
        Sse,
    },
    routing::get,
    Json, Router,
};
use futures::stream::{self, Stream};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use crate::{error::ServerError, state::AppState};
use raibid_common::{Job, JobStatus};

/// Query parameters for job list endpoint
#[derive(Debug, Deserialize)]
pub struct JobsQueryParams {
    /// Filter by status
    pub status: Option<String>,
    /// Filter by repository
    pub repo: Option<String>,
    /// Filter by branch
    pub branch: Option<String>,
    /// Pagination limit
    pub limit: Option<usize>,
    /// Pagination offset
    pub offset: Option<usize>,
    /// Cursor for cursor-based pagination
    pub cursor: Option<String>,
}

/// Response with pagination cursor
#[derive(Debug, Serialize)]
pub struct JobListResponse {
    /// Jobs list
    pub jobs: Vec<Job>,
    /// Total count
    pub total: usize,
    /// Current offset
    pub offset: usize,
    /// Current limit
    pub limit: usize,
    /// Next cursor (for cursor-based pagination)
    pub next_cursor: Option<String>,
}

/// Create job routes
pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/jobs", get(list_jobs))
        .route("/jobs/{id}", get(get_job))
        .route("/jobs/{id}/logs", get(get_job_logs))
}

/// GET /jobs - List all jobs with filtering and pagination
async fn list_jobs(
    State(state): State<Arc<AppState>>,
    Query(params): Query<JobsQueryParams>,
) -> Result<Json<JobListResponse>, ServerError> {
    // Parse status filter if provided
    let status_filter = if let Some(status_str) = &params.status {
        Some(
            status_str
                .parse::<JobStatus>()
                .map_err(|_| ServerError::BadRequest(format!("Invalid status: {}", status_str)))?,
        )
    } else {
        None
    };

    // Get Redis connection
    let mut conn = state.redis_connection().await?;

    // For MVP, we'll store jobs as Redis hashes with keys like "job:{id}"
    // In production, you'd want a more sophisticated data structure

    // Get all job IDs using SCAN for cursor-based pagination
    let pattern = "job:*";
    let cursor = params.cursor.clone().unwrap_or_else(|| "0".to_string());
    let scan_limit = params.limit.unwrap_or(20).min(100); // Max 100 per request

    // SCAN returns (next_cursor, keys)
    let (next_cursor, job_keys): (String, Vec<String>) = redis::cmd("SCAN")
        .arg(&cursor)
        .arg("MATCH")
        .arg(pattern)
        .arg("COUNT")
        .arg(scan_limit)
        .query_async(&mut conn)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to scan jobs: {}", e)))?;

    // Fetch job data for each key
    let mut jobs = Vec::new();
    for key in &job_keys {
        // Get job hash data
        let job_data: std::collections::HashMap<String, String> = conn
            .hgetall(key)
            .await
            .map_err(|e| ServerError::Internal(format!("Failed to get job: {}", e)))?;

        if job_data.is_empty() {
            continue;
        }

        // Parse job from hash
        if let Ok(job) = parse_job_from_hash(&job_data) {
            // Apply filters
            if let Some(status) = status_filter {
                if job.status != status {
                    continue;
                }
            }
            if let Some(ref repo) = params.repo {
                if &job.repo != repo {
                    continue;
                }
            }
            if let Some(ref branch) = params.branch {
                if &job.branch != branch {
                    continue;
                }
            }
            jobs.push(job);
        }
    }

    // Sort by started_at descending (newest first)
    jobs.sort_by(|a, b| b.started_at.cmp(&a.started_at));

    // Apply offset-based pagination if cursor not used
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(20);
    let total = jobs.len();

    let paginated_jobs = if params.cursor.is_none() {
        jobs.into_iter().skip(offset).take(limit).collect()
    } else {
        jobs.into_iter().take(limit).collect()
    };

    let response = JobListResponse {
        jobs: paginated_jobs,
        total,
        offset,
        limit,
        next_cursor: if next_cursor == "0" {
            None
        } else {
            Some(next_cursor)
        },
    };

    Ok(Json(response))
}

/// GET /jobs/{id} - Get a specific job by ID
async fn get_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Job>, ServerError> {
    let mut conn = state.redis_connection().await?;

    // Get job hash
    let key = format!("job:{}", id);
    let job_data: std::collections::HashMap<String, String> = conn
        .hgetall(&key)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to get job: {}", e)))?;

    if job_data.is_empty() {
        return Err(ServerError::NotFound(format!("Job not found: {}", id)));
    }

    let job = parse_job_from_hash(&job_data)?;
    Ok(Json(job))
}

/// GET /jobs/{id}/logs - Stream job logs via Server-Sent Events
async fn get_job_logs(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, ServerError> {
    // First verify the job exists
    let mut conn = state.redis_connection().await?;
    let key = format!("job:{}", id);
    let exists: bool = conn
        .exists(&key)
        .await
        .map_err(|e| ServerError::Internal(format!("Failed to check job: {}", e)))?;

    if !exists {
        return Err(ServerError::NotFound(format!("Job not found: {}", id)));
    }

    // Create stream for SSE
    // In production, this would use Redis Streams (XREAD) to stream logs in real-time
    // For MVP, we'll demonstrate the SSE pattern with periodic polling
    let stream = stream::unfold(
        (state.clone(), id.clone(), 0u64),
        |(state, job_id, last_seq)| async move {
            // Get logs from Redis stream
            let log_stream = format!("job:{}:logs", job_id);

            match state.redis_connection().await {
                Ok(mut conn) => {
                    // Use XREAD to get new log entries
                    // Format: XREAD COUNT count STREAMS stream_key last_id
                    let start_id = if last_seq == 0 {
                        "0".to_string()
                    } else {
                        last_seq.to_string()
                    };

                    // Try to read log entries
                    match read_log_entries(&mut conn, &log_stream, &start_id).await {
                        Ok((entries, new_seq)) => {
                            if !entries.is_empty() {
                                // Create SSE event with log data
                                let data = serde_json::to_string(&entries).unwrap_or_default();
                                let event = Event::default().data(data);
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                Some((Ok(event), (state, job_id, new_seq)))
                            } else {
                                // No new entries, wait and retry
                                tokio::time::sleep(Duration::from_millis(500)).await;
                                Some((
                                    Ok(Event::default().comment("keepalive")),
                                    (state, job_id, last_seq),
                                ))
                            }
                        }
                        Err(_) => {
                            // Error reading, send keepalive
                            tokio::time::sleep(Duration::from_millis(500)).await;
                            Some((
                                Ok(Event::default().comment("keepalive")),
                                (state, job_id, last_seq),
                            ))
                        }
                    }
                }
                Err(_) => None, // Connection error, end stream
            }
        },
    );

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Parse job from Redis hash data
fn parse_job_from_hash(
    data: &std::collections::HashMap<String, String>,
) -> Result<Job, ServerError> {
    let id = data
        .get("id")
        .ok_or_else(|| ServerError::Internal("Missing job id".to_string()))?
        .clone();
    let repo = data
        .get("repo")
        .ok_or_else(|| ServerError::Internal("Missing repo".to_string()))?
        .clone();
    let branch = data
        .get("branch")
        .ok_or_else(|| ServerError::Internal("Missing branch".to_string()))?
        .clone();
    let commit = data
        .get("commit")
        .ok_or_else(|| ServerError::Internal("Missing commit".to_string()))?
        .clone();

    let status = data
        .get("status")
        .ok_or_else(|| ServerError::Internal("Missing status".to_string()))?
        .parse::<JobStatus>()
        .map_err(|e| ServerError::Internal(format!("Invalid status: {}", e)))?;

    let started_at = data
        .get("started_at")
        .ok_or_else(|| ServerError::Internal("Missing started_at".to_string()))?
        .parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| ServerError::Internal(format!("Invalid started_at: {}", e)))?;

    let finished_at = data
        .get("finished_at")
        .and_then(|s| s.parse::<chrono::DateTime<chrono::Utc>>().ok());

    let duration = data.get("duration").and_then(|s| s.parse::<u64>().ok());
    let agent_id = data.get("agent_id").cloned();
    let exit_code = data.get("exit_code").and_then(|s| s.parse::<i32>().ok());

    Ok(Job {
        id,
        repo,
        branch,
        commit,
        status,
        started_at,
        finished_at,
        duration,
        agent_id,
        exit_code,
    })
}

/// Read log entries from Redis stream
async fn read_log_entries(
    conn: &mut redis::aio::MultiplexedConnection,
    stream_key: &str,
    start_id: &str,
) -> Result<(Vec<serde_json::Value>, u64), redis::RedisError> {
    // XREAD COUNT 100 STREAMS stream_key start_id
    let result: Vec<Vec<Vec<(String, Vec<(String, Vec<(String, String)>)>)>>> = redis::cmd("XREAD")
        .arg("COUNT")
        .arg(100)
        .arg("STREAMS")
        .arg(stream_key)
        .arg(start_id)
        .query_async(conn)
        .await?;

    let mut entries = Vec::new();
    let mut last_seq = start_id.parse::<u64>().unwrap_or(0);

    // Parse XREAD response
    for stream_data in result {
        for stream_info in stream_data {
            for (_, messages) in stream_info {
                for (msg_id, fields) in messages {
                    // Parse message ID to get sequence
                    if let Some((seq_str, _)) = msg_id.split_once('-') {
                        if let Ok(seq) = seq_str.parse::<u64>() {
                            last_seq = last_seq.max(seq);
                        }
                    }

                    // Convert fields to JSON
                    let mut log_entry = serde_json::Map::new();
                    log_entry.insert("id".to_string(), serde_json::Value::String(msg_id.clone()));

                    for (key, value) in fields {
                        log_entry.insert(key.clone(), serde_json::Value::String(value.clone()));
                    }

                    entries.push(serde_json::Value::Object(log_entry));
                }
            }
        }
    }

    Ok((entries, last_seq))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_list_jobs_without_redis() {
        let state = Arc::new(AppState::new());
        let app = routes().with_state(state);

        let response = app
            .oneshot(Request::builder().uri("/jobs").body(Body::empty()).unwrap())
            .await
            .unwrap();

        // Should return error since Redis is not configured
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_get_job_without_redis() {
        let state = Arc::new(AppState::new());
        let app = routes().with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/jobs/test-job-123")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should return error since Redis is not configured
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_parse_job_from_hash() {
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), "job-123".to_string());
        data.insert("repo".to_string(), "raibid-ci".to_string());
        data.insert("branch".to_string(), "main".to_string());
        data.insert("commit".to_string(), "abc123".to_string());
        data.insert("status".to_string(), "running".to_string());
        data.insert("started_at".to_string(), "2025-11-01T12:00:00Z".to_string());

        let job = parse_job_from_hash(&data).unwrap();
        assert_eq!(job.id, "job-123");
        assert_eq!(job.repo, "raibid-ci");
        assert_eq!(job.status, JobStatus::Running);
    }

    #[test]
    fn test_parse_job_from_hash_missing_field() {
        let mut data = std::collections::HashMap::new();
        data.insert("id".to_string(), "job-123".to_string());
        // Missing required fields

        let result = parse_job_from_hash(&data);
        assert!(result.is_err());
    }
}
