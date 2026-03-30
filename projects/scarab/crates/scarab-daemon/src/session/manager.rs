use super::{ClientId, GridState, SessionId, SessionStore};
use anyhow::{bail, Result};
use parking_lot::RwLock;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// A single terminal session with PTY and state
pub struct Session {
    pub id: SessionId,
    pub name: String,
    /// PTY master for reading/writing terminal data (public API for VTE integration)
    #[allow(dead_code)]
    pub pty_master: Arc<RwLock<Option<Box<dyn portable_pty::MasterPty + Send + Sync>>>>,
    pub grid_state: Arc<RwLock<GridState>>,
    pub created_at: SystemTime,
    pub last_attached: Arc<RwLock<SystemTime>>,
    pub attached_clients: Arc<RwLock<HashSet<ClientId>>>,
}

// Session is Sync because all interior mutability is behind locks
unsafe impl Sync for Session {}

impl Session {
    /// Create a new session with a PTY
    pub fn new(name: String, cols: u16, rows: u16) -> Result<Self> {
        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        // Spawn shell in PTY
        let cmd = CommandBuilder::new("bash");
        let _child = pair.slave.spawn_command(cmd)?;

        // Extract master and slave separately to avoid partial move
        let master: Box<dyn portable_pty::MasterPty + Send + Sync> =
            unsafe { std::mem::transmute(pair.master) };
        let _slave = pair.slave;
        // Slave is dropped here, released in parent process

        let id = Uuid::new_v4().to_string();
        let now = SystemTime::now();

        Ok(Self {
            id,
            name,
            pty_master: Arc::new(RwLock::new(Some(master))),
            grid_state: Arc::new(RwLock::new(GridState::new(cols, rows))),
            created_at: now,
            last_attached: Arc::new(RwLock::new(now)),
            attached_clients: Arc::new(RwLock::new(HashSet::new())),
        })
    }

    /// Restore session from persisted data (without PTY initially)
    pub fn restore(
        id: SessionId,
        name: String,
        created_at: SystemTime,
        last_attached: SystemTime,
    ) -> Self {
        Self {
            id,
            name,
            pty_master: Arc::new(RwLock::new(None)),
            grid_state: Arc::new(RwLock::new(GridState::new(80, 24))),
            created_at,
            last_attached: Arc::new(RwLock::new(last_attached)),
            attached_clients: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// Attach a client to this session
    pub fn attach_client(&self, client_id: ClientId) {
        let mut clients = self.attached_clients.write();
        clients.insert(client_id);
        *self.last_attached.write() = SystemTime::now();
    }

    /// Detach a client from this session
    pub fn detach_client(&self, client_id: ClientId) {
        let mut clients = self.attached_clients.write();
        clients.remove(&client_id);
    }

    /// Check if session has any attached clients
    pub fn has_attached_clients(&self) -> bool {
        !self.attached_clients.read().is_empty()
    }

    /// Get attached client count
    pub fn attached_client_count(&self) -> usize {
        self.attached_clients.read().len()
    }

    /// Resize the PTY (public API for client resize events)
    #[allow(dead_code)]
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        if let Some(ref master) = *self.pty_master.read() {
            master.resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })?;

            let mut state = self.grid_state.write();
            state.cols = cols;
            state.rows = rows;
        }
        Ok(())
    }

    /// Get PTY master for reading/writing (public API for VTE integration)
    #[allow(dead_code)]
    pub fn pty_master(
        &self,
    ) -> Arc<RwLock<Option<Box<dyn portable_pty::MasterPty + Send + Sync>>>> {
        Arc::clone(&self.pty_master)
    }
}

