//! # provider-dummy
//!
//! A dummy provider implementation for testing and development.
//!
//! This provider returns static fixture data and does not connect to any real services.
//! It implements the `Provider` and `HasFeeds` traits to demonstrate the provider pattern
//! and to facilitate testing of the daemon and TUI components.

use async_trait::async_trait;
use chrono::Utc;
use fusabi_streams_core::prelude::*;

/// A dummy provider that returns static test data.
pub struct DummyProvider;

impl DummyProvider {
    /// Create a new dummy provider instance.
    pub fn new() -> Self {
        Self
    }

    /// Generate static dummy feeds.
    fn dummy_feeds() -> Vec<Feed> {
        vec![
            Feed {
                id: FeedId("dummy:inbox".to_string()),
                name: "Dummy Inbox".to_string(),
                description: Some("A simulated inbox feed with test items".to_string()),
                icon: Some("📥".to_string()),
                unread_count: Some(3),
                total_count: Some(10),
            },
            Feed {
                id: FeedId("dummy:updates".to_string()),
                name: "Dummy Updates".to_string(),
                description: Some("Simulated notification feed".to_string()),
                icon: Some("🔔".to_string()),
                unread_count: Some(5),
                total_count: Some(20),
            },
            Feed {
                id: FeedId("dummy:archive".to_string()),
                name: "Dummy Archive".to_string(),
                description: Some("Archived test items".to_string()),
                icon: Some("📦".to_string()),
                unread_count: Some(0),
                total_count: Some(100),
            },
        ]
    }

    /// Generate static dummy items for a given feed.
    fn dummy_items(feed_id: &FeedId) -> Vec<Item> {
        let stream_id = StreamId::new("dummy", "feed", feed_id.0.as_str());

        match feed_id.0.as_str() {
            "dummy:inbox" => vec![
                Item {
                    id: ItemId::new("dummy", "item-1"),
                    stream_id: stream_id.clone(),
                    title: "Welcome to Scryforge".to_string(),
                    content: ItemContent::Text(
                        "This is a dummy item from the test provider. \
                         Scryforge is working correctly!"
                            .to_string(),
                    ),
                    author: Some(Author {
                        name: "Dummy Provider".to_string(),
                        email: Some("dummy@scryforge.test".to_string()),
                        url: None,
                        avatar_url: None,
                    }),
                    published: Some(Utc::now() - chrono::Duration::hours(2)),
                    updated: None,
                    url: Some("https://example.com/item-1".to_string()),
                    thumbnail_url: None,
                    is_read: false,
                    is_saved: false,
                    tags: vec!["test".to_string(), "welcome".to_string()],
                    metadata: Default::default(),
                },
                Item {
                    id: ItemId::new("dummy", "item-2"),
                    stream_id: stream_id.clone(),
                    title: "Test Article with Markdown".to_string(),
                    content: ItemContent::Markdown(
                        "# Dummy Article\n\n\
                         This is a **test article** with _markdown_ formatting.\n\n\
                         - Bullet point 1\n\
                         - Bullet point 2\n\n\
                         [Link to example](https://example.com)"
                            .to_string(),
                    ),
                    author: Some(Author {
                        name: "Test Author".to_string(),
                        email: None,
                        url: Some("https://example.com/author".to_string()),
                        avatar_url: Some("https://example.com/avatar.jpg".to_string()),
                    }),
                    published: Some(Utc::now() - chrono::Duration::hours(5)),
                    updated: Some(Utc::now() - chrono::Duration::hours(3)),
                    url: Some("https://example.com/item-2".to_string()),
                    thumbnail_url: Some("https://example.com/thumb-2.jpg".to_string()),
                    is_read: false,
                    is_saved: true,
                    tags: vec!["test".to_string(), "article".to_string()],
                    metadata: Default::default(),
                },
                Item {
                    id: ItemId::new("dummy", "item-3"),
                    stream_id: stream_id.clone(),
                    title: "Read Item Example".to_string(),
                    content: ItemContent::Text("This item has been marked as read.".to_string()),
                    author: None,
                    published: Some(Utc::now() - chrono::Duration::days(1)),
                    updated: None,
                    url: None,
                    thumbnail_url: None,
                    is_read: true,
                    is_saved: false,
                    tags: vec![],
                    metadata: Default::default(),
                },
            ],
            "dummy:updates" => vec![
                Item {
                    id: ItemId::new("dummy", "update-1"),
                    stream_id: stream_id.clone(),
                    title: "System Update Available".to_string(),
                    content: ItemContent::Text(
                        "A new version of the system is available.".to_string(),
                    ),
                    author: None,
                    published: Some(Utc::now() - chrono::Duration::minutes(30)),
                    updated: None,
                    url: None,
                    thumbnail_url: None,
                    is_read: false,
                    is_saved: false,
                    tags: vec!["notification".to_string()],
                    metadata: Default::default(),
                },
                Item {
                    id: ItemId::new("dummy", "update-2"),
                    stream_id: stream_id.clone(),
                    title: "New Feature Released".to_string(),
                    content: ItemContent::Markdown(
                        "## New Feature\n\nCheck out our latest feature update!".to_string(),
                    ),
                    author: Some(Author {
                        name: "Product Team".to_string(),
                        email: None,
                        url: None,
                        avatar_url: None,
                    }),
                    published: Some(Utc::now() - chrono::Duration::hours(1)),
                    updated: None,
                    url: Some("https://example.com/updates/feature-1".to_string()),
                    thumbnail_url: None,
                    is_read: false,
                    is_saved: false,
                    tags: vec!["feature".to_string(), "announcement".to_string()],
                    metadata: Default::default(),
                },
            ],
            "dummy:archive" => vec![Item {
                id: ItemId::new("dummy", "archive-1"),
                stream_id: stream_id.clone(),
                title: "Archived Item 1".to_string(),
                content: ItemContent::Text("Old archived content.".to_string()),
                author: None,
                published: Some(Utc::now() - chrono::Duration::days(30)),
                updated: None,
                url: None,
                thumbnail_url: None,
                is_read: true,
                is_saved: false,
                tags: vec!["archived".to_string()],
                metadata: Default::default(),
            }],
            _ => vec![],
        }
    }
}

