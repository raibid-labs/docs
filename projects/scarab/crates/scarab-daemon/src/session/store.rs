use super::{Session, SessionId};
use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// SQLite-based session persistence
pub struct SessionStore {
    db_path: PathBuf,
}

// Explicitly mark SessionStore as Send since Connection is not Send
// but we never send connections across threads (only the path)
unsafe impl Send for SessionStore {}

impl SessionStore {
    /// Create a new session store with database at given path
    pub fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create database directory")?;
        }

        let store = Self { db_path };

        // Initialize database schema
        store.init_schema()?;

        Ok(store)
    }

    /// Get database connection
    fn connect(&self) -> Result<Connection> {
        Connection::open(&self.db_path).context("Failed to open database connection")
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        let conn = self.connect()?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_attached INTEGER NOT NULL,
                cols INTEGER NOT NULL DEFAULT 80,
                rows INTEGER NOT NULL DEFAULT 24
            )",
            [],
        )?;

        // Index for faster lookups by name
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_name ON sessions(name)",
            [],
        )?;

        log::info!("Session database initialized at: {:?}", self.db_path);
        Ok(())
    }

    /// Save a session to the database
    pub fn save_session(&self, session: &Session) -> Result<()> {
        let conn = self.connect()?;

        let created_at = session
            .created_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let last_attached = session
            .last_attached
            .read()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let grid_state = session.grid_state.read();

        conn.execute(
            "INSERT OR REPLACE INTO sessions (id, name, created_at, last_attached, cols, rows)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                &session.id,
                &session.name,
                created_at,
                last_attached,
                grid_state.cols as i64,
                grid_state.rows as i64,
            ],
        )?;

        Ok(())
    }

    /// Load all sessions from the database
    pub fn load_sessions(&self) -> Result<Vec<Session>> {
        let conn = self.connect()?;

        let mut stmt = conn.prepare(
            "SELECT id, name, created_at, last_attached FROM sessions ORDER BY last_attached DESC",
        )?;

        let sessions = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let created_at: i64 = row.get(2)?;
                let last_attached: i64 = row.get(3)?;

                Ok((id, name, created_at, last_attached))
            })?
            .filter_map(|r| r.ok())
            .map(|(id, name, created_at, last_attached)| {
                let created = UNIX_EPOCH + std::time::Duration::from_secs(created_at as u64);
                let attached = UNIX_EPOCH + std::time::Duration::from_secs(last_attached as u64);

                Session::restore(id, name, created, attached)
            })
            .collect();

        Ok(sessions)
    }

    /// Delete a session from the database
    pub fn delete_session(&self, id: &SessionId) -> Result<()> {
        let conn = self.connect()?;

        conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;

        log::info!("Deleted session from storage: {}", id);
        Ok(())
    }

    /// Update last attached timestamp
    pub fn update_last_attached(&self, id: &SessionId) -> Result<()> {
        let conn = self.connect()?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        conn.execute(
            "UPDATE sessions SET last_attached = ?1 WHERE id = ?2",
            params![now, id],
        )?;

        Ok(())
    }

    /// Rename a session
    pub fn rename_session(&self, id: &SessionId, new_name: &str) -> Result<()> {
        let conn = self.connect()?;

        conn.execute(
            "UPDATE sessions SET name = ?1 WHERE id = ?2",
            params![new_name, id],
        )?;

        log::info!("Renamed session {} to {}", id, new_name);
        Ok(())
    }

    /// Get session by ID - Public API for session restoration
    #[allow(dead_code)]
    pub fn get_session(&self, id: &SessionId) -> Result<Option<Session>> {
        let conn = self.connect()?;

        let result = conn
            .query_row(
                "SELECT id, name, created_at, last_attached FROM sessions WHERE id = ?1",
                params![id],
                |row| {
                    let id: String = row.get(0)?;
                    let name: String = row.get(1)?;
                    let created_at: i64 = row.get(2)?;
                    let last_attached: i64 = row.get(3)?;

                    Ok((id, name, created_at, last_attached))
                },
            )
            .optional()?;

        if let Some((id, name, created_at, last_attached)) = result {
            let created = UNIX_EPOCH + std::time::Duration::from_secs(created_at as u64);
            let attached = UNIX_EPOCH + std::time::Duration::from_secs(last_attached as u64);

            Ok(Some(Session::restore(id, name, created, attached)))
        } else {
            Ok(None)
        }
    }

    /// Get session count - Public API for statistics
    #[allow(dead_code)]
    pub fn session_count(&self) -> Result<usize> {
        let conn = self.connect()?;

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_session_store_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let store = SessionStore::new(db_path).unwrap();

        // Create and save a session
        let session = Session::new("test".to_string(), 80, 24).unwrap();
        store.save_session(&session).unwrap();

        // Load sessions
        let sessions = store.load_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].name, "test");

        // Delete session
        store.delete_session(&session.id).unwrap();
        let sessions = store.load_sessions().unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[test]
    fn test_session_rename() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let store = SessionStore::new(db_path).unwrap();

        let session = Session::new("old_name".to_string(), 80, 24).unwrap();
        store.save_session(&session).unwrap();

        store.rename_session(&session.id, "new_name").unwrap();

        let loaded = store.get_session(&session.id).unwrap().unwrap();
        assert_eq!(loaded.name, "new_name");
    }

    #[test]
    fn test_last_attached_update() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let store = SessionStore::new(db_path).unwrap();

        let session = Session::new("test".to_string(), 80, 24).unwrap();
        let id = session.id.clone();
        store.save_session(&session).unwrap();

        // Get initial timestamp from DB (which has second precision)
        let initial_loaded = store.get_session(&id).unwrap().unwrap();
        let initial_attached = *initial_loaded.last_attached.read();

        // Wait for at least 1 second (SQLite stores timestamps with second precision)
        std::thread::sleep(std::time::Duration::from_secs(1));

        store.update_last_attached(&id).unwrap();

        // Verify timestamp was updated in DB
        let loaded = store.get_session(&id).unwrap().unwrap();
        let loaded_attached = *loaded.last_attached.read();

        // The loaded timestamp should be at least 1 second later
        assert!(
            loaded_attached > initial_attached,
            "Expected loaded timestamp ({:?}) to be later than initial ({:?})",
            loaded_attached,
            initial_attached
        );
    }
}
