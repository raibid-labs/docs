//! Error Handling Integration Tests
//!
//! This module tests the comprehensive error handling, retry logic,
//! pre-flight validation, rollback mechanisms, and health checks.

use raibid_cli::infrastructure::{
    InfraError, InfraResult, InstallPhase, RetryConfig, retry_with_backoff,
    SystemRequirements, PreFlightValidator, PreFlightResult,
    RollbackManager, RollbackContext, HealthCheckResult, HealthStatus,
};
use std::time::Duration;
use std::sync::{Arc, Mutex};

#[test]
fn test_error_types_download() {
    let err = InfraError::download("k3s", "https://example.com/k3s", "HTTP 404");
    let msg = err.to_string();
    assert!(msg.contains("k3s"));
    assert!(msg.contains("404"));
    assert!(msg.contains("Suggestion"));
}

#[test]
fn test_error_types_installation() {
    let err = InfraError::installation("gitea", InstallPhase::PreFlight, "kubectl not found");
    let msg = err.to_string();
    assert!(msg.contains("gitea"));
    assert!(msg.contains("pre-flight"));
    assert!(msg.contains("kubectl"));
    assert!(msg.contains("prerequisites"));
}

#[test]
fn test_error_types_timeout() {
    let err = InfraError::Timeout {
        operation: "pod readiness".to_string(),
        duration: Duration::from_secs(300),
        suggestion: "Increase timeout".to_string(),
    };
    let msg = err.to_string();
    assert!(msg.contains("timed out"));
    assert!(msg.contains("300"));
}

#[test]
fn test_error_transient_detection() {
    let transient = InfraError::Transient {
        operation: "network".to_string(),
        reason: "temporary failure".to_string(),
        retry_after: Some(Duration::from_secs(5)),
    };
    assert!(transient.is_transient());
    assert_eq!(transient.retry_delay(), Some(Duration::from_secs(5)));

    let network = InfraError::network("download", "timeout");
    assert!(network.is_transient());

    let fatal = InfraError::Fatal {
        component: "test".to_string(),
        reason: "unsupported".to_string(),
        context: vec![],
    };
    assert!(fatal.is_fatal());
    assert!(!fatal.is_transient());
}

#[test]
fn test_retry_config_delay_calculation() {
    let config = RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(16),
        backoff_multiplier: 2.0,
        use_jitter: false,
    };

    assert_eq!(config.delay_for_attempt(0), Duration::from_secs(0));
    assert_eq!(config.delay_for_attempt(1), Duration::from_secs(1));
    assert_eq!(config.delay_for_attempt(2), Duration::from_secs(2));
    assert_eq!(config.delay_for_attempt(3), Duration::from_secs(4));
    assert_eq!(config.delay_for_attempt(4), Duration::from_secs(8));
    assert_eq!(config.delay_for_attempt(5), Duration::from_secs(16)); // Capped
    assert_eq!(config.delay_for_attempt(6), Duration::from_secs(16)); // Still capped
}

#[test]
fn test_retry_success_first_attempt() {
    let config = RetryConfig::quick();
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(&config, "test_operation", move || {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;
        Ok(42)
    });

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(*counter.lock().unwrap(), 1);
}

#[test]
fn test_retry_success_after_failures() {
    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 1.5,
        use_jitter: false,
    };
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(&config, "test_operation", move || {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;

        if *count < 3 {
            Err(InfraError::Transient {
                operation: "test".to_string(),
                reason: "temporary".to_string(),
                retry_after: None,
            })
        } else {
            Ok(42)
        }
    });

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(*counter.lock().unwrap(), 3);
}

