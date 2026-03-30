//! Example provider implementation demonstrating TokenFetcher usage.
//!
//! This example shows how a provider can use the TokenFetcher trait to
//! request authentication tokens from the Sigilforge daemon.

use scryforge_sigilforge_client::{MockTokenFetcher, SigilforgeClient, TokenFetcher};
use std::collections::HashMap;
use std::sync::Arc;

/// Example provider that requires OAuth authentication.
struct SpotifyProvider {
    service_name: &'static str,
    account_name: String,
    token_fetcher: Arc<dyn TokenFetcher>,
}

impl SpotifyProvider {
    /// Create a new provider with the given token fetcher.
    pub fn new(account_name: String, token_fetcher: Arc<dyn TokenFetcher>) -> Self {
        Self {
            service_name: "spotify",
            account_name,
            token_fetcher,
        }
    }

    /// Simulate fetching playlists from Spotify API.
    pub async fn fetch_playlists(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Request a fresh OAuth token
        let token = self
            .token_fetcher
            .fetch_token(self.service_name, &self.account_name)
            .await?;

        println!("Got token for {}: {}", self.account_name, token);

        // In a real implementation, use the token to make API calls
        // For this example, just return mock data
        Ok(vec![
            "Discover Weekly".to_string(),
            "Release Radar".to_string(),
            "Liked Songs".to_string(),
        ])
    }

    /// Simulate playing a track.
    pub async fn play_track(&self, track_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let token = self
            .token_fetcher
            .fetch_token(self.service_name, &self.account_name)
            .await?;

        println!("Playing track {} with token: {}", track_id, token);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Sigilforge Client Provider Example ===\n");

    // Example 1: Using a real Sigilforge client
    println!("Example 1: Real Sigilforge client (will fail if daemon not running)");
    let real_client = SigilforgeClient::with_default_path();

    if real_client.is_available() {
        println!("  Sigilforge daemon is available!");
        let provider = SpotifyProvider::new("personal".to_string(), Arc::new(real_client));

        match provider.fetch_playlists().await {
            Ok(playlists) => {
                println!("  Playlists: {:?}", playlists);
            }
            Err(e) => {
                println!("  Error fetching playlists: {}", e);
            }
        }
    } else {
        println!("  Sigilforge daemon is NOT available (socket not found)");
        println!("  This is expected if the daemon is not running.\n");
    }

    // Example 2: Using a mock token fetcher for testing
    println!("\nExample 2: Mock token fetcher (for testing)");
    let mut tokens = HashMap::new();
    tokens.insert(
        ("spotify".to_string(), "personal".to_string()),
        "mock_access_token_abc123".to_string(),
    );
    tokens.insert(
        ("spotify".to_string(), "work".to_string()),
        "mock_access_token_xyz789".to_string(),
    );

    let mock_fetcher = MockTokenFetcher::new(tokens);

    let provider1 = SpotifyProvider::new("personal".to_string(), Arc::new(mock_fetcher.clone()));
    let playlists = provider1.fetch_playlists().await?;
    println!("  Personal playlists: {:?}", playlists);

    let provider2 = SpotifyProvider::new("work".to_string(), Arc::new(mock_fetcher.clone()));
    provider2.play_track("track_12345").await?;

    // Example 3: Using builder pattern with MockTokenFetcher
    println!("\nExample 3: Mock token fetcher with builder pattern");
    let builder_fetcher = MockTokenFetcher::empty()
        .with_token(
            "spotify".to_string(),
            "test".to_string(),
            "builder_token_123".to_string(),
        )
        .with_token(
            "github".to_string(),
            "test".to_string(),
            "builder_token_456".to_string(),
        );

    let provider3 = SpotifyProvider::new("test".to_string(), Arc::new(builder_fetcher));
    let playlists = provider3.fetch_playlists().await?;
    println!("  Test playlists: {:?}", playlists);

    // Example 4: Handling token not found
    println!("\nExample 4: Handling missing tokens");
    let empty_fetcher = MockTokenFetcher::empty();
    let provider4 = SpotifyProvider::new("missing".to_string(), Arc::new(empty_fetcher));

    match provider4.fetch_playlists().await {
        Ok(_) => println!("  Unexpected success!"),
        Err(e) => println!("  Expected error: {}", e),
    }

    println!("\n=== Example Complete ===");

    Ok(())
}
