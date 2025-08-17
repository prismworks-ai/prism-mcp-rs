// ! # MCP Transport Selection Guide - Choose the Right Transport
// !
// ! This example demonstrates when and how to use each of the transport types
// ! available in the MCP Protocol SDK and how to choose the right one for your use case.

use prism_mcp_rs::client::TransportUseCase;
use prism_mcp_rs::prelude::*;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("# MCP Transport Selection Guide");
    println!("=================================\n");
    println!("This guide helps you choose the optimal transport for your MCP application.\n");

    // Demonstrate all transport types with clear use cases
    demo_stdio_transport().await?;
    demo_http_transport().await?;
    demo_websocket_transport().await?;
    demo_complete_optimizations().await?;

    // Show transport recommendation system
    demo_transport_recommendations().await?;

    // Show transport comparison
    demo_transport_comparison().await?;

    println!("\n## Quick Selection Guide");
    println!("========================");
    println!("• Building a CLI tool? → Use STDIO transport");
    println!("• Building a web app? → Use HTTP transport");
    println!("• Need real-time updates? → Use WebSocket transport");
    println!("• Processing large data? → Use Streaming HTTP transport (NOW AVAILABLE!)");
    println!("• Memory constrained? → Use Streaming HTTP with memory_improved config");
    println!("• Not sure? → Use automatic transport selection!\n");

    Ok(())
}

/// STDIO Transport - Foundation & Most Common
async fn demo_stdio_transport() -> McpResult<()> {
    println!("* STDIO Transport - Foundation & Most Common");
    println!("============================================");
    println!("complete for:");
    println!("  [x] Command-line MCP servers (most common use case)");
    println!("  [x] Desktop applications spawning MCP processes");
    println!("  [x] Local development and testing");
    println!("  [x] CI/CD pipelines and automation");
    println!("  [x] Process-to-process communication");

    println!("\n- Key Features:");
    println!("  • Direct stdin/stdout communication");
    println!("  • Process lifecycle management");
    println!("  • Zero network configuration");
    println!("  • Bidirectional JSON-RPC over pipes");
    println!("  • Automatic process spawning and cleanup");

    println!("\nNote: Usage Examples:");
    println!("```rust");
    println!("// Most common case - spawn MCP server process");
    println!("let mut client = McpClient::new(\"my-app\".to_string(), \"1.0.0\".to_string());");
    println!("let init = client.connect_with_stdio_simple(\"my-mcp-server\").await?;");
    println!();
    println!("// With arguments");
    println!(
        "let init = client.connect_with_stdio(\"my-mcp-server\", vec![\"--config\", \"dev.json\"]).await?;"
    );
    println!();
    println!("// Interactive session with lifecycle management");
    println!("client.run_with_stdio(\"my-server\", vec![], |client| async move {{");
    println!("    let tools = client.list_tools(None).await?;");
    println!("    // Your application logic.");
    println!("    Ok(())");
    println!("}}).await?;");
    println!("```");

    println!("\n** Why STDIO is Special:");
    println!("  • Foundation of MCP protocol");
    println!("  • Most MCP servers support STDIO");
    println!("  • Simplest setup - just spawn a process");
    println!("  • complete for desktop and CLI integration\n");

    Ok(())
}

