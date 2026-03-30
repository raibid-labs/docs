//! Integration tests for account lifecycle operations.
//!
//! These tests verify the end-to-end functionality of account management:
//! - Adding accounts
//! - Listing accounts
//! - Retrieving specific accounts
//! - Removing accounts
//! - Error handling for edge cases

use sigilforge_core::{Account, AccountId, AccountStore, AccountStoreError, ServiceId};
use tempfile::TempDir;

/// Helper to create a test store in a temporary directory.
fn test_store() -> (AccountStore, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("accounts.json");
    let store = AccountStore::load_from_path(path).unwrap();
    (store, temp_dir)
}

/// Helper to create a test account.
fn test_account(service: &str, account: &str, scopes: Vec<&str>) -> Account {
    Account::new(
        ServiceId::new(service),
        AccountId::new(account),
        scopes.into_iter().map(String::from).collect(),
    )
}

#[test]
fn test_add_account_happy_path() {
    let (store, _temp) = test_store();

    let account = test_account("spotify", "personal", vec!["user-read-email"]);
    let result = store.add_account(account);

    assert!(result.is_ok(), "Should successfully add account");

    let retrieved = store
        .get_account(&ServiceId::new("spotify"), &AccountId::new("personal"))
        .unwrap();

    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.service.as_str(), "spotify");
    assert_eq!(retrieved.id.as_str(), "personal");
    assert_eq!(retrieved.scopes, vec!["user-read-email"]);
}

#[test]
fn test_add_duplicate_account_fails() {
    let (store, _temp) = test_store();

    let account1 = test_account("spotify", "personal", vec![]);
    let account2 = test_account("spotify", "personal", vec!["different-scope"]);

    store.add_account(account1).unwrap();
    let result = store.add_account(account2);

    assert!(result.is_err(), "Should fail when adding duplicate account");
    assert!(
        matches!(result, Err(AccountStoreError::AlreadyExists { .. })),
        "Error should be AlreadyExists"
    );
}

#[test]
fn test_list_all_accounts() {
    let (store, _temp) = test_store();

    let account1 = test_account("spotify", "personal", vec![]);
    let account2 = test_account("spotify", "work", vec![]);
    let account3 = test_account("github", "main", vec![]);

    store.add_account(account1).unwrap();
    store.add_account(account2).unwrap();
    store.add_account(account3).unwrap();

    let all_accounts = store.list_accounts(None).unwrap();

    assert_eq!(all_accounts.len(), 3, "Should return all 3 accounts");
}

#[test]
fn test_list_accounts_filtered_by_service() {
    let (store, _temp) = test_store();

    let account1 = test_account("spotify", "personal", vec![]);
    let account2 = test_account("spotify", "work", vec![]);
    let account3 = test_account("github", "main", vec![]);

    store.add_account(account1).unwrap();
    store.add_account(account2).unwrap();
    store.add_account(account3).unwrap();

    let spotify_accounts = store
        .list_accounts(Some(&ServiceId::new("spotify")))
        .unwrap();

    assert_eq!(
        spotify_accounts.len(),
        2,
        "Should return 2 spotify accounts"
    );
    assert!(spotify_accounts
        .iter()
        .all(|a| a.service.as_str() == "spotify"));

    let github_accounts = store
        .list_accounts(Some(&ServiceId::new("github")))
        .unwrap();

    assert_eq!(github_accounts.len(), 1, "Should return 1 github account");
}

#[test]
fn test_list_accounts_empty() {
    let (store, _temp) = test_store();

    let accounts = store.list_accounts(None).unwrap();

    assert_eq!(accounts.len(), 0, "Should return empty list");
}

#[test]
fn test_get_account_exists() {
    let (store, _temp) = test_store();

    let account = test_account("spotify", "personal", vec!["user-read-email"]);
    store.add_account(account).unwrap();

    let retrieved = store
        .get_account(&ServiceId::new("spotify"), &AccountId::new("personal"))
        .unwrap();

    assert!(retrieved.is_some(), "Account should exist");
}

#[test]
fn test_get_account_not_found() {
    let (store, _temp) = test_store();

    let retrieved = store
        .get_account(
            &ServiceId::new("nonexistent"),
            &AccountId::new("account"),
        )
        .unwrap();

    assert!(retrieved.is_none(), "Account should not exist");
}

#[test]
fn test_remove_account_happy_path() {
    let (store, _temp) = test_store();

    let account = test_account("spotify", "personal", vec![]);
    store.add_account(account).unwrap();

    let result = store.remove_account(&ServiceId::new("spotify"), &AccountId::new("personal"));

    assert!(result.is_ok(), "Should successfully remove account");

    let retrieved = store
        .get_account(&ServiceId::new("spotify"), &AccountId::new("personal"))
        .unwrap();

    assert!(retrieved.is_none(), "Account should be removed");
}

#[test]
fn test_remove_nonexistent_account_fails() {
    let (store, _temp) = test_store();

    let result = store.remove_account(
        &ServiceId::new("spotify"),
        &AccountId::new("nonexistent"),
    );

    assert!(result.is_err(), "Should fail when removing nonexistent account");
    assert!(
        matches!(result, Err(AccountStoreError::NotFound { .. })),
        "Error should be NotFound"
    );
}

