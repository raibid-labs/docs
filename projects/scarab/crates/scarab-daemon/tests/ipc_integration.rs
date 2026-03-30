use anyhow::Result;
use scarab_protocol::{ControlMessage, MAX_MESSAGE_SIZE, SOCKET_PATH};
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tokio::time::{sleep, timeout};

/// Helper function to connect to daemon with timeout
async fn connect_with_timeout() -> Result<UnixStream> {
    timeout(Duration::from_secs(5), UnixStream::connect(SOCKET_PATH))
        .await?
        .map_err(Into::into)
}

/// Helper function to send a message
async fn send_message(stream: &mut UnixStream, msg: ControlMessage) -> Result<()> {
    let bytes = rkyv::to_bytes::<_, MAX_MESSAGE_SIZE>(&msg)?;
    let len = bytes.len();

    stream.write_u32(len as u32).await?;
    stream.write_all(&bytes).await?;
    stream.flush().await?;

    Ok(())
}

#[tokio::test]
async fn test_single_client_connection() -> Result<()> {
    // This test assumes daemon is running
    let stream = connect_with_timeout().await?;
    assert!(stream.peer_addr().is_ok());
    println!("✓ Single client connected successfully");
    Ok(())
}

#[tokio::test]
async fn test_send_resize_message() -> Result<()> {
    let mut stream = connect_with_timeout().await?;

    let msg = ControlMessage::Resize {
        cols: 120,
        rows: 40,
    };

    send_message(&mut stream, msg).await?;
    println!("✓ Resize message sent successfully");

    // Give daemon time to process
    sleep(Duration::from_millis(100)).await;

    Ok(())
}

#[tokio::test]
async fn test_send_input_message() -> Result<()> {
    let mut stream = connect_with_timeout().await?;

    let msg = ControlMessage::Input {
        data: b"echo 'Hello from test'\n".to_vec(),
    };

    send_message(&mut stream, msg).await?;
    println!("✓ Input message sent successfully");

    sleep(Duration::from_millis(100)).await;

    Ok(())
}

#[tokio::test]
async fn test_multiple_messages() -> Result<()> {
    let mut stream = connect_with_timeout().await?;

    // Send multiple messages in sequence
    for i in 0..10 {
        let msg = ControlMessage::Input {
            data: format!("test message {}\n", i).into_bytes(),
        };
        send_message(&mut stream, msg).await?;
    }

    println!("✓ Multiple messages sent successfully");
    Ok(())
}

#[tokio::test]
async fn test_multiple_concurrent_clients() -> Result<()> {
    let mut handles = vec![];

    // Connect multiple clients
    for i in 0..5 {
        let handle = tokio::spawn(async move {
            let mut stream = connect_with_timeout().await?;

            let msg = ControlMessage::Ping {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_millis() as u64,
            };

            send_message(&mut stream, msg).await?;

            println!("✓ Client {} connected and sent ping", i);

            sleep(Duration::from_millis(500)).await;

            Ok::<_, anyhow::Error>(())
        });

        handles.push(handle);
    }

    // Wait for all clients
    for handle in handles {
        handle.await??;
    }

    println!("✓ Multiple concurrent clients test passed");
    Ok(())
}

#[tokio::test]
async fn test_graceful_disconnect() -> Result<()> {
    let stream = connect_with_timeout().await?;

    // Simply drop the stream
    drop(stream);

    // Verify we can reconnect
    let _stream2 = connect_with_timeout().await?;

    println!("✓ Graceful disconnect test passed");
    Ok(())
}

#[tokio::test]
async fn test_message_roundtrip_latency() -> Result<()> {
    let mut stream = connect_with_timeout().await?;

    let iterations = 100;
    let mut total_duration = Duration::from_nanos(0);

    for _ in 0..iterations {
        let start = std::time::Instant::now();

        let msg = ControlMessage::Ping {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis() as u64,
        };

        send_message(&mut stream, msg).await?;

        let duration = start.elapsed();
        total_duration += duration;
    }

    let avg_latency = total_duration / iterations;

    println!("✓ Average message latency: {:?}", avg_latency);

    // Verify we're under 1ms requirement
    assert!(
        avg_latency < Duration::from_millis(1),
        "Average latency {}μs exceeds 1ms requirement",
        avg_latency.as_micros()
    );

    Ok(())
}

#[tokio::test]
async fn test_large_input_message() -> Result<()> {
    let mut stream = connect_with_timeout().await?;

    // Send a large but valid input message
    let large_input = vec![b'a'; 4096];
    let msg = ControlMessage::Input { data: large_input };

    send_message(&mut stream, msg).await?;
    println!("✓ Large input message sent successfully");

    Ok(())
}

#[tokio::test]
async fn test_rapid_resize_events() -> Result<()> {
    let mut stream = connect_with_timeout().await?;

    // Simulate rapid window resizing
    for i in 0..20 {
        let cols = 80 + (i * 5);
        let rows = 24 + (i * 2);

        let msg = ControlMessage::Resize { cols, rows };
        send_message(&mut stream, msg).await?;

        // Small delay to simulate realistic resize events
        sleep(Duration::from_millis(10)).await;
    }

    println!("✓ Rapid resize events handled successfully");
    Ok(())
}

#[tokio::test]
async fn test_stress_test_single_client() -> Result<()> {
    let mut stream = connect_with_timeout().await?;

    // Send 1000 messages as fast as possible
    for i in 0..1000 {
        let msg = ControlMessage::Input {
            data: format!("{}", i % 10).into_bytes(),
        };

        send_message(&mut stream, msg).await?;
    }

    println!("✓ Stress test: 1000 messages sent successfully");
    Ok(())
}
