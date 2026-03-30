//! Account metadata persistence.
//!
//! This module provides disk-backed storage for account metadata using JSON
//! serialization and platform-specific configuration directories.
//!
//! # Storage Location
//!
//! Accounts are stored at `~/.config/sigilforge/accounts.json` on Linux/macOS
//! and `%APPDATA%\sigilforge\accounts.json` on Windows.
//!
//! # Example
//!
//! ```rust,ignore
//! use sigilforge_core::account_store::AccountStore;
//! use sigilforge_core::{ServiceId, AccountId, Account};
//!
//! let store = AccountStore::load()?;
//! let account = Account::new(
//!     ServiceId::new("spotify"),
//!     AccountId::new("personal"),
//!     vec!["user-read-email".to_string()],
//! );
//! store.add_account(account)?;
//! ```

use crate::model::{Account, AccountId, ServiceId};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use thiserror::Error;

/// Error type for account store operations.
#[derive(Debug, Error)]
pub enum AccountStoreError {
    /// Account already exists.
    #[error("account {service}/{account} already exists")]
    AlreadyExists { service: String, account: String },

    /// Account not found.
    #[error("account {service}/{account} not found")]
    NotFound { service: String, account: String },

    /// I/O error reading or writing the store.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Configuration directory not available.
    #[error("configuration directory not available")]
    ConfigDirUnavailable,

    /// Internal lock poisoning error.
    #[error("internal lock error: {message}")]
    LockError { message: String },
}

/// Internal storage format for accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountStoreData {
    /// Version of the store format (for future migrations).
    version: u32,

    /// All stored accounts.
    accounts: Vec<Account>,
}

impl Default for AccountStoreData {
    fn default() -> Self {
        Self {
            version: 1,
            accounts: Vec::new(),
        }
    }
}

/// Disk-backed account metadata store.
///
/// This store manages account metadata persistence using JSON files in the
/// platform-specific configuration directory.
///
/// # Thread Safety
///
/// This implementation uses interior mutability via `RwLock` and is safe to
/// share across threads via `Arc`.
pub struct AccountStore {
    /// Path to the accounts JSON file.
    path: PathBuf,

    /// In-memory cache of account data.
    data: Arc<RwLock<AccountStoreData>>,
}

impl AccountStore {
    /// Get the default storage path for accounts.
    ///
    /// Returns the platform-specific configuration directory path for the
    /// accounts.json file.
    pub fn default_path() -> Result<PathBuf, AccountStoreError> {
        let dirs = directories::ProjectDirs::from("com", "raibid-labs", "sigilforge")
            .ok_or(AccountStoreError::ConfigDirUnavailable)?;

        let config_dir = dirs.config_dir();
        Ok(config_dir.join("accounts.json"))
    }

    /// Load the account store from the default location.
    ///
    /// Creates the file and parent directories if they don't exist.
    pub fn load() -> Result<Self, AccountStoreError> {
        let path = Self::default_path()?;
        Self::load_from_path(path)
    }

    /// Load the account store from a specific path.
    ///
    /// Creates the file and parent directories if they don't exist.
    pub fn load_from_path(path: PathBuf) -> Result<Self, AccountStoreError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Load or create the data file
        let data = if path.exists() {
            let contents = fs::read_to_string(&path)?;
            serde_json::from_str(&contents)?
        } else {
            AccountStoreData::default()
        };

