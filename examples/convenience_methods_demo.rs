// ! Convenience Methods Demo
// !
// ! This example demonstrates the new convenience methods for both server and client
// ! that reduce boilerplate for common MCP use cases across all transport types.
// !
// ! Features demonstrated:
// ! - Server convenience methods: run_with_stdio(), run_with_http(), run_with_websocket()
// ! - Client convenience methods: connect_with_stdio(), connect_with_http(), connect_with_websocket()
// ! - Simple one-line server/client setup and lifecycle management
// !
// ! Run different examples:
// ! - STDIO: cargo run --example convenience_methods_demo --features stdio -- stdio
// ! - HTTP: cargo run --example convenience_methods_demo --features http -- http
// ! - WebSocket: cargo run --example convenience_methods_demo --features websocket -- websocket

use prism_mcp_rs::core::ResourceInfo;
use prism_mcp_rs::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;

// Example tool handler
struct CalculatorHandler;

#[async_trait]
impl ToolHandler for CalculatorHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let a = arguments.get("a").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let b = arguments.get("b").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let operation = arguments
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("add");

        let result = match operation {
            "add" => a + b,
            "subtract" => a - b,
            "multiply" => a * b,
            "divide" => {
                if b != 0.0 {
                    a / b
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };

        Ok(ToolResult {
            content: vec![ContentBlock::text(format!(
                "{a} {operation} {b} = {result}"
            ))],
            is_error: Some(false),
            structured_content: None,
            meta: None,
        })
    }
}

// Example resource handler
struct StatusResourceHandler;

#[async_trait]
impl ResourceHandler for StatusResourceHandler {
    async fn read(
        &self,
        _uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        let status = json!({
            "status": "healthy",
            "uptime": "5 minutes",
            "version": "1.0.0"
        });

        Ok(vec![ResourceContents::Text {
            uri: "server://status".to_string(),
            mime_type: Some("application/json".to_string()),
            text: serde_json::to_string_pretty(&status).unwrap(),
            meta: None,
        }])
    }

    async fn list(&self) -> McpResult<Vec<ResourceInfo>> {
        Ok(vec![ResourceInfo {
            uri: "server://status".to_string(),
            name: "Server Status".to_string(),
            description: Some("Current server status and health information".to_string()),
            mime_type: Some("application/json".to_string()),
            annotations: None,
            size: None,
            title: None,
            meta: None,
        }])
    }
}

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    #[cfg(feature = "tracing-subscriber")]
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let transport_type = args.get(1).map(|s| s.as_str()).unwrap_or("stdio");

    match transport_type {
        "stdio" => demo_stdio_convenience().await,
        "http" => demo_http_convenience().await,
        "websocket" => demo_websocket_convenience().await,
        "client-stdio" => demo_client_stdio_convenience().await,
        "client-http" => demo_client_http_convenience().await,
        "client-websocket" => demo_client_websocket_convenience().await,
        _ => {
            println!(
                "Usage: cargo run --example convenience_methods_demo --features <transport> -- <transport_type>"
            );
            println!(
                "Available transport types: stdio, http, websocket, client-stdio, client-http, client-websocket"
            );
            Ok(())
        }
    }
}

// ============================================================================
// Server Convenience Method Demos
// ============================================================================

/// Demonstrate STDIO server convenience method
#[cfg(feature = "stdio")]
async fn demo_stdio_convenience() -> McpResult<()> {
    println!("# STDIO Server Convenience Method Demo");
    println!("=======================================\n");

    let mut server = McpServer::new("convenience-stdio-server".to_string(), "1.0.0".to_string());
    setup_server_tools_and_resources(&mut server).await?;

    println!("[x] Server configured with tools and resources");
    println!("📡 Starting server with STDIO transport (one line!)...");
    println!("Note: Press Ctrl+C to stop the server\n");

    // ## ONE LINE to start server with STDIO transport and handle shutdown!
    server.run_with_stdio().await
}

#[cfg(not(feature = "stdio"))]
async fn demo_stdio_convenience() -> McpResult<()> {
    println!("[!] STDIO feature not enabled. Run with --features stdio");
    Ok(())
}

/// Demonstrate HTTP server convenience method
#[cfg(feature = "http")]
async fn demo_http_convenience() -> McpResult<()> {
    println!("🌐 HTTP Server Convenience Method Demo");
    println!("======================================\n");

    let mut server = McpServer::new("convenience-http-server".to_string(), "1.0.0".to_string());
    setup_server_tools_and_resources(&mut server).await?;

    println!("[x] Server configured with tools and resources");
    println!("🌐 Starting HTTP server on localhost:3000 (one line!)...");
    println!("🔗 Connect to: http://localhost:3000/mcp");
    println!("📊 Health check: http://localhost:3000/health");
    println!("Note: Press Ctrl+C to stop the server\n");

    // ## ONE LINE to start server with HTTP transport and handle shutdown!
    server.run_with_http("127.0.0.1:3000").await
}

#[cfg(not(feature = "http"))]
async fn demo_http_convenience() -> McpResult<()> {
    println!("[!] HTTP feature not enabled. Run with --features http");
    Ok(())
}

