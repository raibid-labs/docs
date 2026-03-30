//! # fusabi-streams-core
//!
//! Core traits and types for stream-based information providers in the Fusabi ecosystem.
//!
//! This crate defines the fundamental abstractions used by Scryforge and other
//! Fusabi-powered applications that work with information streams:
//!
//! - [`Stream`] - A logical feed or collection (inbox, playlist, subreddit, etc.)
//! - [`Item`] - An entry within a stream (email, article, video, track, etc.)
//! - [`Action`] - Operations that can be performed on items
//! - Provider capability traits: [`HasFeeds`], [`HasCollections`], [`HasSavedItems`], [`HasCommunities`]
//!
//! ## Authentication
//!
//! Providers that require OAuth tokens can use the `TokenFetcher` trait from
//! `scryforge-sigilforge-client` to fetch credentials from the Sigilforge daemon.
//! Enable the `sigilforge` feature to access authentication utilities.
//!
//! ```toml
//! [dependencies]
//! fusabi-streams-core = { version = "0.1", features = ["sigilforge"] }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum StreamError {
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Stream not found: {0}")]
    StreamNotFound(String),

    #[error("Item not found: {0}")]
    ItemNotFound(String),

    #[error("Authentication required: {0}")]
    AuthRequired(String),

    #[error("Rate limited: retry after {0} seconds")]
    RateLimited(u64),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, StreamError>;

// ============================================================================
// Core ID Types
// ============================================================================

/// Unique identifier for a stream.
/// Format: `{provider}:{type}:{local_id}` e.g., "email:inbox:gmail-main"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StreamId(pub String);

impl StreamId {
    pub fn new(provider: &str, stream_type: &str, local_id: &str) -> Self {
        Self(format!("{provider}:{stream_type}:{local_id}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Unique identifier for an item.
/// Format: `{provider}:{item_id}` e.g., "email:msg-12345"
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemId(pub String);

impl ItemId {
    pub fn new(provider: &str, local_id: &str) -> Self {
        Self(format!("{provider}:{local_id}"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Unique identifier for a feed within a provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeedId(pub String);

/// Unique identifier for a collection within a provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CollectionId(pub String);

/// Unique identifier for a community within a provider.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommunityId(pub String);

// ============================================================================
// Stream and Item Types
// ============================================================================

/// A logical feed or collection of items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stream {
    pub id: StreamId,
    pub name: String,
    pub provider_id: String,
    pub stream_type: StreamType,
    pub icon: Option<String>,
    pub unread_count: Option<u32>,
    pub total_count: Option<u32>,
    pub last_updated: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

/// The type/category of a stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StreamType {
    Feed,
    Collection,
    SavedItems,
    Community,
    Custom(String),
}

/// An entry within a stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub stream_id: StreamId,
    pub title: String,
    pub content: ItemContent,
    pub author: Option<Author>,
    pub published: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_read: bool,
    pub is_saved: bool,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

/// The content/body of an item, varying by type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemContent {
    /// Plain text content
    Text(String),

    /// Markdown content
    Markdown(String),

    /// HTML content (for rendering in compatible viewers)
    Html(String),

    /// Email-specific content
    Email {
        subject: String,
        body_text: Option<String>,
        body_html: Option<String>,
        snippet: String,
    },

    /// RSS/Article content
    Article {
        summary: Option<String>,
        full_content: Option<String>,
    },

    /// Video content
    Video {
        description: String,
        duration_seconds: Option<u32>,
        view_count: Option<u64>,
    },

    /// Audio/Track content
    Track {
        album: Option<String>,
        duration_ms: Option<u32>,
        artists: Vec<String>,
    },

    /// Task/Todo content
    Task {
        body: Option<String>,
        due_date: Option<NaiveDate>,
        is_completed: bool,
    },

    /// Calendar event content
    Event {
        description: Option<String>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        location: Option<String>,
        is_all_day: bool,
    },

    /// Bookmark content
    Bookmark { description: Option<String> },

    /// Generic/fallback content
    Generic { body: Option<String> },
}

/// Author/creator information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
    pub url: Option<String>,
    pub avatar_url: Option<String>,
}

// ============================================================================
// Actions
// ============================================================================

/// An action that can be performed on an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: ActionKind,
    pub keyboard_shortcut: Option<String>,
}

/// The type of action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionKind {
    /// Open item in default viewer/browser
    Open,
    /// Show preview within TUI
    Preview,
    /// Copy URL/link to clipboard
    CopyLink,
    /// Open in external browser
    OpenInBrowser,
    /// Add a local tag
    TagLocal,
    /// Mark as read
    MarkRead,
    /// Mark as unread
    MarkUnread,
    /// Save/bookmark item
    Save,
    /// Unsave/remove bookmark
    Unsave,
    /// Archive item
    Archive,
    /// Delete item
    Delete,
    /// Add to collection/playlist
    AddToCollection,
    /// Remove from collection/playlist
    RemoveFromCollection,
    /// Custom action
    Custom(String),
}

/// Result of executing an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}

// ============================================================================
// Provider Base Trait
// ============================================================================

/// Health status of a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub is_healthy: bool,
    pub message: Option<String>,
    pub last_sync: Option<DateTime<Utc>>,
    pub error_count: u32,
}

/// Result of a sync operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub items_added: u32,
    pub items_updated: u32,
    pub items_removed: u32,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

/// Capabilities that a provider supports.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub has_feeds: bool,
    pub has_collections: bool,
    pub has_saved_items: bool,
    pub has_communities: bool,
}

/// Base trait for all providers.
///
/// Every provider must implement this trait. Additional capabilities
/// are expressed through the `Has*` traits.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Unique identifier for this provider type (e.g., "email-imap", "rss", "spotify")
    fn id(&self) -> &'static str;

