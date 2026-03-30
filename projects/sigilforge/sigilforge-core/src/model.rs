//! Domain model types for Sigilforge.
//!
//! This module defines the core types used throughout Sigilforge:
//! - [`ServiceId`] - Identifier for a service (e.g., "spotify", "gmail")
//! - [`AccountId`] - Identifier for an account within a service
//! - [`Account`] - Full account metadata
//! - [`CredentialRef`] - Reference to a stored credential
//! - [`CredentialType`] - Type of credential (token, api_key, etc.)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Identifier for a service (e.g., "spotify", "gmail", "github").
///
/// Service IDs should be lowercase and use hyphens for multi-word names.
///
/// # Examples
///
/// ```
/// use sigilforge_core::ServiceId;
///
/// let spotify = ServiceId::new("spotify");
/// let ms_graph = ServiceId::new("msgraph");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServiceId(String);

impl ServiceId {
    /// Create a new service ID.
    ///
    /// The ID is normalized to lowercase.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into().to_lowercase())
    }

    /// Get the service ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ServiceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for ServiceId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ServiceId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Identifier for an account within a service.
///
/// Account IDs allow multiple accounts per service (e.g., "personal", "work").
///
/// # Examples
///
/// ```
/// use sigilforge_core::AccountId;
///
/// let personal = AccountId::new("personal");
/// let work = AccountId::new("work");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AccountId(String);

impl AccountId {
    /// Create a new account ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the account ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for AccountId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for AccountId {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

/// Full account metadata.
///
/// Represents a configured account with its service, ID, granted scopes,
/// and timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// The service this account belongs to.
    pub service: ServiceId,

    /// The account identifier within the service.
    pub id: AccountId,

    /// OAuth scopes or permissions granted to this account.
    pub scopes: Vec<String>,

    /// When the account was first configured.
    pub created_at: DateTime<Utc>,

    /// When the account was last used to fetch a token.
    pub last_used: Option<DateTime<Utc>>,
}

impl Account {
    /// Create a new account with the current timestamp.
    pub fn new(service: ServiceId, id: AccountId, scopes: Vec<String>) -> Self {
        Self {
            service,
            id,
            scopes,
            created_at: Utc::now(),
            last_used: None,
        }
    }

    /// Create a unique key for this account.
    pub fn key(&self) -> String {
        format!("{}/{}", self.service, self.id)
    }
}

/// Type of credential stored for an account.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialType {
    /// OAuth access token (short-lived).
    AccessToken,

    /// OAuth refresh token (long-lived).
    RefreshToken,

    /// Token expiry timestamp.
    TokenExpiry,

    /// Static API key.
    ApiKey,

    /// OAuth client ID (usually at provider level).
    ClientId,

    /// OAuth client secret (usually at provider level).
    ClientSecret,

    /// OAuth scopes (comma-separated).
    TokenScopes,

    /// Custom credential type.
    Custom(String),
}

impl CredentialType {
    /// Get the credential type as a string for storage keys.
    pub fn as_str(&self) -> &str {
        match self {
            Self::AccessToken => "access_token",
            Self::RefreshToken => "refresh_token",
            Self::TokenExpiry => "token_expiry",
            Self::ApiKey => "api_key",
            Self::ClientId => "client_id",
            Self::ClientSecret => "client_secret",
            Self::TokenScopes => "token_scopes",
            Self::Custom(s) => s,
        }
    }
}

impl fmt::Display for CredentialType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Reference to a stored credential.
///
/// Used to construct storage keys and parse `auth://` URIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CredentialRef {
    /// The service the credential belongs to.
    pub service: ServiceId,

    /// The account the credential belongs to.
    pub account: AccountId,

    /// The type of credential.
    pub credential_type: CredentialType,
}

impl CredentialRef {
    /// Create a new credential reference.
    pub fn new(
        service: impl Into<ServiceId>,
        account: impl Into<AccountId>,
        credential_type: CredentialType,
    ) -> Self {
        Self {
            service: service.into(),
            account: account.into(),
            credential_type,
        }
    }