/// Demonstrate WebSocket server convenience method
#[cfg(feature = "websocket")]
async fn demo_websocket_convenience() -> McpResult<()> {
    println!("- WebSocket Server Convenience Method Demo");
    println!("============================================\n");

    let mut server = McpServer::new("convenience-ws-server".to_string(), "1.0.0".to_string());
    setup_server_tools_and_resources(&mut server).await?;

    println!("[x] Server configured with tools and resources");
    println!("- Starting WebSocket server on localhost:8080 (one line!)...");
    println!("🔗 Connect to: ws://localhost:8080");
    println!("Note: Press Ctrl+C to stop the server\n");

    // ## ONE LINE to start server with WebSocket transport and handle shutdown!
    server.run_with_websocket("127.0.0.1:8080").await
}

#[cfg(not(feature = "websocket"))]
async fn demo_websocket_convenience() -> McpResult<()> {
    println!("[!] WebSocket feature not enabled. Run with --features websocket");
    Ok(())
}

// ============================================================================
// Client Convenience Method Demos
// ============================================================================

/// Demonstrate STDIO client convenience methods
#[cfg(feature = "stdio")]
async fn demo_client_stdio_convenience() -> McpResult<()> {
    println!("📱 STDIO Client Convenience Method Demo");
    println!("=======================================\n");

    println!("- Connecting to STDIO server (would connect to actual server process)...");
    println!("Note: In real usage, you would specify the actual server command\n");

    // Example 1: Simple connection (most common case)
    println!(" Example 1: Simple connection convenience method");
    println!("   client.connect_with_stdio_simple(\"my-mcp-server\").await?;");

    // Example 2: Connection with arguments
    println!(" Example 2: Connection with arguments convenience method");
    println!(
        "   client.connect_with_stdio(\"my-mcp-server\", vec![\"--verbose\", \"--port\", \"3000\"]).await?;"
    );

    // Example 3: Interactive session (full lifecycle management)
    println!(" Example 3: Interactive session convenience method");
    println!("   client.run_with_stdio(\"my-mcp-server\", vec![], |client| async move {{");
    println!("       // Your client operations here");
    println!("       let tools = client.list_tools(None).await?;");
    println!("       println!(\"Available tools: {{:?}}\", tools);");
    println!("       Ok(())");
    println!("   }}).await?;");

    println!("\n[x] STDIO client convenience methods demonstrated!");
    Ok(())
}

#[cfg(not(feature = "stdio"))]
async fn demo_client_stdio_convenience() -> McpResult<()> {
    println!("[!] STDIO feature not enabled. Run with --features stdio");
    Ok(())
}

/// Demonstrate HTTP client convenience methods
#[cfg(feature = "http")]
async fn demo_client_http_convenience() -> McpResult<()> {
    println!("🌐 HTTP Client Convenience Method Demo");
    println!("======================================\n");

    println!("- Demonstrating HTTP client convenience methods...");
    println!("Note: These would connect to actual HTTP servers\n");

    // Example 1: Basic HTTP connection
    println!(" Example 1: Basic HTTP connection convenience method");
    println!("   client.connect_with_http(\"http://localhost:3000\", None).await?;");

    // Example 2: HTTP with Server-Sent Events
    println!(" Example 2: HTTP with Server-Sent Events convenience method");
    println!(
        "   client.connect_with_http(\"http://localhost:3000\", Some(\"http://localhost:3000/events\")).await?;"
    );

    println!("\n[x] HTTP client convenience methods demonstrated!");
    Ok(())
}

#[cfg(not(feature = "http"))]
async fn demo_client_http_convenience() -> McpResult<()> {
    println!("[!] HTTP feature not enabled. Run with --features http");
    Ok(())
}

/// Demonstrate WebSocket client convenience methods
#[cfg(feature = "websocket")]
async fn demo_client_websocket_convenience() -> McpResult<()> {
    println!("- WebSocket Client Convenience Method Demo");
    println!("============================================\n");

    println!("- Demonstrating WebSocket client convenience methods...");
    println!("Note: These would connect to actual WebSocket servers\n");

    // Example: WebSocket connection
    println!(" Example: WebSocket connection convenience method");
    println!("   client.connect_with_websocket(\"ws://localhost:8080\").await?;");

    println!("\n[x] WebSocket client convenience methods demonstrated!");
    Ok(())
}

#[cfg(not(feature = "websocket"))]
async fn demo_client_websocket_convenience() -> McpResult<()> {
    println!("[!] WebSocket feature not enabled. Run with --features websocket");
    Ok(())
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Set up common tools and resources for server demos
async fn setup_server_tools_and_resources(server: &mut McpServer) -> McpResult<()> {
    // Add calculator tool
    server
        .add_tool(
            "calculator".to_string(),
            Some("Perform basic arithmetic operations".to_string()),
            json!({
                "type": "object",
                "properties": {
                    "a": {"type": "number", "description": "First number"},
                    "b": {"type": "number", "description": "Second number"},
                    "operation": {
                        "type": "string",
                        "enum": ["add", "subtract", "multiply", "divide"],
                        "description": "Operation to perform"
                    }
                },
                "required": ["a", "b", "operation"]
            }),
            CalculatorHandler,
        )
        .await?;

    // Add status resource
    server
        .add_resource(
            "Server Status".to_string(),
            "server://status".to_string(),
            StatusResourceHandler,
        )
        .await?;

    Ok(())
}
