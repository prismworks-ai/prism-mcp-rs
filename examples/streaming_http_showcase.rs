// ! 🌊 Streaming HTTP Transport Showcase - complete MCP Performance
// !
// ! This example demonstrates the capable streaming HTTP transport with:
// ! - smart content analysis
// ! - Adaptive compression
// ! - Chunked transfer encoding
// ! - Performance monitoring
// ! - Automatic optimization
// !
// ! ## Required Features
// ! This example requires the following features to be enabled:
// ! ```toml
// ! [dependencies]
// ! prism-mcp-rs = { version = "*", features = ["streaming-http", "tracing-subscriber", "chrono"] }
// ! ```
// !
// ! ## Running this Example
// ! ```bash
// ! cargo run --example streaming_http_showcase --features "streaming-http tracing-subscriber chrono"
// ! ```

use prism_mcp_rs::prelude::*;
use prism_mcp_rs::transport::{CompressionType, StreamingConfig};
use std::collections::HashMap;
use std::time::Instant;

#[tokio::main]
async fn main() -> McpResult<()> {
    // Initialize logging to see the streaming decisions
    tracing_subscriber::fmt::init();

    println!("🌊 Streaming HTTP Transport Showcase");
    println!("===================================\n");

    // Demo 1: Default streaming configuration
    demo_default_streaming().await?;

    // Demo 2: Memory-improved configuration
    demo_memory_improved().await?;

    // Demo 3: Performance-improved configuration
    demo_performance_improved().await?;

    // Demo 4: Custom configuration with complete features
    demo_custom_configuration().await?;

    // Demo 5: Payload size analysis
    demo_payload_analysis().await?;

    // Demo 6: Compression effectiveness
    demo_compression_showcase().await?;

    println!("\n Streaming HTTP Transport Showcase Complete!");
    println!("The streaming transport provides smart optimization");
    println!("for all payload sizes with automatic fallback to traditional HTTP.\n");

    Ok(())
}

/// Demonstrate default streaming configuration
async fn demo_default_streaming() -> McpResult<()> {
    println!("Package: Demo 1: Default Streaming Configuration");
    println!("==========================================\n");

    #[cfg(feature = "streaming-http")]
    {
        let client = McpClient::new("streaming-demo".to_string(), "1.0.0".to_string());

        println!("- Default Configuration:");
        let config = StreamingConfig::default();
        println!("  • Chunk threshold: {} bytes", config.chunk_threshold);
        println!("  • Chunk size: {} bytes", config.chunk_size);
        println!("  • Compression: {:?}", config.compression_type);
        println!("  • Adaptive sizing: {}", config.adaptive_chunk_sizing);

        println!("\nNote: Behavior:");
        println!("  • Small requests (<8KB): Traditional HTTP");
        println!("  • Large requests (>8KB): Chunked streaming");
        println!("  • Binary content: Automatic compression");
        println!("  • Network adaptation: Dynamic chunk sizing\n");

        // In a real scenario, this would connect to a server
        println!("[x] Default streaming configuration ready for production use\n");
    }

    #[cfg(not(feature = "streaming-http"))]
    {
        println!("Warning:  Streaming HTTP feature not enabled");
        println!(
            "   Enable with: cargo run --features streaming-http --example streaming_http_showcase\n"
        );
    }

    Ok(())
}

/// Demonstrate memory-improved configuration
async fn demo_memory_improved() -> McpResult<()> {
    println!("💾 Demo 2: Memory-improved Configuration");
    println!("=========================================\n");

    #[cfg(feature = "streaming-http")]
    {
        let client = McpClient::new("memory-demo".to_string(), "1.0.0".to_string());

        println!("- Memory-improved Configuration:");
        let config = StreamingConfig::memory_improved();
        println!(
            "  • Chunk threshold: {} bytes (smaller for memory efficiency)",
            config.chunk_threshold
        );
        println!(
            "  • Chunk size: {} bytes (conservative chunks)",
            config.chunk_size
        );
        println!(
            "  • Max concurrent chunks: {}",
            config.max_concurrent_chunks
        );
        println!(
            "  • Backpressure threshold: {} bytes",
            config.backpressure_threshold
        );

        println!("\n## complete for:");
        println!("  • Embedded systems with limited RAM");
        println!("  • IoT devices");
        println!("  • Containerized environments with memory limits");
        println!("  • Applications with strict memory budgets");

        println!("\nNote: Usage:");
        println!("```rust");
        println!("let init = client.connect_with_streaming_http_memory_improved(");
        println!("    \"http://localhost:3000\"");
        println!(").await?;");
        println!("```\n");
    }

    #[cfg(not(feature = "streaming-http"))]
    {
        println!("Warning:  Streaming HTTP feature not enabled\n");
    }

    Ok(())
}