    /// Convert to a storage key.
    ///
    /// Keys follow the pattern: `sigilforge/{service}/{account}/{type}`
    pub fn to_key(&self) -> String {
        format!(
            "sigilforge/{}/{}/{}",
            self.service, self.account, self.credential_type
        )
    }

    /// Parse from an `auth://` URI.
    ///
    /// # Format
    ///
    /// `auth://service/account/credential_type`
    ///
    /// # Examples
    ///
    /// ```
    /// use sigilforge_core::{CredentialRef, CredentialType};
    ///
    /// let cred = CredentialRef::from_auth_uri("auth://spotify/personal/token").unwrap();
    /// assert_eq!(cred.service.as_str(), "spotify");
    /// assert_eq!(cred.account.as_str(), "personal");
    /// assert_eq!(cred.credential_type, CredentialType::AccessToken);
    /// ```
    pub fn from_auth_uri(uri: &str) -> Result<Self, ParseError> {
        // Check prefix
        let path = uri
            .strip_prefix("auth://")
            .ok_or_else(|| ParseError::InvalidScheme {
                expected: "auth".to_string(),
                got: uri.split("://").next().unwrap_or("").to_string(),
            })?;

        // Split into parts
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() != 3 {
            return Err(ParseError::InvalidPath {
                message: format!(
                    "expected 3 path components (service/account/type), got {}",
                    parts.len()
                ),
            });
        }

        let service = ServiceId::new(parts[0]);
        let account = AccountId::new(parts[1]);
        let credential_type = match parts[2] {
            "token" | "access_token" => CredentialType::AccessToken,
            "refresh_token" => CredentialType::RefreshToken,
            "api_key" => CredentialType::ApiKey,
            "client_id" => CredentialType::ClientId,
            "client_secret" => CredentialType::ClientSecret,
            other => CredentialType::Custom(other.to_string()),
        };

        Ok(Self {
            service,
            account,
            credential_type,
        })
    }

    /// Convert to an `auth://` URI.
    pub fn to_auth_uri(&self) -> String {
        format!(
            "auth://{}/{}/{}",
            self.service, self.account, self.credential_type
        )
    }
}

/// Error parsing a credential reference.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid scheme: expected '{expected}', got '{got}'")]
    InvalidScheme { expected: String, got: String },

    #[error("invalid path: {message}")]
    InvalidPath { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_id_normalization() {
        let id = ServiceId::new("SPOTIFY");
        assert_eq!(id.as_str(), "spotify");
    }

    #[test]
    fn test_credential_ref_to_key() {
        let cred = CredentialRef::new("spotify", "personal", CredentialType::AccessToken);
        assert_eq!(cred.to_key(), "sigilforge/spotify/personal/access_token");
    }

    #[test]
    fn test_credential_ref_from_auth_uri() {
        let cred = CredentialRef::from_auth_uri("auth://spotify/personal/token").unwrap();
        assert_eq!(cred.service.as_str(), "spotify");
        assert_eq!(cred.account.as_str(), "personal");
        assert_eq!(cred.credential_type, CredentialType::AccessToken);
    }

    #[test]
    fn test_credential_ref_roundtrip() {
        let original = CredentialRef::new("gmail", "work", CredentialType::RefreshToken);
        let uri = original.to_auth_uri();
        let parsed = CredentialRef::from_auth_uri(&uri).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn test_invalid_auth_uri_scheme() {
        let result = CredentialRef::from_auth_uri("https://spotify/personal/token");
        assert!(matches!(result, Err(ParseError::InvalidScheme { .. })));
    }

    #[test]
    fn test_invalid_auth_uri_path() {
        let result = CredentialRef::from_auth_uri("auth://spotify/personal");
        assert!(matches!(result, Err(ParseError::InvalidPath { .. })));
    }
}
