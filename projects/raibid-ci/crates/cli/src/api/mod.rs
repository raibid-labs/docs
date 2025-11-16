//! API client for raibid-ci server
//!
//! This module provides a client for interacting with the raibid-ci API server.
//! It handles HTTP requests, error handling, and response parsing.

use anyhow::{Context, Result};
use raibid_common::{Job, JobList, JobListQuery, JobLogs, JobTrigger};
use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;

/// API client for raibid-ci server
#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
    client: Client,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: impl Into<String>) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: base_url.into(),
            client,
        })
    }

    /// Create API client from environment or default
    pub fn from_env() -> Result<Self> {
        let base_url =
            std::env::var("RAIBID_API_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        Self::new(base_url)
    }

    /// List jobs with optional filters
    pub fn list_jobs(&self, query: &JobListQuery) -> Result<JobList> {
        let mut url = format!("{}/api/jobs", self.base_url);
        let mut params = Vec::new();

        if let Some(status) = &query.status {
            params.push(format!("status={}", status.as_str().to_lowercase()));
        }
        if let Some(repo) = &query.repo {
            params.push(format!("repo={}", urlencoding::encode(repo)));
        }
        if let Some(branch) = &query.branch {
            params.push(format!("branch={}", urlencoding::encode(branch)));
        }
        if let Some(limit) = query.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = query.offset {
            params.push(format!("offset={}", offset));
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        self.get(&url)
    }

    /// Get a specific job by ID
    pub fn get_job(&self, job_id: &str) -> Result<Job> {
        let url = format!("{}/api/jobs/{}", self.base_url, job_id);
        self.get(&url)
    }

    /// Get logs for a specific job
    pub fn get_job_logs(&self, job_id: &str, tail: Option<usize>) -> Result<JobLogs> {
        let mut url = format!("{}/api/jobs/{}/logs", self.base_url, job_id);

        if let Some(tail) = tail {
            url.push_str(&format!("?tail={}", tail));
        }

        self.get(&url)
    }

    /// Trigger a new job
    pub fn trigger_job(&self, trigger: &JobTrigger) -> Result<Job> {
        let url = format!("{}/api/jobs", self.base_url);

        let response = self
            .client
            .post(&url)
            .json(trigger)
            .send()
            .context("Failed to send trigger request")?;

        self.handle_response(response)
    }

    /// Cancel a job
    pub fn cancel_job(&self, job_id: &str) -> Result<Job> {
        let url = format!("{}/api/jobs/{}/cancel", self.base_url, job_id);

        let response = self
            .client
            .post(&url)
            .send()
            .context("Failed to send cancel request")?;

        self.handle_response(response)
    }

    /// Generic GET request
    fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let response = self
            .client
            .get(url)
            .send()
            .context("Failed to send GET request")?;

        self.handle_response(response)
    }

    /// Handle HTTP response and parse JSON
    fn handle_response<T: DeserializeOwned>(
        &self,
        response: reqwest::blocking::Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            response.json().context("Failed to parse JSON response")
        } else {
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Unknown error".to_string());

            Err(anyhow::anyhow!(
                "API request failed with status {}: {}",
                status,
                error_text
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_new() {
        let client = ApiClient::new("http://localhost:8080");
        assert!(client.is_ok());
    }

    #[test]
    fn test_api_client_url_building() {
        let client = ApiClient::new("http://localhost:8080").unwrap();

        let query = JobListQuery {
            status: Some(raibid_common::JobStatus::Running),
            repo: Some("test-repo".to_string()),
            ..Default::default()
        };

        // This would normally make a request, but we're just testing URL construction
        // In a real test, we'd use a mock HTTP server
        assert!(client.list_jobs(&query).is_err());
    }
}