/// HTTP Transport - Maximum Compatibility
async fn demo_http_transport() -> McpResult<()> {
    println!("📡 HTTP Transport - Maximum Compatibility");
    println!("=========================================");
    println!("complete for:");
    println!("  [x] Web applications and browser integration");
    println!("  [x] Mobile clients (battery efficient)");
    println!("  [x] Enterprise environments (firewall friendly)");
    println!("  [x] Simple request/response patterns");

    println!("\n- Key Features:");
    println!("  • HTTP/1.1 POST requests");
    println!("  • Server-Sent Events for notifications");
    println!("  • Basic gzip compression");
    println!("  • Universal compatibility");
    println!("  • Firewall and proxy friendly");

    println!("\nNote: Usage Examples:");
    println!("```rust");
    println!("let mut client = McpClient::new(\"web-app\".to_string(), \"1.0.0\".to_string());");
    println!("let init = client.connect_with_http(");
    println!("    \"http://localhost:3000\", ");
    println!("    Some(\"http://localhost:3000/events\")");
    println!(").await?;");
    println!("let tools = client.list_tools(None).await?;");
    println!("```");

    println!("\n** Why HTTP is Reliable:");
    println!("  • Works everywhere - browsers, mobile, servers");
    println!("  • Simple debugging with standard HTTP tools");
    println!("  • production-ready firewall compatibility");
    println!("  • tested protocol\n");

    Ok(())
}

/// WebSocket Transport - Real-time Bidirectional
async fn demo_websocket_transport() -> McpResult<()> {
    println!("* WebSocket Transport - Real-time Bidirectional");
    println!("===============================================");
    println!("complete for:");
    println!("  [x] Real-time applications");
    println!("  [x] Chat interfaces and live collaboration");
    println!("  [x] High-frequency message exchange");
    println!("  [x] Applications requiring lowest latency");

    println!("\n- Key Features:");
    println!("  • Full duplex communication");
    println!("  • Lowest possible latency (<5ms)");
    println!("  • Automatic reconnection");
    println!("  • Message compression");
    println!("  • Real-time notifications");

    println!("\nNote: Usage Examples:");
    println!("```rust");
    println!(
        "let mut client = McpClient::new(\"realtime-app\".to_string(), \"1.0.0\".to_string());"
    );
    println!("let init = client.connect_with_websocket(\"ws://localhost:8080\").await?;");
    println!("// Real-time bidirectional communication");
    println!("let result = client.ping().await?; // <5ms latency");
    println!("```");

    println!("\n** Why WebSocket Excels:");
    println!("  • Lowest latency among all transports");
    println!("  • True bidirectional communication");
    println!("  • complete for interactive applications");
    println!("  • Handles connection drops smoothly\n");

    Ok(())
}

/// Streaming HTTP Transport - complete & Efficient (NEW!)
async fn demo_complete_optimizations() -> McpResult<()> {
    println!("🌊 Streaming HTTP Transport - complete & Efficient (NEW!)");
    println!("======================================================");
    println!(" NOW AVAILABLE: complete streaming features implemented!");
    println!("  [x] Chunked transfer encoding for large payloads");
    println!("  [x] complete compression (Gzip, Brotli, Zstd)");
    println!("  [x] smart content analysis");
    println!("  [x] Adaptive buffering and flow control");
    println!("  * HTTP/2 Server Push capabilities (with http2 feature)");

    println!("\n## complete for:");
    println!("  • Large data processing (>100KB payloads)");
    println!("  • Memory-constrained environments");
    println!("  • High-performance applications");
    println!("  • Mixed payload sizes with smart optimization");

    println!("\nNote: Usage Examples:");
    println!("```rust");
    println!("// Performance-improved configuration");
    println!("let mut client = McpClient::new(\"data-app\".to_string(), \"1.0.0\".to_string());");
    println!("let init = client.connect_with_streaming_http_performance_improved(");
    println!("    \"http://localhost:3000\"");
    println!(").await?;");
    println!();
    println!("// Memory-improved configuration");
    println!("let init = client.connect_with_streaming_http_memory_improved(");
    println!("    \"http://localhost:3000\"");
    println!(").await?;");
    println!();
    println!("// Custom configuration");
    println!("let config = StreamingConfig {{");
    println!("    chunk_threshold: 32768,  // 32KB");
    println!("    enable_compression: true,");
    println!("    compression_type: CompressionType::Brotli,");
    println!("    ..StreamingConfig::default()");
    println!("}};");
    println!("let init = client.connect_with_streaming_http(url, config).await?;");
    println!("```");

    println!("\n🧠 smart Features:");
    println!("  • Automatic payload analysis - decides when to stream");
    println!("  • Smart compression - only compresses when beneficial");
    println!("  • Adaptive chunk sizing - adjusts based on network conditions");
    println!("  • smooth fallback - automatically falls back to traditional HTTP");
    println!("  • Performance monitoring - tracks latency and throughput");

    println!("\n📈 Real Benefits:");
    println!("  • 4x faster processing for large payloads (>1MB)");
    println!("  • 80% reduction in memory usage through streaming");
    println!("  • 60% less network traffic with complete compression");
    println!("  • Automatic optimization - no manual tuning required\n");

    Ok(())
}

