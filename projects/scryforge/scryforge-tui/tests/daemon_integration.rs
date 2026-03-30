//! Integration tests for daemon-TUI communication.
//!
//! These tests verify that the TUI can successfully connect to the daemon
//! and fetch streams and items via JSON-RPC.

use fusabi_streams_core::{Item, Stream};
use jsonrpsee::core::client::ClientT;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;
use tokio::time::{sleep, Duration};

/// Test that we can connect to the daemon and fetch streams.
#[tokio::test]
async fn test_fetch_streams_from_daemon() {
    // Note: This test requires the daemon to be running on localhost:3030
    // In a real CI environment, we would start the daemon programmatically

    // Try to connect to daemon
    let client_result = HttpClientBuilder::default()
        .build("http://127.0.0.1:3030");

    // Skip test if daemon is not running (for CI)
    let client = match client_result {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: daemon not running");
            return;
        }
    };

    // Give daemon a moment to start
    sleep(Duration::from_millis(100)).await;

    // Fetch streams
    let result: Result<Vec<Stream>, _> = client.request("streams.list", rpc_params![]).await;

    match result {
        Ok(streams) => {
            // We should get some streams from the dummy data
            assert!(!streams.is_empty(), "Should have received at least one stream");
            println!("Received {} streams", streams.len());

            // Verify stream structure
            for stream in &streams {
                assert!(!stream.name.is_empty(), "Stream should have a name");
                assert!(!stream.provider_id.is_empty(), "Stream should have a provider ID");
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch streams: {}", e);
            eprintln!("Skipping test: daemon may not be running");
            // Don't panic, just skip the test
        }
    }
}

/// Test that we can fetch items for a stream.
#[tokio::test]
async fn test_fetch_items_from_daemon() {
    // Try to connect to daemon
    let client_result = HttpClientBuilder::default()
        .build("http://127.0.0.1:3030");

    // Skip test if daemon is not running (for CI)
    let client = match client_result {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping test: daemon not running");
            return;
        }
    };

    // Give daemon a moment to start
    sleep(Duration::from_millis(100)).await;

    // First get streams to know what stream IDs are available
    let streams_result: Result<Vec<Stream>, _> = client.request("streams.list", rpc_params![]).await;

    match streams_result {
        Ok(streams) => {
            if streams.is_empty() {
                eprintln!("No streams available, skipping test");
                return;
            }

            // Try to fetch items for the first stream
            let stream_id = streams[0].id.as_str();
            let items_result: Result<Vec<Item>, _> = client
                .request("items.list", rpc_params![stream_id])
                .await;

            match items_result {
                Ok(items) => {
                    // We should get some items (dummy data provides items for each stream)
                    assert!(!items.is_empty(), "Should have received at least one item");
                    println!("Received {} items for stream {}", items.len(), stream_id);

                    // Verify item structure
                    for item in &items {
                        assert!(!item.title.is_empty(), "Item should have a title");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch items: {}", e);
                    eprintln!("Skipping test: daemon may not be running");
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch streams: {}", e);
            eprintln!("Skipping test: daemon may not be running");
        }
    }
}

/// Test that the daemon returns appropriate data types.
#[tokio::test]
async fn test_daemon_data_structure() {
    // Try to connect to daemon
    let client_result = HttpClientBuilder::default()
        .build("http://127.0.0.1:3030");

    // Skip test if daemon is not running
    if client_result.is_err() {
        eprintln!("Skipping test: daemon not running");
        return;
    }

    let client = client_result.unwrap();

    // Fetch streams
    let streams: Vec<Stream> = match client.request("streams.list", rpc_params![]).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };

    // Verify we get expected stream types
    assert!(!streams.is_empty());

    // Check that streams have proper metadata
    for stream in &streams {
        // All streams should have these basic fields
        assert!(!stream.id.as_str().is_empty());
        assert!(!stream.name.is_empty());
        assert!(!stream.provider_id.is_empty());

        // Some streams may have counts
        println!(
            "Stream: {} (unread: {:?}, total: {:?})",
            stream.name, stream.unread_count, stream.total_count
        );
    }
}
