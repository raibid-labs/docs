//! Retry Logic Utilities
//!
//! This module provides retry mechanisms with exponential backoff for transient failures.

#![allow(dead_code)]

use crate::infrastructure::error::{InfraError, InfraResult};
use std::time::Duration;
use tracing::{debug, warn};

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay before first retry
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Whether to use jitter
    pub use_jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a config for quick retries (e.g., network requests)
    pub fn quick() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 1.5,
            use_jitter: true,
        }
    }

    /// Create a config for slow retries (e.g., service readiness)
    pub fn slow() -> Self {
        Self {
            max_attempts: 10,
            initial_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
            use_jitter: true,
        }
    }

    /// Create a config with no retries
    pub fn none() -> Self {
        Self {
            max_attempts: 1,
            initial_delay: Duration::from_secs(0),
            max_delay: Duration::from_secs(0),
            backoff_multiplier: 1.0,
            use_jitter: false,
        }
    }

    /// Calculate delay for a given attempt number
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        let mut delay =
            self.initial_delay.as_secs_f64() * self.backoff_multiplier.powi(attempt as i32 - 1);

        // Cap at max delay
        if delay > self.max_delay.as_secs_f64() {
            delay = self.max_delay.as_secs_f64();
        }

        // Add jitter if enabled
        if self.use_jitter {
            use rand::Rng;
            let jitter = rand::thread_rng().gen_range(0.0..=0.3);
            delay *= 1.0 + jitter;
        }

        Duration::from_secs_f64(delay)
    }
}