/// Manages multiple sessions
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Arc<Session>>>>,
    store: SessionStore,
    default_session_id: Arc<RwLock<Option<SessionId>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(db_path: std::path::PathBuf) -> Result<Self> {
        let store = SessionStore::new(db_path)?;

        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            store,
            default_session_id: Arc::new(RwLock::new(None)),
        })
    }

    /// Initialize from persisted sessions
    pub fn restore_sessions(&self) -> Result<()> {
        let persisted = self.store.load_sessions()?;

        let mut sessions = self.sessions.write();
        for session in persisted {
            let session = Arc::new(session);
            sessions.insert(session.id.clone(), session);
        }

        log::info!("Restored {} sessions from storage", sessions.len());
        Ok(())
    }

    /// Create a new session
    pub fn create_session(&self, name: String, cols: u16, rows: u16) -> Result<SessionId> {
        let session = Session::new(name.clone(), cols, rows)?;
        let id = session.id.clone();

        // Persist to database
        self.store.save_session(&session)?;

        // Add to active sessions
        let mut sessions = self.sessions.write();
        sessions.insert(id.clone(), Arc::new(session));

        // Set as default if first session
        if sessions.len() == 1 {
            *self.default_session_id.write() = Some(id.clone());
        }

        log::info!("Created session: {} ({})", name, id);
        Ok(id)
    }

    /// Delete a session
    pub fn delete_session(&self, id: &SessionId) -> Result<()> {
        let mut sessions = self.sessions.write();

        if let Some(session) = sessions.remove(id) {
            // Check if any clients are attached
            if session.has_attached_clients() {
                bail!("Cannot delete session with attached clients");
            }

            // Remove from database
            self.store.delete_session(id)?;

            // Clear default if this was it
            let mut default_id = self.default_session_id.write();
            if default_id.as_ref() == Some(id) {
                *default_id = sessions.keys().next().cloned();
            }

            log::info!("Deleted session: {}", id);
            Ok(())
        } else {
            bail!("Session not found: {}", id)
        }
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &SessionId) -> Option<Arc<Session>> {
        self.sessions.read().get(id).cloned()
    }

    /// Get default session (or first available) - Public API for session routing
    #[allow(dead_code)]
    pub fn get_default_session(&self) -> Option<Arc<Session>> {
        let sessions = self.sessions.read();

        if let Some(id) = self.default_session_id.read().as_ref() {
            sessions.get(id).cloned()
        } else {
            sessions.values().next().cloned()
        }
    }

    /// List all sessions
    pub fn list_sessions(&self) -> Vec<(SessionId, String, u64, u64, usize)> {
        let sessions = self.sessions.read();
        sessions
            .values()
            .map(|s| {
                let created = s
                    .created_at
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let last_attached = s
                    .last_attached
                    .read()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let client_count = s.attached_client_count();

                (
                    s.id.clone(),
                    s.name.clone(),
                    created,
                    last_attached,
                    client_count,
                )
            })
            .collect()
    }

    /// Attach a client to a session
    pub fn attach_client(&self, session_id: &SessionId, client_id: ClientId) -> Result<()> {
        if let Some(session) = self.get_session(session_id) {
            session.attach_client(client_id);

            // Update persistence
            self.store.update_last_attached(session_id)?;

            log::info!("Client {} attached to session {}", client_id, session_id);
            Ok(())
        } else {
            bail!("Session not found: {}", session_id)
        }
    }

    /// Detach a client from a session
    pub fn detach_client(&self, session_id: &SessionId, client_id: ClientId) -> Result<()> {
        if let Some(session) = self.get_session(session_id) {
            session.detach_client(client_id);
            log::info!("Client {} detached from session {}", client_id, session_id);
            Ok(())
        } else {
            bail!("Session not found: {}", session_id)
        }
    }

    /// Rename a session
    pub fn rename_session(&self, session_id: &SessionId, new_name: String) -> Result<()> {
        let sessions = self.sessions.read();

        if let Some(_session) = sessions.get(session_id) {
            // Update in persistence layer (in-memory name would need RwLock wrapper for full mutability)
            self.store.rename_session(session_id, &new_name)?;
            log::info!("Renamed session {} to {}", session_id, new_name);
            Ok(())
        } else {
            bail!("Session not found: {}", session_id)
        }
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.read().len()
    }

    /// Cleanup sessions with no attached clients (optional maintenance) - Public API
    #[allow(dead_code)]
    pub fn cleanup_detached_sessions(&self, max_age_secs: u64) -> Result<usize> {
        let _now = SystemTime::now();
        let sessions_to_delete: Vec<SessionId> = {
            let sessions = self.sessions.read();
            sessions
                .values()
                .filter(|s| {
                    !s.has_attached_clients()
                        && s.last_attached
                            .read()
                            .elapsed()
                            .map(|d| d.as_secs() > max_age_secs)
                            .unwrap_or(false)
                })
                .map(|s| s.id.clone())
                .collect()
        };

        let count = sessions_to_delete.len();
        for id in sessions_to_delete {
            self.delete_session(&id)?;
        }

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_lifecycle() {
        let session = Session::new("test".to_string(), 80, 24).unwrap();
        assert_eq!(session.name, "test");
        assert_eq!(session.attached_client_count(), 0);

        session.attach_client(1);
        assert_eq!(session.attached_client_count(), 1);
        assert!(session.has_attached_clients());

        session.detach_client(1);
        assert_eq!(session.attached_client_count(), 0);
        assert!(!session.has_attached_clients());
    }

    #[test]
    fn test_session_manager_create_delete() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("sessions.db");

        let manager = SessionManager::new(db_path).unwrap();

        let id = manager.create_session("test".to_string(), 80, 24).unwrap();
        assert_eq!(manager.session_count(), 1);

        manager.delete_session(&id).unwrap();
        assert_eq!(manager.session_count(), 0);
    }

    #[test]
    fn test_session_attach_detach() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("sessions.db");

        let manager = SessionManager::new(db_path).unwrap();
        let id = manager.create_session("test".to_string(), 80, 24).unwrap();

        manager.attach_client(&id, 1).unwrap();
        let session = manager.get_session(&id).unwrap();
        assert!(session.has_attached_clients());

        manager.detach_client(&id, 1).unwrap();
        assert!(!session.has_attached_clients());
    }
}
