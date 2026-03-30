use scarab_daemon::session::SessionManager;
use std::time::Duration;
use tempfile::TempDir;

#[test]
fn test_session_creation_and_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    // Create manager and session
    let manager = SessionManager::new(db_path.clone()).unwrap();
    let id1 = manager
        .create_session("session1".to_string(), 80, 24)
        .unwrap();
    let _id2 = manager
        .create_session("session2".to_string(), 100, 30)
        .unwrap();

    assert_eq!(manager.session_count(), 2);

    // Create new manager instance to test persistence
    let manager2 = SessionManager::new(db_path).unwrap();
    manager2.restore_sessions().unwrap();

    assert_eq!(manager2.session_count(), 2);

    let session = manager2.get_session(&id1).unwrap();
    assert_eq!(session.name, "session1");
}

#[test]
fn test_session_attach_detach() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let id = manager.create_session("test".to_string(), 80, 24).unwrap();

    // Attach multiple clients
    manager.attach_client(&id, 1).unwrap();
    manager.attach_client(&id, 2).unwrap();
    manager.attach_client(&id, 3).unwrap();

    let session = manager.get_session(&id).unwrap();
    assert_eq!(session.attached_client_count(), 3);

    // Detach one client
    manager.detach_client(&id, 2).unwrap();
    assert_eq!(session.attached_client_count(), 2);

    // Cannot delete session with attached clients
    assert!(manager.delete_session(&id).is_err());

    // Detach remaining clients
    manager.detach_client(&id, 1).unwrap();
    manager.detach_client(&id, 3).unwrap();

    // Now deletion should work
    assert!(manager.delete_session(&id).is_ok());
}

#[test]
fn test_session_listing() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();

    let _id1 = manager.create_session("alpha".to_string(), 80, 24).unwrap();
    let _id2 = manager.create_session("beta".to_string(), 100, 30).unwrap();
    let _id3 = manager
        .create_session("gamma".to_string(), 120, 40)
        .unwrap();

    let sessions = manager.list_sessions();
    assert_eq!(sessions.len(), 3);

    let names: Vec<String> = sessions
        .iter()
        .map(|(_, name, _, _, _)| name.clone())
        .collect();
    assert!(names.contains(&"alpha".to_string()));
    assert!(names.contains(&"beta".to_string()));
    assert!(names.contains(&"gamma".to_string()));
}

#[test]
fn test_session_rename() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path.clone()).unwrap();
    let id = manager
        .create_session("old_name".to_string(), 80, 24)
        .unwrap();

    manager.rename_session(&id, "new_name".to_string()).unwrap();

    // Verify rename persisted
    let manager2 = SessionManager::new(db_path).unwrap();
    manager2.restore_sessions().unwrap();

    let session = manager2.get_session(&id).unwrap();
    assert_eq!(session.name, "new_name");
}

#[test]
fn test_default_session() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();

    // No default when empty
    assert!(manager.get_default_session().is_none());

    let id1 = manager.create_session("first".to_string(), 80, 24).unwrap();

    // First session becomes default
    let default = manager.get_default_session().unwrap();
    assert_eq!(default.id, id1);

    let _id2 = manager
        .create_session("second".to_string(), 80, 24)
        .unwrap();

    // Default should still be first session
    let default = manager.get_default_session().unwrap();
    assert_eq!(default.id, id1);
}

#[test]
fn test_session_cleanup_detached() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();

    let id1 = manager
        .create_session("attached".to_string(), 80, 24)
        .unwrap();
    let id2 = manager
        .create_session("detached_old".to_string(), 80, 24)
        .unwrap();

    // Attach client to first session
    manager.attach_client(&id1, 1).unwrap();

    // Wait briefly
    std::thread::sleep(Duration::from_millis(100));

    // Cleanup sessions older than 0 seconds (should only remove detached ones)
    let removed = manager.cleanup_detached_sessions(0).unwrap();

    assert_eq!(removed, 1);
    assert_eq!(manager.session_count(), 1);
    assert!(manager.get_session(&id1).is_some());
    assert!(manager.get_session(&id2).is_none());
}

#[test]
fn test_concurrent_session_operations() {
    use std::sync::Arc;
    use std::thread;

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = Arc::new(SessionManager::new(db_path).unwrap());

    // Create sessions concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            manager_clone
                .create_session(format!("session_{}", i), 80, 24)
                .unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(manager.session_count(), 10);
}

#[test]
fn test_session_resurrection() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    // First daemon instance
    {
        let manager = SessionManager::new(db_path.clone()).unwrap();
        let _id1 = manager
            .create_session("persistent1".to_string(), 80, 24)
            .unwrap();
        let _id2 = manager
            .create_session("persistent2".to_string(), 100, 30)
            .unwrap();

        assert_eq!(manager.session_count(), 2);
    } // Manager dropped, simulating daemon shutdown

    // Second daemon instance - should resurrect sessions
    {
        let manager = SessionManager::new(db_path).unwrap();
        manager.restore_sessions().unwrap();

        assert_eq!(manager.session_count(), 2);

        let sessions = manager.list_sessions();
        let names: Vec<String> = sessions
            .iter()
            .map(|(_, name, _, _, _)| name.clone())
            .collect();

        assert!(names.contains(&"persistent1".to_string()));
        assert!(names.contains(&"persistent2".to_string()));
    }
}

#[test]
fn test_session_memory_efficiency() {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();

    // Create 50+ sessions to test scalability
    for i in 0..60 {
        manager
            .create_session(format!("session_{}", i), 80, 24)
            .unwrap();
    }

    assert_eq!(manager.session_count(), 60);

    // Verify all sessions are accessible
    let sessions = manager.list_sessions();
    assert_eq!(sessions.len(), 60);
}

#[tokio::test]
async fn test_session_attach_performance() {
    use std::time::Instant;

    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("sessions.db");

    let manager = SessionManager::new(db_path).unwrap();
    let id1 = manager
        .create_session("perf_test".to_string(), 80, 24)
        .unwrap();
    let id2 = manager
        .create_session("perf_test2".to_string(), 80, 24)
        .unwrap();

    // Measure session switch time
    manager.attach_client(&id1, 1).unwrap();

    let start = Instant::now();
    manager.detach_client(&id1, 1).unwrap();
    manager.attach_client(&id2, 1).unwrap();
    let elapsed = start.elapsed();

    println!("Session switch time: {:?}", elapsed);

    // Should be well under 10ms target
    assert!(
        elapsed.as_millis() < 10,
        "Session switch took {:?}, expected <10ms",
        elapsed
    );
}