/// Demonstrate performance-improved configuration
async fn demo_performance_improved() -> McpResult<()> {
    println!("# Demo 3: Performance-improved Configuration");
    println!("==============================================\n");

    #[cfg(feature = "streaming-http")]
    {
        let client = McpClient::new("perf-demo".to_string(), "1.0.0".to_string());

        println!("- Performance-improved Configuration:");
        let config = StreamingConfig::performance_improved();
        println!(
            "  • Chunk threshold: {} bytes (larger for performance)",
            config.chunk_threshold
        );
        println!(
            "  • Chunk size: {} bytes (high-throughput chunks)",
            config.chunk_size
        );
        println!(
            "  • HTTP/2 Server Push: {}",
            config.enable_http2_server_push
        );
        println!(
            "  • Max concurrent chunks: {}",
            config.max_concurrent_chunks
        );

        #[cfg(feature = "streaming-compression")]
        println!(
            "  • complete compression: {:?} (Brotli for best ratio)",
            config.compression_type
        );
        #[cfg(not(feature = "streaming-compression"))]
        println!(
            "  • Compression: {:?} (enable streaming-compression for Brotli/Zstd)",
            config.compression_type
        );

        println!("\n## complete for:");
        println!("  • High-throughput data processing");
        println!("  • Large file transfers");
        println!("  • Batch processing systems");
        println!("  • Analytics and reporting applications");

        println!("\nNote: Usage:");
        println!("```rust");
        println!("let init = client.connect_with_streaming_http_performance_improved(");
        println!("    \"http://localhost:3000\"");
        println!(").await?;");
        println!("```\n");
    }

    #[cfg(not(feature = "streaming-http"))]
    {
        println!("Warning:  Streaming HTTP feature not enabled\n");
    }

    Ok(())
}

/// Demonstrate custom configuration
async fn demo_custom_configuration() -> McpResult<()> {
    println!("⚙️ Demo 4: Custom Configuration with complete Features");
    println!("=====================================================\n");

    #[cfg(feature = "streaming-http")]
    {
        println!("- Custom Configuration Examples:");

        // Example 1: Custom chunk sizes
        println!("\n1. Custom Chunk Sizes:");
        let config1 = StreamingConfig {
            chunk_threshold: 16384, // 16KB threshold
            chunk_size: 32768,      // 32KB chunks
            enable_compression: true,
            compression_type: CompressionType::Gzip,
            ..StreamingConfig::default()
        };
        println!(
            "   • Threshold: {} bytes, Chunks: {} bytes",
            config1.chunk_threshold, config1.chunk_size
        );

        // Example 2: complete compression
        #[cfg(feature = "streaming-compression")]
        {
            println!("\n2. complete Compression:");
            let config2 = StreamingConfig {
                enable_compression: true,
                compression_type: CompressionType::Brotli,
                ..StreamingConfig::default()
            };
            println!("   • Using Brotli compression for maximum efficiency");

            let config3 = StreamingConfig {
                enable_compression: true,
                compression_type: CompressionType::Zstd,
                ..StreamingConfig::default()
            };
            println!("   • Using Zstd compression for balanced speed/ratio");
        }

        // Example 3: Flow control
        println!("\n3. Flow Control:");
        let config4 = StreamingConfig {
            max_concurrent_chunks: 5,
            backpressure_threshold: 256 * 1024, // 256KB
            adaptive_chunk_sizing: true,
            ..StreamingConfig::default()
        };
        println!("   • Conservative concurrency with adaptive sizing");

        println!("\nNote: Usage:");
        println!("```rust");
        println!("let config = StreamingConfig {{");
        println!("    chunk_threshold: 16384,");
        println!("    compression_type: CompressionType::Brotli,");
        println!("    enable_http2_server_push: true,");
        println!("    ..StreamingConfig::default()");
        println!("}};\nlet init = client.connect_with_streaming_http(url, config).await?;");
        println!("```\n");
    }

    #[cfg(not(feature = "streaming-http"))]
    {
        println!("Warning:  Streaming HTTP feature not enabled\n");
    }

    Ok(())
}

