//! JSON-RPC API handlers for the daemon.

use super::types::{
    AccountInfo, AddAccountResponse, GetTokenResponse, ListAccountsResponse, ResolveResponse,
};
use anyhow::Result;
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::types::{ErrorCode, ErrorObject};
use tracing::{debug, info};

use sigilforge_core::{
    account_store::AccountStore,
    model::{Account, AccountId, ServiceId},
};
use std::sync::Arc;

/// State shared across RPC handlers.
pub struct ApiState {
    /// Persistent account store
    pub accounts: Arc<AccountStore>,
}

impl ApiState {
    /// Create a new API state.
    pub fn new() -> Result<Self> {
        let accounts = AccountStore::load()?;
        Ok(Self {
            accounts: Arc::new(accounts),
        })
    }

    /// Create API state with a provided account store (useful for tests).
    #[allow(dead_code)]
    pub fn with_store(accounts: AccountStore) -> Self {
        Self {
            accounts: Arc::new(accounts),
        }
    }
}

impl Default for ApiState {
    fn default() -> Self {
        Self::new().expect("failed to load AccountStore")
    }
}

/// JSON-RPC API trait definition.
#[rpc(server)]
pub trait SigilforgeApi {
    /// Get a fresh access token for the specified account.
    ///
    /// # Parameters
    ///
    /// - `service`: Service identifier (e.g., "spotify")
    /// - `account`: Account identifier (e.g., "personal")
    ///
    /// # Returns
    ///
    /// A fresh access token and optional expiration timestamp.
    #[method(name = "get_token")]
    async fn get_token(&self, service: String, account: String) -> RpcResult<GetTokenResponse>;

    /// List all configured accounts, optionally filtered by service.
    ///
    /// # Parameters
    ///
    /// - `service`: Optional service filter
    ///
    /// # Returns
    ///
    /// List of configured accounts.
    #[method(name = "list_accounts")]
    async fn list_accounts(&self, service: Option<String>) -> RpcResult<ListAccountsResponse>;

    /// Add a new account with OAuth flow.
    ///
    /// # Parameters
    ///
    /// - `service`: Service identifier
    /// - `account`: Account identifier
    /// - `scopes`: OAuth scopes to request
    ///
    /// # Returns
    ///
    /// Confirmation message.
    #[method(name = "add_account")]
    async fn add_account(
        &self,
        service: String,
        account: String,
        scopes: Vec<String>,
    ) -> RpcResult<AddAccountResponse>;

    /// Resolve a credential reference to its actual value.
    ///
    /// # Parameters
    ///
    /// - `reference`: Credential reference (e.g., "auth://spotify/personal/token")
    ///
    /// # Returns
    ///
    /// The resolved credential value.
    #[method(name = "resolve")]
    async fn resolve(&self, reference: String) -> RpcResult<ResolveResponse>;
}

/// Implementation of the Sigilforge API.
pub struct SigilforgeApiImpl {
    state: ApiState,
}

impl SigilforgeApiImpl {
    /// Create a new API implementation with the given state.
    pub fn new(state: ApiState) -> Self {
        Self { state }
    }
}

#[async_trait::async_trait]
impl SigilforgeApiServer for SigilforgeApiImpl {
    async fn get_token(&self, service: String, account: String) -> RpcResult<GetTokenResponse> {
        info!("RPC: get_token({}/{})", service, account);

        // Check if account exists
        let service_id = ServiceId::new(&service);
        let account_id = AccountId::new(&account);
        if self
            .state
            .accounts
            .get_account(&service_id, &account_id)
            .map_err(internal_error)?
            .is_none()
        {
            return Err(ErrorObject::owned(
                ErrorCode::InvalidParams.code(),
                format!("Account {}/{} not found", service, account),
                None::<()>,
            ));
        }

        // Update last_used timestamp
        let _ = self
            .state
            .accounts
            .update_last_used(&service_id, &account_id)
            .map_err(internal_error);

        // TODO: Integrate with actual token manager
        // For now, return a stub token
        Ok(GetTokenResponse {
            token: format!("stub_token_for_{}_{}", service, account),
            expires_at: Some("2025-12-07T00:00:00Z".to_string()),
        })
    }

    async fn list_accounts(&self, service: Option<String>) -> RpcResult<ListAccountsResponse> {
        debug!("RPC: list_accounts(service: {:?})", service);

        let service_filter = service.as_ref().map(ServiceId::new);
        let accounts = self
            .state
            .accounts
            .list_accounts(service_filter.as_ref())
            .map_err(internal_error)?;

        let filtered: Vec<AccountInfo> = accounts
            .into_iter()
            .map(|a| AccountInfo {
                service: a.service.to_string(),
                account: a.id.to_string(),
                scopes: a.scopes,
                created_at: a.created_at.to_rfc3339(),
                last_used: a.last_used.map(|dt| dt.to_rfc3339()),
            })
            .collect();

        Ok(ListAccountsResponse {
            accounts: filtered,
        })
    }

    async fn add_account(
        &self,
        service: String,
        account: String,
        scopes: Vec<String>,
    ) -> RpcResult<AddAccountResponse> {
        info!("RPC: add_account({}/{}, scopes: {:?})", service, account, scopes);

        let new_account = Account::new(
            ServiceId::new(&service),
            AccountId::new(&account),
            scopes,
        );

        if let Err(e) = self.state.accounts.add_account(new_account) {
            return Err(match e {
                sigilforge_core::account_store::AccountStoreError::AlreadyExists { .. } => {
                    ErrorObject::owned(
                        ErrorCode::InvalidParams.code(),
                        format!("Account {}/{} already exists", service, account),
                        None::<()>,
                    )
                }
                other => internal_error(other),
            });
        }

        Ok(AddAccountResponse {
            message: format!("Account {}/{} added successfully", service, account),
        })
    }

    async fn resolve(&self, reference: String) -> RpcResult<ResolveResponse> {
        info!("RPC: resolve({})", reference);

        // Parse the reference
        use sigilforge_core::CredentialRef;
        let cred_ref = CredentialRef::from_auth_uri(&reference).map_err(|e| {
            ErrorObject::owned(
                ErrorCode::InvalidParams.code(),
                format!("Invalid reference: {}", e),
                None::<()>,
            )
        })?;

        // Check if account exists
        let account_exists = self
            .state
            .accounts
            .get_account(&cred_ref.service, &cred_ref.account)
            .map_err(internal_error)?
            .is_some();

        if !account_exists {
            return Err(ErrorObject::owned(
                ErrorCode::InvalidParams.code(),
                format!("Account {}/{} not found", cred_ref.service, cred_ref.account),
                None::<()>,
            ));
        }

        // TODO: Integrate with actual resolver
        // For now, return a stub value
        Ok(ResolveResponse {
            value: format!(
                "resolved_{}_{}_{}",
                cred_ref.service, cred_ref.account, cred_ref.credential_type
            ),
        })
    }
}

fn internal_error<E: std::fmt::Display>(err: E) -> ErrorObject<'static> {
    ErrorObject::owned(
        ErrorCode::InternalError.code(),
        format!("{}", err),
        None::<()>,
    )
}
