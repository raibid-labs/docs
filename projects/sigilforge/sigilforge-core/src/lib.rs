//! # Sigilforge Core
//!
//! Core library for Sigilforge credential management.
//!
//! This crate provides:
//! - Domain types for services, accounts, and credentials
//! - Traits for secret storage, token management, and reference resolution
//! - In-memory and (optionally) keyring-based storage implementations
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use sigilforge_core::{ServiceId, AccountId, TokenManager};
//!
//! async fn get_token(manager: &impl TokenManager) -> Result<String, sigilforge_core::TokenError> {
//!     let service = ServiceId::new("spotify");
//!     let account = AccountId::new("personal");
//!     let token = manager.ensure_access_token(&service, &account).await?;
//!     Ok(token.access_token.expose().to_string())
//! }
//! ```

pub mod model;
pub mod store;
pub mod token;
pub mod resolve;
pub mod error;
pub mod account_store;

#[cfg(feature = "oauth")]
pub mod provider;

#[cfg(feature = "oauth")]
pub mod token_manager;

#[cfg(feature = "oauth")]
pub mod oauth;

// Re-export commonly used types at crate root
pub use model::{
    ServiceId,
    AccountId,
    Account,
    CredentialRef,
    CredentialType,
};

pub use store::{
    Secret,
    SecretStore,
    StoreError,
    MemoryStore,
    create_store,
};

#[cfg(feature = "keyring-store")]
pub use store::KeyringStore;

pub use token::{
    Token,
    TokenSet,
    TokenInfo,
    TokenManager,
    TokenError,
};

pub use resolve::{
    ResolvedValue,
    ReferenceResolver,
    ResolveError,
};

pub use error::SigilforgeError;

pub use account_store::{
    AccountStore,
    AccountStoreError,
};

#[cfg(feature = "oauth")]
pub use provider::{
    ProviderConfig,
    ProviderRegistry,
};

#[cfg(feature = "oauth")]
pub use token_manager::DefaultTokenManager;
