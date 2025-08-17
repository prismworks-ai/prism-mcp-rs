//! Error Recovery and Resilience Example
//!
//! This example demonstrates error handling, retry logic, and graceful degradation.

use prism_mcp_rs::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure retry policy
    let retry_policy = RetryPolicy::exponential()
        .with_max_attempts(3)
        .with_initial_delay(Duration::from_millis(100))
        .with_max_delay(Duration::from_secs(5));

    // Create client with retry configuration
    let client = McpClient::new(
        ClientConfig::builder()
            .name("resilient-client")
            .version("1.0.0")
            .with_retry_policy(retry_policy.clone())
            .with_timeout(Duration::from_secs(10))
            .build()?,
    );

    // Attempt connection with retries
    let connection_result = retry_with_backoff(
        || client.connect_with_http("http://localhost:8080"),
        &retry_policy,
    )
    .await;

    match connection_result {
        Ok(_) => {
            println!("✅ Connected successfully");
            run_with_error_handling(&client).await?;
        }
        Err(e) => {
            println!("❌ Connection failed after retries: {}", e);

            // Fallback to stdio connection
            println!("🔄 Attempting fallback to stdio connection...");
            client.connect_with_stdio("./fallback-server").await?;
            run_with_error_handling(&client).await?;
        }
    }

    Ok(())
}

async fn run_with_error_handling(client: &McpClient) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with error handling
    match client.initialize().await {
        Ok(result) => {
            println!("Initialized: {:?}", result.server_info);
        }
        Err(McpError::Timeout(_)) => {
            println!("⚠️ Initialization timed out, proceeding with defaults");
        }
        Err(e) => {
            return Err(e.into());
        }
    }

    // Batch operations with partial failure handling
    let operations = vec![
        ("tool1", json!({ "param": "value1" })),
        ("tool2", json!({ "param": "value2" })),
        ("tool3", json!({ "param": "value3" })),
    ];

    let results = execute_batch_with_recovery(client, operations).await;

    // Process results
    for (i, result) in results.iter().enumerate() {
        match result {
            Ok(value) => {
                println!("Operation {} succeeded: {}", i + 1, value);
            }
            Err(e) => {
                println!("Operation {} failed: {}, continuing...", i + 1, e);
            }
        }
    }

    // Graceful shutdown with cleanup
    cleanup_and_shutdown(client).await?;

    Ok(())
}

async fn retry_with_backoff<F, Fut, T, E>(mut operation: F, policy: &RetryPolicy) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = policy.initial_delay;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= policy.max_attempts => {
                return Err(e);
            }
            Err(e) => {
                println!(
                    "Attempt {} failed: {}, retrying in {:?}...",
                    attempt, e, delay
                );
                sleep(delay).await;

                // Exponential backoff
                delay = std::cmp::min(delay * 2, policy.max_delay);
            }
        }
    }
}

async fn execute_batch_with_recovery(
    client: &McpClient,
    operations: Vec<(&str, serde_json::Value)>,
) -> Vec<Result<serde_json::Value, String>> {
    let mut results = Vec::new();

    for (tool_name, params) in operations {
        let result =
            tokio::time::timeout(Duration::from_secs(5), client.call_tool(tool_name, params)).await;

        match result {
            Ok(Ok(value)) => results.push(Ok(value)),
            Ok(Err(e)) => {
                // Try to recover from specific errors
                if is_recoverable(&e) {
                    println!(
                        "Recoverable error for {}: {}, attempting recovery...",
                        tool_name, e
                    );
                    // Implement recovery logic here
                    results.push(Err(format!("Recovered from: {}", e)));
                } else {
                    results.push(Err(e.to_string()));
                }
            }
            Err(_) => {
                results.push(Err("Operation timed out".to_string()));
            }
        }
    }

    results
}

fn is_recoverable(error: &McpError) -> bool {
    matches!(
        error,
        McpError::Transport(_) | McpError::Timeout(_) | McpError::ToolError { .. }
    )
}

async fn cleanup_and_shutdown(client: &McpClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧹 Performing cleanup...");

    // Send any pending notifications
    client
        .send_notification("cleanup.started", json!({}))
        .await?;

    // Close gracefully
    match tokio::time::timeout(Duration::from_secs(5), client.close()).await {
        Ok(Ok(_)) => println!("✅ Shutdown complete"),
        Ok(Err(e)) => println!("⚠️ Shutdown error: {}", e),
        Err(_) => println!("⚠️ Shutdown timed out, forcing close"),
    }

    Ok(())
}

#[derive(Clone)]
struct RetryPolicy {
    max_attempts: usize,
    initial_delay: Duration,
    max_delay: Duration,
}

impl RetryPolicy {
    fn exponential() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
        }
    }

    fn with_max_attempts(mut self, attempts: usize) -> Self {
        self.max_attempts = attempts;
        self
    }

    fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }
}
