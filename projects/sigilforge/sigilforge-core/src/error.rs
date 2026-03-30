//! Top-level error types for Sigilforge.

use thiserror::Error;

use crate::store::StoreError;
use crate::token::TokenError;
use crate::resolve::ResolveError;

/// Top-level error type encompassing all Sigilforge errors.
#[derive(Debug, Error)]
pub enum SigilforgeError {
    /// Error from secret storage operations.
    #[error("store error: {0}")]
    Store(#[from] StoreError),

    /// Error from token operations.
    #[error("token error: {0}")]
    Token(#[from] TokenError),

    /// Error from reference resolution.
    #[error("resolve error: {0}")]
    Resolve(#[from] ResolveError),

    /// Configuration error.
    #[error("configuration error: {message}")]
    Config { message: String },

    /// Generic internal error.
    #[error("internal error: {message}")]
    Internal { message: String },
}