impl Default for DummyProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for DummyProvider {
    fn id(&self) -> &'static str {
        "dummy"
    }

    fn name(&self) -> &'static str {
        "Dummy Provider"
    }

    async fn health_check(&self) -> Result<ProviderHealth> {
        Ok(ProviderHealth {
            is_healthy: true,
            message: Some("Dummy provider is always healthy".to_string()),
            last_sync: Some(Utc::now()),
            error_count: 0,
        })
    }

    async fn sync(&self) -> Result<SyncResult> {
        // Simulate a successful sync
        Ok(SyncResult {
            success: true,
            items_added: 0,
            items_updated: 0,
            items_removed: 0,
            errors: vec![],
            duration_ms: 10,
        })
    }

    fn capabilities(&self) -> ProviderCapabilities {
        ProviderCapabilities {
            has_feeds: true,
            has_collections: false,
            has_saved_items: false,
            has_communities: false,
        }
    }

    async fn available_actions(&self, _item: &Item) -> Result<Vec<Action>> {
        Ok(vec![
            Action {
                id: "open".to_string(),
                name: "Open".to_string(),
                description: "Open item".to_string(),
                kind: ActionKind::Open,
                keyboard_shortcut: Some("o".to_string()),
            },
            Action {
                id: "preview".to_string(),
                name: "Preview".to_string(),
                description: "Show preview".to_string(),
                kind: ActionKind::Preview,
                keyboard_shortcut: Some("p".to_string()),
            },
            Action {
                id: "mark_read".to_string(),
                name: "Mark as Read".to_string(),
                description: "Mark item as read".to_string(),
                kind: ActionKind::MarkRead,
                keyboard_shortcut: Some("r".to_string()),
            },
        ])
    }

    async fn execute_action(&self, _item: &Item, action: &Action) -> Result<ActionResult> {
        Ok(ActionResult {
            success: true,
            message: Some(format!("Executed action: {}", action.name)),
            data: None,
        })
    }
}

#[async_trait]
impl HasFeeds for DummyProvider {
    async fn list_feeds(&self) -> Result<Vec<Feed>> {
        Ok(Self::dummy_feeds())
    }