    /// Human-readable name for display
    fn name(&self) -> &'static str;

    /// Check provider health and connectivity
    async fn health_check(&self) -> Result<ProviderHealth>;

    /// Trigger a sync operation to fetch new data
    async fn sync(&self) -> Result<SyncResult>;

    /// Get the capabilities this provider supports
    fn capabilities(&self) -> ProviderCapabilities;

    /// Get available actions for an item
    async fn available_actions(&self, item: &Item) -> Result<Vec<Action>>;

    /// Execute an action on an item
    async fn execute_action(&self, item: &Item, action: &Action) -> Result<ActionResult>;
}

// ============================================================================
// Capability Traits
// ============================================================================

/// A feed within a provider (e.g., an inbox, RSS feed, subreddit).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feed {
    pub id: FeedId,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub unread_count: Option<u32>,
    pub total_count: Option<u32>,
}

/// Options for fetching feed items.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeedOptions {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub since: Option<DateTime<Utc>>,
    pub include_read: bool,
}

/// Providers that have feeds (streams of items over time).
///
/// Examples: Email inboxes, RSS feeds, Reddit home/subreddits, YouTube subscriptions
#[async_trait]
pub trait HasFeeds: Provider {
    /// List all available feeds
    async fn list_feeds(&self) -> Result<Vec<Feed>>;

    /// Get items from a specific feed
    async fn get_feed_items(&self, feed_id: &FeedId, options: FeedOptions) -> Result<Vec<Item>>;
}

/// A named collection of items.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: CollectionId,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub item_count: u32,
    pub is_editable: bool,
    pub owner: Option<String>,
}

/// Providers that have collections (ordered sets of items).
///
/// Examples: Spotify playlists, YouTube playlists, bookmark folders
#[async_trait]
pub trait HasCollections: Provider {
    /// List all collections
    async fn list_collections(&self) -> Result<Vec<Collection>>;

    /// Get items in a collection (ordered)
    async fn get_collection_items(&self, collection_id: &CollectionId) -> Result<Vec<Item>>;

    // TODO: Phase 4 - write operations
    // async fn add_to_collection(&self, collection_id: &CollectionId, item_id: &ItemId) -> Result<()>;
    // async fn remove_from_collection(&self, collection_id: &CollectionId, item_id: &ItemId) -> Result<()>;
    // async fn reorder_collection(&self, collection_id: &CollectionId, item_ids: &[ItemId]) -> Result<()>;
}

/// Options for fetching saved items.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SavedItemsOptions {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub category: Option<String>,
}

/// Providers that have saved/bookmarked/liked items.
///
/// Examples: Reddit saved, YouTube Watch Later, Spotify Liked Songs
#[async_trait]
pub trait HasSavedItems: Provider {
    /// Get all saved items
    async fn get_saved_items(&self, options: SavedItemsOptions) -> Result<Vec<Item>>;

    /// Check if a specific item is saved
    async fn is_saved(&self, item_id: &ItemId) -> Result<bool>;

    // TODO: Phase 4 - write operations
    // async fn save_item(&self, item_id: &ItemId) -> Result<()>;
    // async fn unsave_item(&self, item_id: &ItemId) -> Result<()>;
}

/// A community or subscription (subreddit, channel, publication).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: CommunityId,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub member_count: Option<u64>,
    pub url: Option<String>,
}

/// Providers that have communities/subscriptions.
///
/// Examples: Reddit subreddits, YouTube channels, Medium publications
#[async_trait]
pub trait HasCommunities: Provider {
    /// List subscribed communities
    async fn list_communities(&self) -> Result<Vec<Community>>;

    /// Get details for a specific community
    async fn get_community(&self, id: &CommunityId) -> Result<Community>;
}

// ============================================================================
// Authentication Support (Optional)
// ============================================================================

#[cfg(feature = "sigilforge")]
pub mod auth {
    //! Authentication utilities for providers that need OAuth tokens.
    //!
    //! This module re-exports types from `scryforge-sigilforge-client` to make it
    //! easier for providers to request authentication tokens.
    //!
    //! # Example
    //!
    //! ```no_run
    //! use fusabi_streams_core::auth::{TokenFetcher, MockTokenFetcher};
    //! use std::sync::Arc;
    //!
    //! struct MyProvider {
    //!     token_fetcher: Arc<dyn TokenFetcher>,
    //! }
    //!
    //! impl MyProvider {
    //!     pub fn new(token_fetcher: Arc<dyn TokenFetcher>) -> Self {
    //!         Self { token_fetcher }
    //!     }
    //!
    //!     pub async fn make_api_call(&self) -> Result<(), Box<dyn std::error::Error>> {
    //!         let token = self.token_fetcher.fetch_token("spotify", "personal").await?;
    //!         // Use token for API calls...
    //!         Ok(())
    //!     }
    //! }
    //! ```

    pub use scryforge_sigilforge_client::{
        default_socket_path, MockTokenFetcher, SigilforgeClient, SigilforgeError, TokenFetcher,
    };
}

// ============================================================================
// Re-exports
// ============================================================================

pub mod prelude {
    pub use crate::{
        Action, ActionKind, ActionResult, Author, Collection, CollectionId, Community, CommunityId,
        Feed, FeedId, FeedOptions, HasCollections, HasCommunities, HasFeeds, HasSavedItems, Item,
        ItemContent, ItemId, Provider, ProviderCapabilities, ProviderHealth, Result,
        SavedItemsOptions, Stream, StreamError, StreamId, StreamType, SyncResult,
    };

    #[cfg(feature = "sigilforge")]
    pub use crate::auth;
}