#[test]
fn test_retry_fatal_error_no_retry() {
    let config = RetryConfig::quick();
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(&config, "test_operation", move || {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;

        Err(InfraError::Fatal {
            component: "test".to_string(),
            reason: "fatal error".to_string(),
            context: vec![],
        })
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().is_fatal());
    assert_eq!(*counter.lock().unwrap(), 1); // Should not retry
}

#[test]
fn test_retry_max_attempts_exhausted() {
    let config = RetryConfig {
        max_attempts: 3,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 1.5,
        use_jitter: false,
    };
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();

    let result = retry_with_backoff(&config, "test_operation", move || {
        let mut count = counter_clone.lock().unwrap();
        *count += 1;

        Err(InfraError::Transient {
            operation: "test".to_string(),
            reason: "always fails".to_string(),
            retry_after: None,
        })
    });

    assert!(result.is_err());
    assert_eq!(*counter.lock().unwrap(), 3);
}

#[test]
fn test_preflight_validator_required_commands() {
    let requirements = SystemRequirements {
        min_disk_space_gb: 0,
        min_memory_gb: 0,
        required_commands: vec!["sh".to_string()], // sh should exist
        optional_commands: vec![],
        required_directories: vec![],
        required_endpoints: vec![],
    };

    let validator = PreFlightValidator::new(requirements);
    let result = validator.validate("test");

    #[cfg(unix)]
    assert!(result.is_ok());
}

#[test]
fn test_preflight_validator_missing_command() {
    let requirements = SystemRequirements {
        min_disk_space_gb: 0,
        min_memory_gb: 0,
        required_commands: vec!["nonexistent_command_xyz123".to_string()],
        optional_commands: vec![],
        required_directories: vec![],
        required_endpoints: vec![],
    };

    let validator = PreFlightValidator::new(requirements);
    let result = validator.validate("test");

    assert!(result.is_err());
    if let Err(InfraError::Validation { component, errors }) = result {
        assert_eq!(component, "test");
        assert!(!errors.is_empty());
        assert!(errors[0].message.contains("nonexistent_command_xyz123"));
    } else {
        panic!("Expected validation error");
    }
}

#[test]
fn test_preflight_validator_optional_command_missing() {
    let requirements = SystemRequirements {
        min_disk_space_gb: 0,
        min_memory_gb: 0,
        required_commands: vec![],
        optional_commands: vec!["nonexistent_optional_xyz123".to_string()],
        required_directories: vec![],
        required_endpoints: vec![],
    };

    let validator = PreFlightValidator::new(requirements);
    let result = validator.validate("test");

    // Should pass even with missing optional command
    assert!(result.is_ok());
}

#[test]
fn test_rollback_manager_basic() {
    let mut manager = RollbackManager::new("test_component");
    let executed = Arc::new(Mutex::new(false));
    let executed_clone = executed.clone();

    manager.add_action("test action", Box::new(move || {
        *executed_clone.lock().unwrap() = true;
        Ok(())
    }));

    manager.disable_auto_rollback();
    assert!(!*executed.lock().unwrap());

    let result = manager.rollback();
    assert!(result.is_ok());
    assert!(*executed.lock().unwrap());
}

#[test]
fn test_rollback_manager_multiple_actions() {
    let mut manager = RollbackManager::new("test_component");
    let order = Arc::new(Mutex::new(Vec::new()));

    let order1 = order.clone();
    manager.add_action("action 1", Box::new(move || {
        order1.lock().unwrap().push(1);
        Ok(())
    }));

    let order2 = order.clone();
    manager.add_action("action 2", Box::new(move || {
        order2.lock().unwrap().push(2);
        Ok(())
    }));

    let order3 = order.clone();
    manager.add_action("action 3", Box::new(move || {
        order3.lock().unwrap().push(3);
        Ok(())
    }));

    manager.disable_auto_rollback();
    let result = manager.rollback();
    assert!(result.is_ok());

    // Actions should execute in LIFO order
    let execution_order = order.lock().unwrap();
    assert_eq!(*execution_order, vec![3, 2, 1]);
}

#[test]
fn test_rollback_manager_commit() {
    let mut manager = RollbackManager::new("test_component");
    let executed = Arc::new(Mutex::new(false));
    let executed_clone = executed.clone();

    manager.add_action("test action", Box::new(move || {
        *executed_clone.lock().unwrap() = true;
        Ok(())
    }));

    manager.commit();

    // Action should not execute after commit
    assert!(!*executed.lock().unwrap());
}

#[test]
fn test_rollback_context_tracking() {
    let mut context = RollbackContext::new();

    context.add_file("/tmp/test.txt");
    context.add_directory("/tmp/testdir");
    context.add_k8s_resource("Pod", "test-pod", Some("default".to_string()));
    context.add_helm_release("test-release", "test-namespace");

    assert_eq!(context.files.len(), 1);
    assert_eq!(context.directories.len(), 1);
    assert_eq!(context.k8s_resources.len(), 1);
    assert_eq!(context.helm_releases.len(), 1);
}

#[test]
fn test_health_check_result_all_passed() {
    let mut result = HealthCheckResult::new("test");
    result.add_check("check1", true, "ok");
    result.add_check("check2", true, "ok");
    result.add_check("check3", true, "ok");

    result.evaluate_status();

    assert_eq!(result.status, HealthStatus::Healthy);
    assert!(result.is_healthy());
    assert!(result.to_result().is_ok());
}

#[test]
fn test_health_check_result_some_failed() {
    let mut result = HealthCheckResult::new("test");
    result.add_check("check1", true, "ok");
    result.add_check("check2", false, "failed");
    result.add_check("check3", true, "ok");

    result.evaluate_status();

    assert_eq!(result.status, HealthStatus::Degraded);
    assert!(!result.is_healthy());
    assert!(result.to_result().is_err());
}

#[test]
fn test_health_check_result_all_failed() {
    let mut result = HealthCheckResult::new("test");
    result.add_check("check1", false, "failed");
    result.add_check("check2", false, "failed");

    result.evaluate_status();

    assert_eq!(result.status, HealthStatus::Unhealthy);
    assert!(!result.is_healthy());
    assert!(result.to_result().is_err());
}

#[test]
fn test_health_check_result_no_checks() {
    let mut result = HealthCheckResult::new("test");
    result.evaluate_status();

    assert_eq!(result.status, HealthStatus::Unknown);
}

// Integration test for combined error handling workflow
#[test]
fn test_combined_error_handling_workflow() {
    // Simulate a complete installation workflow with error handling

    // 1. Pre-flight validation
    let requirements = SystemRequirements {
        min_disk_space_gb: 0,
        min_memory_gb: 0,
        required_commands: vec!["sh".to_string()],
        optional_commands: vec![],
        required_directories: vec![],
        required_endpoints: vec![],
    };

    let validator = PreFlightValidator::new(requirements);
    assert!(validator.validate("test").is_ok());

    // 2. Setup rollback manager
    let mut rollback_manager = RollbackManager::new("test_installation");
    let cleanup_executed = Arc::new(Mutex::new(false));
    let cleanup_clone = cleanup_executed.clone();

    rollback_manager.add_action("cleanup", Box::new(move || {
        *cleanup_clone.lock().unwrap() = true;
        Ok(())
    }));

    // 3. Simulate successful operation with retry
    let config = RetryConfig::quick();
    let attempt_counter = Arc::new(Mutex::new(0));
    let attempt_clone = attempt_counter.clone();

    let result = retry_with_backoff(&config, "simulated_operation", move || {
        let mut count = attempt_clone.lock().unwrap();
        *count += 1;

        if *count == 1 {
            // Fail on first attempt
            Err(InfraError::Transient {
                operation: "test".to_string(),
                reason: "temporary failure".to_string(),
                retry_after: None,
            })
        } else {
            // Succeed on second attempt
            Ok(())
        }
    });

    assert!(result.is_ok());
    assert_eq!(*attempt_counter.lock().unwrap(), 2);

    // 4. Commit successful installation (disable rollback)
    rollback_manager.commit();
    assert!(!*cleanup_executed.lock().unwrap());

    // 5. Create health check result
    let mut health = HealthCheckResult::new("test");
    health.add_check("operation_completed", true, "success");
    health.evaluate_status();

    assert!(health.is_healthy());
}

#[test]
fn test_error_handling_with_rollback() {
    // Test that rollback executes on error

    let mut rollback_manager = RollbackManager::new("test_installation");
    let rollback_executed = Arc::new(Mutex::new(false));
    let rollback_clone = rollback_executed.clone();

    rollback_manager.add_action("cleanup_on_error", Box::new(move || {
        *rollback_clone.lock().unwrap() = true;
        Ok(())
    }));

    rollback_manager.disable_auto_rollback();

    // Simulate a fatal error
    let config = RetryConfig {
        max_attempts: 2,
        initial_delay: Duration::from_millis(1),
        max_delay: Duration::from_secs(1),
        backoff_multiplier: 2.0,
        use_jitter: false,
    };

    let result: InfraResult<()> = retry_with_backoff(&config, "failing_operation", || {
        Err(InfraError::Fatal {
            component: "test".to_string(),
            reason: "critical error".to_string(),
            context: vec!["test context".to_string()],
        })
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().is_fatal());

    // Execute rollback
    let rollback_result = rollback_manager.rollback();
    assert!(rollback_result.is_ok());
    assert!(*rollback_executed.lock().unwrap());
}
