//! JSON-RPC API handlers for scryforge-daemon.
//!
//! This module defines the RPC interface and provides implementations
//! that return dummy data for now (Phase 2 will wire up actual providers).

use chrono::Utc;
use fusabi_streams_core::{Item, ItemContent, ItemId, Stream, StreamId, StreamType};
use jsonrpsee::core::RpcResult;
use jsonrpsee::proc_macros::rpc;
use std::collections::HashMap;

/// The main JSON-RPC API interface for Scryforge.
///
/// This trait defines all available RPC methods that clients can call.
#[rpc(server)]
pub trait ScryforgeApi {
    /// List all available streams across all providers.
    #[method(name = "streams.list")]
    async fn list_streams(&self) -> RpcResult<Vec<Stream>>;

    /// List items for a specific stream.
    #[method(name = "items.list")]
    async fn list_items(&self, stream_id: String) -> RpcResult<Vec<Item>>;
}

/// Implementation of the Scryforge API.
///
/// Currently returns hardcoded dummy data. In Phase 2, this will
/// delegate to the ProviderRegistry to fetch real data.
pub struct ApiImpl;

impl ApiImpl {
    pub fn new() -> Self {
        Self
    }

    /// Generate dummy streams for testing.
    fn generate_dummy_streams() -> Vec<Stream> {
        vec![
            Stream {
                id: StreamId::new("email", "inbox", "gmail"),
                name: "Gmail Inbox".to_string(),
                provider_id: "email-imap".to_string(),
                stream_type: StreamType::Feed,
                icon: Some("📧".to_string()),
                unread_count: Some(5),
                total_count: Some(150),
                last_updated: Some(Utc::now()),
                metadata: HashMap::new(),
            },
            Stream {
                id: StreamId::new("rss", "feed", "hackernews"),
                name: "Hacker News".to_string(),
                provider_id: "rss".to_string(),
                stream_type: StreamType::Feed,
                icon: Some("📰".to_string()),
                unread_count: Some(42),
                total_count: Some(100),
                last_updated: Some(Utc::now()),
                metadata: HashMap::new(),
            },
            Stream {
                id: StreamId::new("reddit", "home", "default"),
                name: "Reddit Home".to_string(),
                provider_id: "reddit".to_string(),
                stream_type: StreamType::Feed,
                icon: Some("🔴".to_string()),
                unread_count: None,
                total_count: None,
                last_updated: Some(Utc::now()),
                metadata: HashMap::new(),
            },
            Stream {
                id: StreamId::new("spotify", "collection", "liked"),
                name: "Liked Songs".to_string(),
                provider_id: "spotify".to_string(),
                stream_type: StreamType::SavedItems,
                icon: Some("💚".to_string()),
                unread_count: None,
                total_count: Some(523),
                last_updated: Some(Utc::now()),
                metadata: HashMap::new(),
            },
        ]
    }

