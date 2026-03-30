//! Registry type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    /// Remote registry URL
    pub registry_url: String,
    /// Local cache directory
    pub cache_dir: PathBuf,
    /// Plugin installation directory
    pub plugin_dir: PathBuf,
    /// Security configuration
    pub security: SecurityConfig,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            registry_url: "https://registry.scarab.dev".to_string(),
            cache_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from(".config"))
                .join("scarab")
                .join("registry"),
            plugin_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from(".config"))
                .join("scarab")
                .join("plugins"),
            security: SecurityConfig::default(),
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Require SHA256 checksum verification
    pub require_checksum: bool,
    /// Require GPG signature verification
    pub require_signature: bool,
    /// Trusted GPG key IDs
    pub trusted_keys: Vec<String>,
    /// Allow unsigned plugins (dangerous!)
    pub allow_unsigned: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            require_checksum: true,
            require_signature: false,
            trusted_keys: Vec::new(),
            allow_unsigned: true, // For initial development
        }
    }
}

/// Plugin entry in registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEntry {
    /// Plugin unique identifier
    pub name: String,
    /// Short description
    pub description: String,
    /// Long-form README content
    pub readme: Option<String>,
    /// Author name
    pub author: String,
    /// Author email
    pub author_email: Option<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// License (SPDX identifier)
    pub license: String,
    /// Latest version
    pub latest_version: String,
    /// All available versions
    pub versions: Vec<PluginVersion>,
    /// Plugin tags/categories
    pub tags: Vec<String>,
    /// Plugin statistics
    pub stats: PluginStats,
    /// Creation timestamp (Unix epoch)
    pub created_at: u64,
    /// Last update timestamp
    pub updated_at: u64,
}

/// Version-specific plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginVersion {
    /// Version string (semver)
    pub version: String,
    /// Download URL
    pub download_url: String,
    /// SHA256 checksum
    pub checksum: String,
    /// GPG signature (detached)
    pub signature: Option<String>,
    /// Release notes
    pub changelog: Option<String>,
    /// API version compatibility
    pub api_version: String,
    /// Minimum Scarab version required
    pub min_scarab_version: String,
    /// File size in bytes
    pub size: u64,
    /// Release timestamp
    pub released_at: u64,
    /// Whether this is a prerelease version
    pub prerelease: bool,
}

/// Plugin statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PluginStats {
    /// Total downloads
    pub downloads: u64,
    /// Downloads in last 30 days
    pub downloads_recent: u64,
    /// Average rating (0.0 - 5.0)
    pub rating: f32,
    /// Total number of ratings
    pub rating_count: u32,
    /// GitHub stars (if applicable)
    pub stars: Option<u32>,
}

/// Plugin rating/review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRating {
    /// User who rated
    pub user: String,
    /// Rating value (1-5)
    pub rating: u8,
    /// Optional review text
    pub review: Option<String>,
    /// Timestamp
    pub created_at: u64,
}

/// Installed plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPlugin {
    /// Plugin name
    pub name: String,
    /// Installed version
    pub version: String,
    /// Installation path
    pub path: PathBuf,
    /// Installation timestamp
    pub installed_at: u64,
    /// Whether plugin is enabled
    pub enabled: bool,
    /// Plugin-specific configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Filter for searching plugins
#[derive(Debug, Clone, Default)]
pub struct PluginFilter {
    /// Search query (matches name, description, author)
    pub query: Option<String>,
    /// Filter by tag
    pub tag: Option<String>,
    /// Filter by author
    pub author: Option<String>,
    /// Minimum rating
    pub min_rating: Option<f32>,
    /// Sort order
    pub sort: SortOrder,
    /// Maximum results to return
    pub limit: Option<usize>,
}

/// Sort order for search results
#[derive(Debug, Clone, Copy, Default)]
pub enum SortOrder {
    /// Most downloads first
    #[default]
    Popular,
    /// Highest rated first
    Rating,
    /// Recently updated first
    Recent,
    /// Alphabetical by name
    Name,
}

/// Plugin download response
#[derive(Debug)]
pub struct PluginDownload {
    /// Plugin file content
    pub content: Vec<u8>,
    /// Content checksum
    pub checksum: String,
    /// Optional signature
    pub signature: Option<String>,
}