#[test]
fn test_account_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("accounts.json");

    // Create store and add accounts
    {
        let store = AccountStore::load_from_path(path.clone()).unwrap();
        let account1 = test_account("spotify", "personal", vec!["scope1"]);
        let account2 = test_account("github", "work", vec!["scope2", "scope3"]);

        store.add_account(account1).unwrap();
        store.add_account(account2).unwrap();
    }

    // Load store again and verify persistence
    {
        let store = AccountStore::load_from_path(path).unwrap();
        let accounts = store.list_accounts(None).unwrap();

        assert_eq!(accounts.len(), 2, "Should persist 2 accounts");

        let spotify_account = accounts
            .iter()
            .find(|a| a.service.as_str() == "spotify")
            .unwrap();
        assert_eq!(spotify_account.id.as_str(), "personal");
        assert_eq!(spotify_account.scopes, vec!["scope1"]);

        let github_account = accounts
            .iter()
            .find(|a| a.service.as_str() == "github")
            .unwrap();
        assert_eq!(github_account.id.as_str(), "work");
        assert_eq!(github_account.scopes, vec!["scope2", "scope3"]);
    }
}

#[test]
fn test_update_last_used() {
    let (store, _temp) = test_store();

    let account = test_account("spotify", "personal", vec![]);
    store.add_account(account).unwrap();

    // Initially last_used should be None
    let retrieved = store
        .get_account(&ServiceId::new("spotify"), &AccountId::new("personal"))
        .unwrap()
        .unwrap();
    assert!(retrieved.last_used.is_none());

    // Update last_used
    store
        .update_last_used(&ServiceId::new("spotify"), &AccountId::new("personal"))
        .unwrap();

    // Verify last_used is now set
    let retrieved = store
        .get_account(&ServiceId::new("spotify"), &AccountId::new("personal"))
        .unwrap()
        .unwrap();
    assert!(retrieved.last_used.is_some());
}

#[test]
fn test_update_last_used_nonexistent_account() {
    let (store, _temp) = test_store();

    let result = store.update_last_used(
        &ServiceId::new("spotify"),
        &AccountId::new("nonexistent"),
    );

    assert!(result.is_err(), "Should fail for nonexistent account");
    assert!(
        matches!(result, Err(AccountStoreError::NotFound { .. })),
        "Error should be NotFound"
    );
}

#[test]
fn test_multiple_accounts_same_service() {
    let (store, _temp) = test_store();

    let personal = test_account("spotify", "personal", vec!["scope1"]);
    let work = test_account("spotify", "work", vec!["scope2"]);
    let family = test_account("spotify", "family", vec!["scope3"]);

    store.add_account(personal).unwrap();
    store.add_account(work).unwrap();
    store.add_account(family).unwrap();

    let accounts = store
        .list_accounts(Some(&ServiceId::new("spotify")))
        .unwrap();

    assert_eq!(accounts.len(), 3, "Should support multiple accounts per service");

    let account_ids: Vec<_> = accounts.iter().map(|a| a.id.as_str()).collect();
    assert!(account_ids.contains(&"personal"));
    assert!(account_ids.contains(&"work"));
    assert!(account_ids.contains(&"family"));
}

#[test]
fn test_account_key_generation() {
    let account = test_account("spotify", "personal", vec![]);
    let key = account.key();

    assert_eq!(key, "spotify/personal", "Should generate correct key format");
}

#[test]
fn test_service_id_normalization() {
    let (store, _temp) = test_store();

    // Add account with uppercase service name
    let account = Account::new(
        ServiceId::new("SPOTIFY"),
        AccountId::new("personal"),
        vec![],
    );
    store.add_account(account).unwrap();

    // Retrieve with lowercase
    let retrieved = store
        .get_account(&ServiceId::new("spotify"), &AccountId::new("personal"))
        .unwrap();

    assert!(
        retrieved.is_some(),
        "Service IDs should be normalized to lowercase"
    );
}

#[test]
fn test_empty_scopes() {
    let (store, _temp) = test_store();

    let account = test_account("github", "main", vec![]);
    store.add_account(account).unwrap();

    let retrieved = store
        .get_account(&ServiceId::new("github"), &AccountId::new("main"))
        .unwrap()
        .unwrap();

    assert_eq!(
        retrieved.scopes.len(),
        0,
        "Should support accounts with no scopes"
    );
}

#[test]
fn test_complex_scopes() {
    let (store, _temp) = test_store();

    let scopes = vec![
        "user-read-email",
        "user-read-private",
        "playlist-modify-public",
        "playlist-modify-private",
    ];
    let account = test_account("spotify", "personal", scopes.clone());
    store.add_account(account).unwrap();

    let retrieved = store
        .get_account(&ServiceId::new("spotify"), &AccountId::new("personal"))
        .unwrap()
        .unwrap();

    assert_eq!(retrieved.scopes, scopes, "Should preserve all scopes");
}