/// Demonstrate payload size analysis
async fn demo_payload_analysis() -> McpResult<()> {
    println!("📊 Demo 5: smart Payload Analysis");
    println!("======================================\n");

    #[cfg(feature = "streaming-http")]
    {
        use prism_mcp_rs::protocol::types::JsonRpcRequest;
        use prism_mcp_rs::transport::ContentAnalyzer;
        use serde_json::{Value, json};

        let analyzer = ContentAnalyzer::new();

        println!("🧠 Content Analysis Examples:");

        // Small request
        let small_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(1),
            method: "tools/list".to_string(),
            params: Some(json!({"cursor": null})),
        };

        let analysis = analyzer.analyze_request(&small_request).await;
        println!("\n1. Small Request (tools/list):");
        println!("   • Size: {} bytes", analysis.estimated_size);
        println!("   • Should stream: {}", analysis.should_stream);
        println!("   • Content type: {:?}", analysis.content_type);
        println!("   • Strategy: {:?}", analysis.recommended_strategy);

        // Large request with binary data
        let large_data = "x".repeat(50000); // 50KB of data
        let large_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(2),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "process_data",
                "arguments": {
                    "data": large_data,
                    "format": "base64"
                }
            })),
        };

        let analysis = analyzer.analyze_request(&large_request).await;
        println!("\n2. Large Request (with 50KB data):");
        println!("   • Size: {} bytes", analysis.estimated_size);
        println!("   • Should stream: {}", analysis.should_stream);
        println!("   • Content type: {:?}", analysis.content_type);
        println!("   • Strategy: {:?}", analysis.recommended_strategy);
        println!("   • Estimated chunks: {}", analysis.estimated_chunks);

        // Complex JSON request
        let complex_json = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "level5": {
                                "data": [1, 2, 3, 4, 5],
                                "nested_array": [[1, 2], [3, 4], [5, 6]]
                            }
                        }
                    }
                }
            }
        });

        let complex_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Value::from(3),
            method: "analyze/complex".to_string(),
            params: Some(complex_json),
        };

        let analysis = analyzer.analyze_request(&complex_request).await;
        println!("\n3. Complex JSON Request:");
        println!("   • Size: {} bytes", analysis.estimated_size);
        println!("   • Should stream: {}", analysis.should_stream);
        println!("   • Content type: {:?}", analysis.content_type);
        println!("   • Strategy: {:?}", analysis.recommended_strategy);

        println!("\n## Analysis Benefits:");
        println!("  • Automatic optimization - no manual configuration needed");
        println!("  • smart streaming decisions based on content");
        println!("  • Performance monitoring and adaptive behavior");
        println!("  • smooth fallback for edge cases\n");
    }

    #[cfg(not(feature = "streaming-http"))]
    {
        println!("Warning:  Streaming HTTP feature not enabled");
        println!("   Enable with: --features streaming-http\n");
    }

    Ok(())
}