    /// Generate dummy items for a stream.
    fn generate_dummy_items(stream_id: &str) -> Vec<Item> {
        // Return different items based on stream
        if stream_id.starts_with("email:") {
            vec![
                Item {
                    id: ItemId::new("email", "msg-001"),
                    stream_id: StreamId(stream_id.to_string()),
                    title: "Meeting tomorrow at 10am".to_string(),
                    content: ItemContent::Email {
                        subject: "Meeting tomorrow at 10am".to_string(),
                        body_text: Some(
                            "Hi,\n\nJust a reminder about our meeting tomorrow.\n\nBest,\nJohn"
                                .to_string(),
                        ),
                        body_html: None,
                        snippet: "Just a reminder about our meeting...".to_string(),
                    },
                    author: Some(fusabi_streams_core::Author {
                        name: "John Doe".to_string(),
                        email: Some("john@example.com".to_string()),
                        url: None,
                        avatar_url: None,
                    }),
                    published: Some(Utc::now()),
                    updated: None,
                    url: None,
                    thumbnail_url: None,
                    is_read: false,
                    is_saved: false,
                    tags: vec![],
                    metadata: HashMap::new(),
                },
                Item {
                    id: ItemId::new("email", "msg-002"),
                    stream_id: StreamId(stream_id.to_string()),
                    title: "Your order has shipped".to_string(),
                    content: ItemContent::Email {
                        subject: "Your order has shipped".to_string(),
                        body_text: Some(
                            "Your order #12345 has shipped and will arrive by Friday.".to_string(),
                        ),
                        body_html: None,
                        snippet: "Your order #12345 has shipped...".to_string(),
                    },
                    author: Some(fusabi_streams_core::Author {
                        name: "Shop Support".to_string(),
                        email: Some("support@shop.com".to_string()),
                        url: None,
                        avatar_url: None,
                    }),
                    published: Some(Utc::now()),
                    updated: None,
                    url: None,
                    thumbnail_url: None,
                    is_read: true,
                    is_saved: false,
                    tags: vec![],
                    metadata: HashMap::new(),
                },
            ]
        } else if stream_id.starts_with("rss:") {
            vec![
                Item {
                    id: ItemId::new("rss", "article-001"),
                    stream_id: StreamId(stream_id.to_string()),
                    title: "Show HN: A new Rust TUI framework".to_string(),
                    content: ItemContent::Article {
                        summary: Some(
                            "I've been working on a new TUI framework in Rust...".to_string(),
                        ),
                        full_content: None,
                    },
                    author: Some(fusabi_streams_core::Author {
                        name: "rustdev".to_string(),
                        email: None,
                        url: None,
                        avatar_url: None,
                    }),
                    published: Some(Utc::now()),
                    updated: None,
                    url: Some("https://news.ycombinator.com/item?id=123".to_string()),
                    thumbnail_url: None,
                    is_read: false,
                    is_saved: false,
                    tags: vec![],
                    metadata: HashMap::new(),
                },
                Item {
                    id: ItemId::new("rss", "article-002"),
                    stream_id: StreamId(stream_id.to_string()),
                    title: "Rust 1.75 Released".to_string(),
                    content: ItemContent::Article {
                        summary: Some("The Rust team is happy to announce a new version of Rust...".to_string()),
                        full_content: None,
                    },
                    author: Some(fusabi_streams_core::Author {
                        name: "Rust Blog".to_string(),
                        email: None,
                        url: Some("https://blog.rust-lang.org".to_string()),
                        avatar_url: None,
                    }),
                    published: Some(Utc::now()),
                    updated: None,
                    url: Some("https://blog.rust-lang.org/2023/12/28/Rust-1.75.0.html".to_string()),
                    thumbnail_url: None,
                    is_read: false,
                    is_saved: false,
                    tags: vec![],
                    metadata: HashMap::new(),
                },
            ]
        } else if stream_id.starts_with("spotify:") {
            vec![
                Item {
                    id: ItemId::new("spotify", "track-001"),
                    stream_id: StreamId(stream_id.to_string()),
                    title: "Example Song".to_string(),
                    content: ItemContent::Track {
                        album: Some("Example Album".to_string()),
                        duration_ms: Some(210000),
                        artists: vec!["Example Artist".to_string()],
                    },
                    author: Some(fusabi_streams_core::Author {
                        name: "Example Artist".to_string(),
                        email: None,
                        url: None,
                        avatar_url: None,
                    }),
                    published: None,
                    updated: None,
                    url: Some("https://open.spotify.com/track/example".to_string()),
                    thumbnail_url: None,
                    is_read: false,
                    is_saved: true,
                    tags: vec![],
                    metadata: HashMap::new(),
                },
            ]
        } else {
            // Generic items for other streams
            vec![
                Item {
                    id: ItemId::new("generic", "item-001"),
                    stream_id: StreamId(stream_id.to_string()),
                    title: "Example Item".to_string(),
                    content: ItemContent::Generic {
                        body: Some("This is a generic item from the daemon API.".to_string()),
                    },
                    author: None,
                    published: Some(Utc::now()),
                    updated: None,
                    url: None,
                    thumbnail_url: None,
                    is_read: false,
                    is_saved: false,
                    tags: vec![],
                    metadata: HashMap::new(),
                },
            ]
        }
    }
}

#[jsonrpsee::core::async_trait]
impl ScryforgeApiServer for ApiImpl {
    async fn list_streams(&self) -> RpcResult<Vec<Stream>> {
        Ok(Self::generate_dummy_streams())
    }

    async fn list_items(&self, stream_id: String) -> RpcResult<Vec<Item>> {
        Ok(Self::generate_dummy_items(&stream_id))
    }
}
