//! Local registry cache management

use super::manifest::RegistryManifest;
use super::types::{PluginEntry, PluginFilter, SortOrder};
use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Local cache for registry data
pub struct RegistryCache {
    /// Cache directory path
    cache_dir: PathBuf,
    /// In-memory manifest
    manifest: RegistryManifest,
}

impl RegistryCache {
    /// Create new cache instance
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        // Ensure cache directory exists
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)?;
        }

        // Load existing manifest or create new one
        let manifest_path = cache_dir.join("manifest.json");
        let manifest = if manifest_path.exists() {
            let content = fs::read_to_string(&manifest_path)?;
            RegistryManifest::from_json(&content)?
        } else {
            RegistryManifest::new()
        };

        Ok(Self {
            cache_dir,
            manifest,
        })
    }

    /// Update cached manifest
    pub fn update_manifest(&mut self, manifest: RegistryManifest) -> Result<()> {
        self.manifest = manifest;
        self.save_manifest()?;
        Ok(())
    }

    /// Save manifest to disk
    fn save_manifest(&self) -> Result<()> {
        let manifest_path = self.cache_dir.join("manifest.json");
        let json = self.manifest.to_json()?;
        fs::write(manifest_path, json)?;
        Ok(())
    }

    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Result<Option<PluginEntry>> {
        Ok(self.manifest.get_plugin(name).cloned())
    }

    /// Search plugins with filter
    pub fn search(&self, filter: &PluginFilter) -> Result<Vec<PluginEntry>> {
        let mut results: Vec<PluginEntry> = self.manifest.all_plugins().into_iter().cloned().collect();

        // Apply query filter
        if let Some(query) = &filter.query {
            let query_lower = query.to_lowercase();
            results.retain(|p| {
                p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.author.to_lowercase().contains(&query_lower)
            });
        }

        // Apply tag filter
        if let Some(tag) = &filter.tag {
            results.retain(|p| p.tags.iter().any(|t| t == tag));
        }

        // Apply author filter
        if let Some(author) = &filter.author {
            results.retain(|p| p.author == *author);
        }

        // Apply rating filter
        if let Some(min_rating) = filter.min_rating {
            results.retain(|p| p.stats.rating >= min_rating);
        }

        // Sort results
        match filter.sort {
            SortOrder::Popular => {
                results.sort_by(|a, b| b.stats.downloads.cmp(&a.stats.downloads));
            }
            SortOrder::Rating => {
                results.sort_by(|a, b| {
                    b.stats
                        .rating
                        .partial_cmp(&a.stats.rating)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
            }
            SortOrder::Recent => {
                results.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            }
            SortOrder::Name => {
                results.sort_by(|a, b| a.name.cmp(&b.name));
            }
        }

        // Apply limit
        if let Some(limit) = filter.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Get all tags used in registry
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags = std::collections::HashSet::new();
        for plugin in self.manifest.all_plugins() {
            for tag in &plugin.tags {
                tags.insert(tag.clone());
            }
        }
        let mut tag_vec: Vec<_> = tags.into_iter().collect();
        tag_vec.sort();
        tag_vec
    }

    /// Get cache age in seconds
    pub fn cache_age(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now.saturating_sub(self.manifest.updated_at)
    }

    /// Check if cache is stale (older than 24 hours)
    pub fn is_stale(&self) -> bool {
        self.cache_age() > 86400 // 24 hours
    }

    /// Clear cache
    pub fn clear(&mut self) -> Result<()> {
        self.manifest = RegistryManifest::new();
        self.save_manifest()?;
        Ok(())
    }

    /// Get cache directory path
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get manifest reference
    pub fn manifest(&self) -> &RegistryManifest {
        &self.manifest
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::types::PluginStats;
    use tempfile::TempDir;

    fn create_test_plugin(name: &str, downloads: u64, rating: f32) -> PluginEntry {
        PluginEntry {
            name: name.to_string(),
            description: format!("Test plugin {}", name),
            readme: None,
            author: "Test Author".to_string(),
            author_email: None,
            homepage: None,
            repository: None,
            license: "MIT".to_string(),
            latest_version: "1.0.0".to_string(),
            versions: vec![],
            tags: vec!["test".to_string()],
            stats: PluginStats {
                downloads,
                downloads_recent: 0,
                rating,
                rating_count: 10,
                stars: None,
            },
            created_at: 0,
            updated_at: 0,
        }
    }

    #[test]
    fn test_cache_search_by_query() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = RegistryCache::new(temp_dir.path().to_path_buf()).unwrap();

        let mut manifest = RegistryManifest::new();
        manifest.upsert_plugin(create_test_plugin("foo", 100, 4.5));
        manifest.upsert_plugin(create_test_plugin("bar", 200, 3.5));
        cache.update_manifest(manifest).unwrap();

        let filter = PluginFilter {
            query: Some("foo".to_string()),
            ..Default::default()
        };

        let results = cache.search(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "foo");
    }

    #[test]
    fn test_cache_search_by_rating() {
        let temp_dir = TempDir::new().unwrap();
        let mut cache = RegistryCache::new(temp_dir.path().to_path_buf()).unwrap();

        let mut manifest = RegistryManifest::new();
        manifest.upsert_plugin(create_test_plugin("foo", 100, 4.5));
        manifest.upsert_plugin(create_test_plugin("bar", 200, 3.5));
        cache.update_manifest(manifest).unwrap();

        let filter = PluginFilter {
            min_rating: Some(4.0),
            ..Default::default()
        };

        let results = cache.search(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "foo");
    }

    #[test]
    fn test_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();

        // Create cache and add plugin
        {
            let mut cache = RegistryCache::new(cache_dir.clone()).unwrap();
            let mut manifest = RegistryManifest::new();
            manifest.upsert_plugin(create_test_plugin("foo", 100, 4.5));
            cache.update_manifest(manifest).unwrap();
        }

        // Reload cache and verify plugin exists
        {
            let cache = RegistryCache::new(cache_dir).unwrap();
            assert!(cache.get_plugin("foo").unwrap().is_some());
        }
    }
}
