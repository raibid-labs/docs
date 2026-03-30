//! API request/response types for the daemon JSON-RPC interface.

use serde::{Deserialize, Serialize};

/// Request to get a fresh access token for an account.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTokenRequest {
    /// Service identifier (e.g., "spotify", "github")
    pub service: String,
    /// Account identifier (e.g., "personal", "work")
    pub account: String,
}

/// Response containing a fresh access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTokenResponse {
    /// The access token value
    pub token: String,
    /// Optional expiration timestamp (ISO 8601)
    pub expires_at: Option<String>,
}

/// Request to add a new account.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAccountRequest {
    /// Service identifier
    pub service: String,
    /// Account identifier
    pub account: String,
    /// OAuth scopes to request
    pub scopes: Vec<String>,
}

/// Response after successfully adding an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAccountResponse {
    /// Confirmation message
    pub message: String,
}

/// Request to list accounts.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAccountsRequest {
    /// Optional service filter
    pub service: Option<String>,
}

/// Information about a configured account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// Service identifier
    pub service: String,
    /// Account identifier
    pub account: String,
    /// Granted scopes
    pub scopes: Vec<String>,
    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last used timestamp (ISO 8601), if ever used
    pub last_used: Option<String>,
}

/// Response containing a list of accounts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAccountsResponse {
    /// List of configured accounts
    pub accounts: Vec<AccountInfo>,
}

/// Request to resolve a credential reference.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveRequest {
    /// The reference to resolve (e.g., "auth://spotify/personal/token")
    pub reference: String,
}

/// Response containing the resolved credential value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveResponse {
    /// The resolved credential value
    pub value: String,
}