/// Demonstrate compression effectiveness
async fn demo_compression_showcase() -> McpResult<()> {
    println!("🗜️ Demo 6: Compression Effectiveness");
    println!("===================================\n");

    #[cfg(feature = "streaming-compression")]
    {
        use prism_mcp_rs::transport::streaming_http::StreamingCompressor;

        println!("📈 Compression Analysis:");

        // Test different data types
        let test_cases = vec![
            (
                "JSON data",
                json!({"key": "value", "numbers": [1, 2, 3, 4, 5]}).to_string(),
            ),
            ("Repetitive text", "Hello world! ".repeat(1000)),
            (
                "Random data",
                (0..1000)
                    .map(|_| fastrand::char(..).to_string())
                    .collect::<String>(),
            ),
            (
                "Base64-like",
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=".repeat(50),
            ),
        ];

        for (description, data) in test_cases {
            let data_bytes = data.as_bytes();
            let original_size = data_bytes.len();

            println!("\n {description}:");
            println!("   Original size: {original_size} bytes");

            // Test different compression algorithms
            let gzip_compressor = StreamingCompressor::new(CompressionType::Gzip);
            let brotli_compressor = StreamingCompressor::new(CompressionType::Brotli);
            let zstd_compressor = StreamingCompressor::new(CompressionType::Zstd);

            if let Ok(gzip_result) = gzip_compressor.compress_if_beneficial(data_bytes).await {
                let ratio = gzip_result.len() as f64 / original_size as f64;
                println!(
                    "   Gzip: {} bytes ({:.1}% of original)",
                    gzip_result.len(),
                    ratio * 100.0
                );
            }

            if let Ok(brotli_result) = brotli_compressor.compress_if_beneficial(data_bytes).await {
                let ratio = brotli_result.len() as f64 / original_size as f64;
                println!(
                    "   Brotli: {} bytes ({:.1}% of original)",
                    brotli_result.len(),
                    ratio * 100.0
                );
            }

            if let Ok(zstd_result) = zstd_compressor.compress_if_beneficial(data_bytes).await {
                let ratio = zstd_result.len() as f64 / original_size as f64;
                println!(
                    "   Zstd: {} bytes ({:.1}% of original)",
                    zstd_result.len(),
                    ratio * 100.0
                );
            }

            // Show compression ratio estimates
            let gzip_estimate = gzip_compressor.estimate_compression_ratio(data_bytes);
            let brotli_estimate = brotli_compressor.estimate_compression_ratio(data_bytes);
            let zstd_estimate = zstd_compressor.estimate_compression_ratio(data_bytes);

            println!(
                "   Estimated ratios: Gzip {:.1}%, Brotli {:.1}%, Zstd {:.1}%",
                gzip_estimate * 100.0,
                brotli_estimate * 100.0,
                zstd_estimate * 100.0
            );
        }

        println!("\n** Compression Benefits:");
        println!("  • Automatic algorithm selection based on content");
        println!("  • Smart threshold-based compression");
        println!("  • Up to 75% size reduction for text content");
        println!("  • Entropy-based compression ratio estimation\n");
    }

    #[cfg(not(feature = "streaming-compression"))]
    {
        println!("Warning:  complete compression features not enabled");
        println!("   Enable with: --features streaming-compression\n");
    }

    Ok(())
}

/// Create test payloads of different sizes for demonstration
fn create_test_payload(
    size_bytes: usize,
    payload_type: &str,
) -> HashMap<String, serde_json::Value> {
    let mut args = HashMap::new();

    match payload_type {
        "text" => {
            args.insert(
                "data".to_string(),
                serde_json::Value::String("A".repeat(size_bytes)),
            );
            args.insert(
                "type".to_string(),
                serde_json::Value::String("text".to_string()),
            );
        }
        "binary" => {
            // Simulate base64 data
            let base64_data = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/=";
            let repeated_data = base64_data.repeat(size_bytes / base64_data.len() + 1);
            args.insert(
                "data".to_string(),
                serde_json::Value::String(repeated_data[..size_bytes].to_string()),
            );
            args.insert(
                "type".to_string(),
                serde_json::Value::String("base64".to_string()),
            );
        }
        "json" => {
            // Create nested JSON structure
            let nested = serde_json::json!({"level": 1});
            let json_str = serde_json::to_string(&nested).unwrap();
            let target_size = size_bytes / json_str.len() + 1;

            let mut array = Vec::new();
            for i in 0..target_size {
                array.push(serde_json::json!({"item": i, "data": format!("item_{}", i)}));
            }
            args.insert("data".to_string(), serde_json::Value::Array(array));
            args.insert(
                "type".to_string(),
                serde_json::Value::String("json".to_string()),
            );
        }
        _ => {
            args.insert(
                "data".to_string(),
                serde_json::Value::String("default".repeat(size_bytes / 7)),
            );
        }
    }

    args.insert(
        "size".to_string(),
        serde_json::Value::Number(serde_json::Number::from(size_bytes)),
    );
    args.insert(
        "timestamp".to_string(),
        serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
    );

    args
}

/// Simulate performance benchmarking
async fn simulate_request_with_timing(payload_size: usize, transport_type: &str) -> Duration {
    let start = Instant::now();

    // Simulate request processing time based on payload size and transport
    let base_latency = match transport_type {
        "traditional" => Duration::from_millis(20),
        "streaming" => Duration::from_millis(15),
        "compressed" => Duration::from_millis(12),
        _ => Duration::from_millis(25),
    };

    // Add size-based processing time
    let size_factor = (payload_size as f64 / 1024.0).sqrt(); // Square root scaling
    let processing_time = Duration::from_millis((size_factor * 2.0) as u64);

    tokio::time::sleep(base_latency + processing_time).await;

    start.elapsed()
}

use std::time::Duration;