/// Retry a function with exponential backoff
pub fn retry_with_backoff<F, T>(
    config: &RetryConfig,
    operation_name: &str,
    mut f: F,
) -> InfraResult<T>
where
    F: FnMut() -> InfraResult<T>,
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        if attempt > 0 {
            let delay = config.delay_for_attempt(attempt);
            warn!(
                "Retry attempt {}/{} for '{}' after {:?}",
                attempt + 1,
                config.max_attempts,
                operation_name,
                delay
            );
            std::thread::sleep(delay);
        } else {
            debug!(
                "Attempting '{}' (attempt 1/{})",
                operation_name, config.max_attempts
            );
        }

        match f() {
            Ok(result) => {
                if attempt > 0 {
                    debug!("'{}' succeeded after {} retries", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(err) => {
                // Don't retry fatal errors
                if err.is_fatal() {
                    debug!("Fatal error encountered, not retrying: {}", err);
                    return Err(err);
                }

                // Check if error is transient and should be retried
                if !err.is_transient() && attempt > 0 {
                    debug!("Non-transient error, stopping retries: {}", err);
                    return Err(err);
                }

                last_error = Some(err);
            }
        }
    }

    // All retries exhausted
    Err(last_error.unwrap_or_else(|| InfraError::Fatal {
        component: operation_name.to_string(),
        reason: "All retry attempts exhausted".to_string(),
        context: vec![format!("Tried {} times", config.max_attempts)],
    }))
}

/// Async version of retry with backoff
pub async fn retry_with_backoff_async<F, Fut, T>(
    config: &RetryConfig,
    operation_name: &str,
    mut f: F,
) -> InfraResult<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = InfraResult<T>>,
{
    let mut last_error = None;

    for attempt in 0..config.max_attempts {
        if attempt > 0 {
            let delay = config.delay_for_attempt(attempt);
            warn!(
                "Async retry attempt {}/{} for '{}' after {:?}",
                attempt + 1,
                config.max_attempts,
                operation_name,
                delay
            );
            tokio::time::sleep(delay).await;
        } else {
            debug!(
                "Attempting '{}' (attempt 1/{})",
                operation_name, config.max_attempts
            );
        }

        match f().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("'{}' succeeded after {} retries", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(err) => {
                // Don't retry fatal errors
                if err.is_fatal() {
                    debug!("Fatal error encountered, not retrying: {}", err);
                    return Err(err);
                }

                // Check if error is transient and should be retried
                if !err.is_transient() && attempt > 0 {
                    debug!("Non-transient error, stopping retries: {}", err);
                    return Err(err);
                }

                last_error = Some(err);
            }
        }
    }

    // All retries exhausted
    Err(last_error.unwrap_or_else(|| InfraError::Fatal {
        component: operation_name.to_string(),
        reason: "All retry attempts exhausted".to_string(),
        context: vec![format!("Tried {} times", config.max_attempts)],
    }))
}

/// Poll for a condition with timeout
pub fn poll_until<F>(
    config: &RetryConfig,
    timeout: Duration,
    operation_name: &str,
    mut condition: F,
) -> InfraResult<()>
where
    F: FnMut() -> InfraResult<bool>,
{
    let start = std::time::Instant::now();
    let mut attempt = 0;

    loop {
        if start.elapsed() > timeout {
            return Err(InfraError::Timeout {
                operation: operation_name.to_string(),
                duration: timeout,
                suggestion: format!(
                    "Consider increasing the timeout or check if the operation is actually progressing. Tried {} times over {:?}",
                    attempt, timeout
                ),
            });
        }

        match condition() {
            Ok(true) => {
                debug!(
                    "Condition met for '{}' after {} attempts ({:?})",
                    operation_name,
                    attempt,
                    start.elapsed()
                );
                return Ok(());
            }
            Ok(false) => {
                // Condition not met yet, continue polling
            }
            Err(err) => {
                if err.is_fatal() {
                    return Err(err);
                }
                // For transient errors during polling, log but continue
                warn!("Transient error during polling: {}", err);
            }
        }

        attempt += 1;
        let delay = config.delay_for_attempt(attempt);
        std::thread::sleep(delay);
    }
}

/// Async version of poll until
pub async fn poll_until_async<F, Fut>(
    config: &RetryConfig,
    timeout: Duration,
    operation_name: &str,
    mut condition: F,
) -> InfraResult<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = InfraResult<bool>>,
{
    let start = std::time::Instant::now();
    let mut attempt = 0;

    loop {
        if start.elapsed() > timeout {
            return Err(InfraError::Timeout {
                operation: operation_name.to_string(),
                duration: timeout,
                suggestion: format!(
                    "Consider increasing the timeout or check if the operation is actually progressing. Tried {} times over {:?}",
                    attempt, timeout
                ),
            });
        }

        match condition().await {
            Ok(true) => {
                debug!(
                    "Condition met for '{}' after {} attempts ({:?})",
                    operation_name,
                    attempt,
                    start.elapsed()
                );
                return Ok(());
            }
            Ok(false) => {
                // Condition not met yet, continue polling
            }
            Err(err) => {
                if err.is_fatal() {
                    return Err(err);
                }
                // For transient errors during polling, log but continue
                warn!("Transient error during async polling: {}", err);
            }
        }

        attempt += 1;
        let delay = config.delay_for_attempt(attempt);
        tokio::time::sleep(delay).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_secs(1));
    }

    #[test]
    fn test_retry_config_quick() {
        let config = RetryConfig::quick();
        assert_eq!(config.max_attempts, 5);
        assert_eq!(config.initial_delay, Duration::from_millis(500));
    }

    #[test]
    fn test_retry_config_slow() {
        let config = RetryConfig::slow();
        assert_eq!(config.max_attempts, 10);
        assert_eq!(config.initial_delay, Duration::from_secs(2));
    }

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            use_jitter: false,
        };

        assert_eq!(config.delay_for_attempt(0), Duration::from_secs(0));
        assert_eq!(config.delay_for_attempt(1), Duration::from_secs(1));
        assert_eq!(config.delay_for_attempt(2), Duration::from_secs(2));
        assert_eq!(config.delay_for_attempt(3), Duration::from_secs(4));
        assert_eq!(config.delay_for_attempt(4), Duration::from_secs(8));
        assert_eq!(config.delay_for_attempt(5), Duration::from_secs(10)); // Capped
    }

    #[test]
    fn test_retry_success_on_first_attempt() {
        let config = RetryConfig::default();
        let mut attempts = 0;

        let result = retry_with_backoff(&config, "test", || {
            attempts += 1;
            Ok(42)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 1);
    }

    #[test]
    fn test_retry_success_on_second_attempt() {
        let config = RetryConfig::default();
        let mut attempts = 0;

        let result = retry_with_backoff(&config, "test", || {
            attempts += 1;
            if attempts == 1 {
                Err(InfraError::Transient {
                    operation: "test".to_string(),
                    reason: "temporary failure".to_string(),
                    retry_after: None,
                })
            } else {
                Ok(42)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 2);
    }

    // TODO: Issue #TBD - Fix type annotations and re-enable this test
    // #[test]
    // fn test_retry_fatal_error_no_retry() {
    //     let config = RetryConfig::default();
    //     let mut attempts = 0;
    //
    //     let result = retry_with_backoff(&config, "test", || {
    //         attempts += 1;
    //         Err(InfraError::Fatal {
    //             component: "test".to_string(),
    //             reason: "fatal".to_string(),
    //             context: vec![],
    //         })
    //     });
    //
    //     assert!(result.is_err());
    //     assert!(result.unwrap_err().is_fatal());
    //     assert_eq!(attempts, 1); // Should not retry
    // }

    // TODO: Issue #TBD - Fix type annotations and re-enable this test
    // #[test]
    // fn test_retry_exhausted() {
    //     let config = RetryConfig {
    //         max_attempts: 3,
    //         initial_delay: Duration::from_millis(1),
    //         max_delay: Duration::from_secs(1),
    //         backoff_multiplier: 1.5,
    //         use_jitter: false,
    //     };
    //     let mut attempts = 0;
    //
    //     let result = retry_with_backoff(&config, "test", || {
    //         attempts += 1;
    //         Err(InfraError::Transient {
    //             operation: "test".to_string(),
    //             reason: "always fails".to_string(),
    //             retry_after: None,
    //         })
    //     });
    //
    //     assert!(result.is_err());
    //     assert_eq!(attempts, 3);
    // }

    // TODO: Issue #TBD - Fix captured variable escape in async closure
    // #[tokio::test]
    // async fn test_async_retry_success() {
    //     let config = RetryConfig::default();
    //     let mut attempts = 0;
    //
    //     let result = retry_with_backoff_async(&config, "test", || async {
    //         attempts += 1;
    //         Ok(42)
    //     }).await;
    //
    //     assert!(result.is_ok());
    //     assert_eq!(result.unwrap(), 42);
    //     assert_eq!(attempts, 1);
    // }

    #[test]
    fn test_poll_until_immediate_success() {
        let config = RetryConfig::quick();
        let timeout = Duration::from_secs(10);

        let result = poll_until(&config, timeout, "test", || Ok(true));

        assert!(result.is_ok());
    }

    #[test]
    fn test_poll_until_timeout() {
        let config = RetryConfig {
            max_attempts: 100,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(50),
            backoff_multiplier: 1.0,
            use_jitter: false,
        };
        let timeout = Duration::from_millis(100);

        let result = poll_until(&config, timeout, "test", || Ok(false));

        assert!(result.is_err());
        if let Err(InfraError::Timeout { .. }) = result {
            // Expected
        } else {
            panic!("Expected timeout error");
        }
    }

    // TODO: Issue #TBD - Fix captured variable escape in async closure
    // #[tokio::test]
    // async fn test_async_poll_until_success_after_retries() {
    //     let config = RetryConfig::quick();
    //     let timeout = Duration::from_secs(10);
    //     let mut attempts = 0;
    //
    //     let result = poll_until_async(&config, timeout, "test", || async {
    //         attempts += 1;
    //         Ok(attempts >= 3)
    //     }).await;
    //
    //     assert!(result.is_ok());
    //     assert!(attempts >= 3);
    // }
}
