//! Simplified working examples that compile with current API

use prism_mcp_rs::prelude::*;

// Example 1: Simple tool without macro
pub async fn example_01_tool() -> McpResult<()> {
    let server = McpServer::new("tool-example".to_string(), "1.0.0".to_string());

    // Tools would be added via server.add_tool() method
    println!("Tool example server created");
    Ok(())
}

// Example 2: Resources
pub async fn example_02_resources() -> McpResult<()> {
    let server = McpServer::new("resource-example".to_string(), "1.0.0".to_string());

    // Resources would be added via server.add_resource_handler()
    println!("Resource example server created");
    Ok(())
}

// Example 3: Prompts
pub async fn example_03_prompts() -> McpResult<()> {
    let server = McpServer::new("prompt-example".to_string(), "1.0.0".to_string());

    // Prompts would be added via server.add_prompt()
    println!("Prompt example server created");
    Ok(())
}

// Example 4: Sampling (if feature enabled)
#[cfg(feature = "sampling")]
pub async fn example_04_sampling() -> McpResult<()> {
    let mut server = McpServer::new("sampling-example".to_string(), "1.0.0".to_string());
    println!("Sampling example server created");
    Ok(())
}

// Example 5: HTTP Transport
#[cfg(feature = "http")]
pub async fn example_05_http() -> McpResult<()> {
    use prism_mcp_rs::transport::HttpServerTransport;

    let server = McpServer::new("http-example".to_string(), "1.0.0".to_string());
    let transport = HttpServerTransport::new("127.0.0.1:8080");
    println!("HTTP transport created at 127.0.0.1:8080");
    Ok(())
}

// Example 6: WebSocket Transport
#[cfg(feature = "websocket")]
pub async fn example_06_websocket() -> McpResult<()> {
    use prism_mcp_rs::transport::WebSocketServerTransport;

    let server = McpServer::new("ws-example".to_string(), "1.0.0".to_string());
    let transport = WebSocketServerTransport::new("127.0.0.1:9000");
    println!("WebSocket transport created at 127.0.0.1:9000");
    Ok(())
}

// Example 7: Authentication
#[cfg(feature = "http")]
pub async fn example_07_auth() -> McpResult<()> {
    let server = McpServer::new("auth-example".to_string(), "1.0.0".to_string());
    println!("Auth example server created");
    Ok(())
}

// Example 8: Error Handling
pub async fn example_08_errors() -> McpResult<()> {
    let server = McpServer::new("error-example".to_string(), "1.0.0".to_string());

    // Demonstrate error handling
    match server.info().name.parse::<i32>() {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("Handled error: {}", e),
    }
    Ok(())
}

// Example 9: Configuration
pub async fn example_09_config() -> McpResult<()> {
    use prism_mcp_rs::server::ServerConfig;

    let config = ServerConfig {
        validate_requests: true,
        enable_logging: false,
        max_concurrent_requests: 100,
        request_timeout_ms: 30000,
    };

    let server = McpServer::with_config("config-example".to_string(), "1.0.0".to_string(), config);
    println!("Configured server created");
    Ok(())
}

// Example 10: Plugin System
#[cfg(feature = "plugin")]
pub async fn example_10_plugin() -> McpResult<()> {
    let server = McpServer::new("plugin-example".to_string(), "1.0.0".to_string());
    println!("Plugin example server created");
    Ok(())
}

// Example 11: Advanced Tools
pub async fn example_11_advanced() -> McpResult<()> {
    let server = McpServer::new("advanced-example".to_string(), "1.0.0".to_string());
    println!("Advanced example server created");
    Ok(())
}

// Example 12: Integration Patterns
pub async fn example_12_integration() -> McpResult<()> {
    let server = McpServer::new("integration-example".to_string(), "1.0.0".to_string());

    // Demonstrate async integration
    let handle = tokio::spawn(async move {
        println!("Running async task");
        42
    });

    let result = handle.await.unwrap();
    println!("Async result: {}", result);
    Ok(())
}

#[tokio::main]
async fn main() -> McpResult<()> {
    println!("Running simplified examples...");

    example_01_tool().await?;
    example_02_resources().await?;
    example_03_prompts().await?;

    #[cfg(feature = "sampling")]
    example_04_sampling().await?;

    #[cfg(feature = "http")]
    example_05_http().await?;

    #[cfg(feature = "websocket")]
    example_06_websocket().await?;

    #[cfg(feature = "http")]
    example_07_auth().await?;

    example_08_errors().await?;
    example_09_config().await?;

    #[cfg(feature = "plugin")]
    example_10_plugin().await?;

    example_11_advanced().await?;
    example_12_integration().await?;

    println!("All examples completed successfully!");
    Ok(())
}