    async fn get_feed_items(&self, feed_id: &FeedId, options: FeedOptions) -> Result<Vec<Item>> {
        let mut items = Self::dummy_items(feed_id);

        // Apply filtering based on options
        if !options.include_read {
            items.retain(|item| !item.is_read);
        }

        // Apply since filter
        if let Some(since) = options.since {
            items.retain(|item| item.published.is_some_and(|pub_date| pub_date > since));
        }

        // Apply offset and limit
        let offset = options.offset.unwrap_or(0) as usize;
        let limit = options.limit.map(|l| l as usize);

        let items = items.into_iter().skip(offset);
        let items = if let Some(limit) = limit {
            items.take(limit).collect()
        } else {
            items.collect()
        };

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dummy_provider_basics() {
        let provider = DummyProvider::new();

        assert_eq!(provider.id(), "dummy");
        assert_eq!(provider.name(), "Dummy Provider");

        let caps = provider.capabilities();
        assert!(caps.has_feeds);
        assert!(!caps.has_collections);
    }

    #[tokio::test]
    async fn test_health_check() {
        let provider = DummyProvider::new();
        let health = provider.health_check().await.unwrap();

        assert!(health.is_healthy);
        assert_eq!(health.error_count, 0);
    }

    #[tokio::test]
    async fn test_list_feeds() {
        let provider = DummyProvider::new();
        let feeds = provider.list_feeds().await.unwrap();

        assert_eq!(feeds.len(), 3);
        assert_eq!(feeds[0].id.0, "dummy:inbox");
        assert_eq!(feeds[1].id.0, "dummy:updates");
        assert_eq!(feeds[2].id.0, "dummy:archive");
    }

    #[tokio::test]
    async fn test_get_feed_items() {
        let provider = DummyProvider::new();
        let feed_id = FeedId("dummy:inbox".to_string());
        let options = FeedOptions {
            include_read: true, // Include all items
            ..Default::default()
        };

        let items = provider.get_feed_items(&feed_id, options).await.unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].title, "Welcome to Scryforge");
    }

    #[tokio::test]
    async fn test_get_feed_items_exclude_read() {
        let provider = DummyProvider::new();
        let feed_id = FeedId("dummy:inbox".to_string());
        let options = FeedOptions {
            include_read: false,
            ..Default::default()
        };

        let items = provider.get_feed_items(&feed_id, options).await.unwrap();
        assert_eq!(items.len(), 2); // One item is marked as read
        assert!(!items.iter().any(|item| item.is_read));
    }

    #[tokio::test]
    async fn test_get_feed_items_with_limit() {
        let provider = DummyProvider::new();
        let feed_id = FeedId("dummy:inbox".to_string());
        let options = FeedOptions {
            limit: Some(2),
            ..Default::default()
        };

        let items = provider.get_feed_items(&feed_id, options).await.unwrap();
        assert_eq!(items.len(), 2);
    }

    #[tokio::test]
    async fn test_available_actions() {
        let provider = DummyProvider::new();
        let item = Item {
            id: ItemId::new("dummy", "test"),
            stream_id: StreamId::new("dummy", "feed", "test"),
            title: "Test".to_string(),
            content: ItemContent::Text("Test".to_string()),
            author: None,
            published: None,
            updated: None,
            url: None,
            thumbnail_url: None,
            is_read: false,
            is_saved: false,
            tags: vec![],
            metadata: Default::default(),
        };

        let actions = provider.available_actions(&item).await.unwrap();
        assert_eq!(actions.len(), 3);
        assert_eq!(actions[0].kind, ActionKind::Open);
        assert_eq!(actions[1].kind, ActionKind::Preview);
        assert_eq!(actions[2].kind, ActionKind::MarkRead);
    }

    #[tokio::test]
    async fn test_execute_action() {
        let provider = DummyProvider::new();
        let item = Item {
            id: ItemId::new("dummy", "test"),
            stream_id: StreamId::new("dummy", "feed", "test"),
            title: "Test".to_string(),
            content: ItemContent::Text("Test".to_string()),
            author: None,
            published: None,
            updated: None,
            url: None,
            thumbnail_url: None,
            is_read: false,
            is_saved: false,
            tags: vec![],
            metadata: Default::default(),
        };
        let action = Action {
            id: "open".to_string(),
            name: "Open".to_string(),
            description: "Open item".to_string(),
            kind: ActionKind::Open,
            keyboard_shortcut: Some("o".to_string()),
        };

        let result = provider.execute_action(&item, &action).await.unwrap();
        assert!(result.success);
    }
}