        Ok(Self {
            path,
            data: Arc::new(RwLock::new(data)),
        })
    }

    /// Save the current state to disk.
    fn save(&self) -> Result<(), AccountStoreError> {
        let data = self.data.read().map_err(|e| AccountStoreError::LockError {
            message: format!("read lock poisoned: {}", e),
        })?;

        let contents = serde_json::to_string_pretty(&*data)?;
        fs::write(&self.path, contents)?;

        Ok(())
    }

    /// Add a new account to the store.
    ///
    /// Returns an error if an account with the same service/id already exists.
    pub fn add_account(&self, account: Account) -> Result<(), AccountStoreError> {
        let mut data = self.data.write().map_err(|e| AccountStoreError::LockError {
            message: format!("write lock poisoned: {}", e),
        })?;

        // Check for duplicates
        if data
            .accounts
            .iter()
            .any(|a| a.service == account.service && a.id == account.id)
        {
            return Err(AccountStoreError::AlreadyExists {
                service: account.service.to_string(),
                account: account.id.to_string(),
            });
        }

        data.accounts.push(account);
        drop(data);

        self.save()
    }

    /// Get an account by service and account ID.
    ///
    /// Returns `Ok(None)` if the account doesn't exist.
    pub fn get_account(
        &self,
        service: &ServiceId,
        account: &AccountId,
    ) -> Result<Option<Account>, AccountStoreError> {
        let data = self.data.read().map_err(|e| AccountStoreError::LockError {
            message: format!("read lock poisoned: {}", e),
        })?;

        Ok(data
            .accounts
            .iter()
            .find(|a| &a.service == service && &a.id == account)
            .cloned())
    }

    /// List all accounts, optionally filtered by service.
    ///
    /// If `service_filter` is `Some`, only accounts for that service are returned.
    /// If `service_filter` is `None`, all accounts are returned.
    pub fn list_accounts(
        &self,
        service_filter: Option<&ServiceId>,
    ) -> Result<Vec<Account>, AccountStoreError> {
        let data = self.data.read().map_err(|e| AccountStoreError::LockError {
            message: format!("read lock poisoned: {}", e),
        })?;

        let accounts = if let Some(service) = service_filter {
            data.accounts
                .iter()
                .filter(|a| &a.service == service)
                .cloned()
                .collect()
        } else {
            data.accounts.clone()
        };

        Ok(accounts)
    }

    /// Remove an account from the store.
    ///
    /// Returns an error if the account doesn't exist.
    pub fn remove_account(
        &self,
        service: &ServiceId,
        account: &AccountId,
    ) -> Result<(), AccountStoreError> {
        let mut data = self.data.write().map_err(|e| AccountStoreError::LockError {
            message: format!("write lock poisoned: {}", e),
        })?;

        let initial_len = data.accounts.len();
        data.accounts
            .retain(|a| &a.service != service || &a.id != account);

        if data.accounts.len() == initial_len {
            return Err(AccountStoreError::NotFound {
                service: service.to_string(),
                account: account.to_string(),
            });
        }

        drop(data);

        self.save()
    }

    /// Update the last_used timestamp for an account.
    ///
    /// This is called when a token is fetched for the account.
    pub fn update_last_used(
        &self,
        service: &ServiceId,
        account: &AccountId,
    ) -> Result<(), AccountStoreError> {
        let mut data = self.data.write().map_err(|e| AccountStoreError::LockError {
            message: format!("write lock poisoned: {}", e),
        })?;

        let account_entry = data
            .accounts
            .iter_mut()
            .find(|a| &a.service == service && &a.id == account)
            .ok_or_else(|| AccountStoreError::NotFound {
                service: service.to_string(),
                account: account.to_string(),
            })?;

        account_entry.last_used = Some(chrono::Utc::now());
        drop(data);

        self.save()
    }

    /// Get the storage path for this store.
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_account() -> Account {
        Account::new(
            ServiceId::new("spotify"),
            AccountId::new("personal"),
            vec!["user-read-email".to_string()],
        )
    }

    fn test_store() -> (AccountStore, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("accounts.json");
        let store = AccountStore::load_from_path(path).unwrap();
        (store, temp_dir)
    }

    #[test]
    fn test_add_and_get_account() {
        let (store, _temp) = test_store();
        let account = test_account();

        store.add_account(account.clone()).unwrap();

        let retrieved = store
            .get_account(&account.service, &account.id)
            .unwrap()
            .unwrap();

        assert_eq!(retrieved.service, account.service);
        assert_eq!(retrieved.id, account.id);
        assert_eq!(retrieved.scopes, account.scopes);
    }

    #[test]
    fn test_add_duplicate_account() {
        let (store, _temp) = test_store();
        let account = test_account();

        store.add_account(account.clone()).unwrap();
        let result = store.add_account(account);

        assert!(matches!(
            result,
            Err(AccountStoreError::AlreadyExists { .. })
        ));
    }

    #[test]
    fn test_list_all_accounts() {
        let (store, _temp) = test_store();

        let account1 = Account::new(
            ServiceId::new("spotify"),
            AccountId::new("personal"),
            vec![],
        );
        let account2 = Account::new(
            ServiceId::new("spotify"),
            AccountId::new("work"),
            vec![],
        );
        let account3 = Account::new(
            ServiceId::new("github"),
            AccountId::new("main"),
            vec![],
        );

        store.add_account(account1).unwrap();
        store.add_account(account2).unwrap();
        store.add_account(account3).unwrap();

        let all_accounts = store.list_accounts(None).unwrap();
        assert_eq!(all_accounts.len(), 3);
    }

    #[test]
    fn test_list_accounts_filtered() {
        let (store, _temp) = test_store();

        let account1 = Account::new(
            ServiceId::new("spotify"),
            AccountId::new("personal"),
            vec![],
        );
        let account2 = Account::new(
            ServiceId::new("spotify"),
            AccountId::new("work"),
            vec![],
        );
        let account3 = Account::new(
            ServiceId::new("github"),
            AccountId::new("main"),
            vec![],
        );

        store.add_account(account1).unwrap();
        store.add_account(account2).unwrap();
        store.add_account(account3).unwrap();

        let spotify_accounts = store
            .list_accounts(Some(&ServiceId::new("spotify")))
            .unwrap();
        assert_eq!(spotify_accounts.len(), 2);

        let github_accounts = store
            .list_accounts(Some(&ServiceId::new("github")))
            .unwrap();
        assert_eq!(github_accounts.len(), 1);
    }

    #[test]
    fn test_remove_account() {
        let (store, _temp) = test_store();
        let account = test_account();

        store.add_account(account.clone()).unwrap();
        store
            .remove_account(&account.service, &account.id)
            .unwrap();

        let retrieved = store.get_account(&account.service, &account.id).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_remove_nonexistent_account() {
        let (store, _temp) = test_store();

        let result = store.remove_account(
            &ServiceId::new("spotify"),
            &AccountId::new("nonexistent"),
        );

        assert!(matches!(result, Err(AccountStoreError::NotFound { .. })));
    }

    #[test]
    fn test_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("accounts.json");

        // Create store and add account
        {
            let store = AccountStore::load_from_path(path.clone()).unwrap();
            let account = test_account();
            store.add_account(account).unwrap();
        }

        // Load store again and verify account persisted
        {
            let store = AccountStore::load_from_path(path).unwrap();
            let accounts = store.list_accounts(None).unwrap();
            assert_eq!(accounts.len(), 1);
            assert_eq!(accounts[0].service.as_str(), "spotify");
            assert_eq!(accounts[0].id.as_str(), "personal");
        }
    }

    #[test]
    fn test_update_last_used() {
        let (store, _temp) = test_store();
        let account = test_account();

        store.add_account(account.clone()).unwrap();

        // Initially last_used should be None
        let retrieved = store
            .get_account(&account.service, &account.id)
            .unwrap()
            .unwrap();
        assert!(retrieved.last_used.is_none());

        // Update last_used
        store
            .update_last_used(&account.service, &account.id)
            .unwrap();

        // Verify last_used is now set
        let retrieved = store
            .get_account(&account.service, &account.id)
            .unwrap()
            .unwrap();
        assert!(retrieved.last_used.is_some());
    }
}