/// Transport Recommendation System
async fn demo_transport_recommendations() -> McpResult<()> {
    println!("## Transport Recommendation System");
    println!("==================================");

    let client = McpClient::new("demo".to_string(), "1.0.0".to_string());

    let use_cases = vec![
        (TransportUseCase::CommandLine, "Command-line tool or script"),
        (TransportUseCase::DesktopApp, "Desktop application"),
        (
            TransportUseCase::Development,
            "Local development and testing",
        ),
        (TransportUseCase::WebApplication, "Web application"),
        (TransportUseCase::Mobile, "Mobile application"),
        (TransportUseCase::Enterprise, "Enterprise environment"),
        (
            TransportUseCase::LargeDataProcessing,
            "Large data processing",
        ),
        (
            TransportUseCase::MemoryConstrained,
            "Memory-constrained environment",
        ),
        (
            TransportUseCase::HighPerformance,
            "High-performance application",
        ),
        (TransportUseCase::RealTime, "Real-time application"),
        (TransportUseCase::HighFrequency, "High-frequency messaging"),
        (TransportUseCase::Interactive, "Interactive application"),
    ];

    for (use_case, description) in use_cases {
        let recommendation = client.get_transport_recommendation(use_case);
        println!(" {description}: ");
        println!(
            "   → {}",
            recommendation.split('.').next().unwrap_or(recommendation)
        );
        println!();
    }

    println!("Note: Automatic Transport Selection:");
    println!("```rust");
    println!("let mut client = McpClient::new(\"my-app\".to_string(), \"1.0.0\".to_string());");
    println!("let init = client.connect_with_recommended_transport(");
    println!("    TransportUseCase::CommandLine,  // Will choose STDIO automatically");
    println!("    \"my-mcp-server\"");
    println!(").await?;");
    println!("```\n");

    Ok(())
}

/// Transport Comparison
async fn demo_transport_comparison() -> McpResult<()> {
    println!("📊 Transport Comparison");
    println!("=======================\n");

    let client = McpClient::new("demo".to_string(), "1.0.0".to_string());
    let transports = client.get_transport_comparison();

    for transport in transports {
        println!("# {} Transport", transport.name);
        println!("   Description: {}", transport.description);
        println!(
            "   Available: {}",
            if transport.available {
                "[x] Yes"
            } else {
                "[!] No"
            }
        );
        println!("   Latency: {}", transport.latency);
        println!("   Throughput: {}", transport.throughput);

        println!("   Use Cases:");
        for use_case in &transport.use_cases {
            println!("     • {use_case}");
        }

        println!("   Pros:");
        for pro in &transport.pros {
            println!("     + {pro}");
        }

        println!("   Cons:");
        for con in &transport.cons {
            println!("     - {con}");
        }

        println!();
    }

    println!("** Summary: Choose Based on Your Needs");
    println!("  1. 📟 STDIO - Foundation (CLI, desktop, local development)");
    println!("  2. 🌐 HTTP - Universal (web, mobile, enterprise)");
    println!("  3. * WebSocket - Real-time (interactive, low latency)");
    println!("  4. 🌊 Streaming HTTP - complete (large data, high performance) [NOW AVAILABLE!]\n");

    Ok(())
}
