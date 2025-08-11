// ! Transport Benchmark
// !
// ! This example compares the performance of different MCP transport implementations,
// ! demonstrating the HTTP transport performance characteristics.
// !
// ! Run with: cargo run --example transport_benchmark --all-features

use futures;
use prism_mcp_rs::prelude::*;
use prism_mcp_rs::transport::{HttpClientTransport, TransportConfig};
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{info, warn};

const BENCHMARK_REQUESTS: usize = 50; // Reduced for simpler demo
const CONCURRENT_REQUESTS: usize = 5;
const SERVER_PORT: u16 = 3002;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("# MCP Transport Benchmark");
    info!(
        "Comparing transport performance with {} requests",
        BENCHMARK_REQUESTS
    );

    // Start demo server
    let server_task = tokio::spawn(async {
        if let Err(e) = demo_benchmark_server().await {
            eprintln!("Demo server error: {}", e);
        }
    });

    // Give server time to start
    sleep(Duration::from_millis(1000)).await;

    let server_url = format!("http://localhost:{}", SERVER_PORT);

    // Benchmark 1: HTTP Transport (Fast config)
    info!("\n📊 Benchmarking HTTP Transport (Fast Config)...");
    let fast_result = benchmark_http_transport(&server_url, create_fast_config()).await?;

    // Benchmark 2: HTTP Transport (Conservative config)
    info!("\n📊 Benchmarking HTTP Transport (Conservative Config)...");
    let conservative_result =
        benchmark_http_transport(&server_url, create_conservative_config()).await?;

    // Benchmark 3: Standard HTTP Transport (for comparison)
    info!("\n📊 Benchmarking Standard HTTP Transport...");
    let standard_result = benchmark_standard_http(&server_url).await?;

    // Display results
    print_benchmark_results(fast_result, conservative_result, standard_result);

    // Cleanup
    server_task.abort();

    Ok(())
}

#[derive(Debug, Clone)]
struct BenchmarkResult {
    name: String,
    total_requests: usize,
    total_time: Duration,
    average_latency: Duration,
    requests_per_second: f64,
    success_rate: f64,
    errors: usize,
}

fn create_fast_config() -> TransportConfig {
    TransportConfig {
        connect_timeout_ms: Some(1_000),
        read_timeout_ms: Some(5_000),
        write_timeout_ms: Some(5_000),
        max_message_size: Some(1024 * 1024), // 1MB
        keep_alive_ms: Some(60_000),         // 1 minute
        compression: false,
        headers: std::collections::HashMap::new(),
    }
}

fn create_conservative_config() -> TransportConfig {
    TransportConfig {
        connect_timeout_ms: Some(10_000),
        read_timeout_ms: Some(30_000),
        write_timeout_ms: Some(30_000),
        max_message_size: Some(512 * 1024), // 512KB
        keep_alive_ms: Some(300_000),       // 5 minutes
        compression: true,
        headers: std::collections::HashMap::new(),
    }
}

async fn benchmark_http_transport(
    url: &str,
    config: TransportConfig,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    let transport = HttpClientTransport::with_config(url, None, config.clone()).await?;

    let mut client = McpClient::new("benchmark-client".to_string(), "1.0.0".to_string());

    match client.connect(transport).await {
        Ok(_) => {}
        Err(e) => {
            warn!("Failed to connect: {}", e);
            return Ok(BenchmarkResult {
                name: format!("HTTP ({:?}ms timeout)", config.read_timeout_ms.unwrap_or(0)),
                total_requests: 0,
                total_time: Duration::ZERO,
                average_latency: Duration::ZERO,
                requests_per_second: 0.0,
                success_rate: 0.0,
                errors: BENCHMARK_REQUESTS,
            });
        }
    }

    let start_time = Instant::now();
    let mut errors = 0;
    let mut latencies = Vec::new();

    // Run concurrent benchmark requests (simplified version)
    for batch in 0..(BENCHMARK_REQUESTS / CONCURRENT_REQUESTS) {
        let mut batch_futures = Vec::new();

        for _ in 0..CONCURRENT_REQUESTS {
            let mut params = HashMap::new();
            params.insert("batch".to_string(), json!(batch));
            params.insert(
                "timestamp".to_string(),
                json!(std::time::Instant::now().elapsed().as_millis()),
            );

            let future = client.call_tool("benchmark_tool".to_string(), Some(params));
            batch_futures.push(future);
        }

        // Wait for this batch to complete
        let results = futures::future::join_all(batch_futures).await;

        for (result, request_start) in results
            .into_iter()
            .zip(std::iter::repeat(std::time::Instant::now()))
        {
            let latency = request_start.elapsed();
            latencies.push(latency);

            if result.is_err() {
                errors += 1;
            }
        }

        // Small delay between batches to avoid overwhelming
        sleep(Duration::from_millis(10)).await;
    }

    let total_time = start_time.elapsed();
    let average_latency = latencies.iter().sum::<Duration>() / latencies.len().max(1) as u32;
    let requests_per_second = BENCHMARK_REQUESTS as f64 / total_time.as_secs_f64();
    let success_rate = (BENCHMARK_REQUESTS - errors) as f64 / BENCHMARK_REQUESTS as f64;

    Ok(BenchmarkResult {
        name: format!("HTTP ({:?}ms timeout)", config.read_timeout_ms.unwrap_or(0)),
        total_requests: BENCHMARK_REQUESTS,
        total_time,
        average_latency,
        requests_per_second,
        success_rate,
        errors,
    })
}

async fn benchmark_standard_http(url: &str) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
    let client = Client::new();

    let start_time = Instant::now();
    let mut errors = 0;
    let mut latencies = Vec::new();

    // Run requests with standard HTTP client
    for i in 0..BENCHMARK_REQUESTS {
        let request_start = Instant::now();

        let request_body = json!({
            "jsonrpc": "2.0",
            "id": i,
            "method": "tools/call",
            "params": {
                "name": "benchmark_tool",
                "arguments": {"request": i}
            }
        });

        let result = client.post(url).json(&request_body).send().await;

        let latency = request_start.elapsed();
        latencies.push(latency);

        match result {
            Ok(response) => {
                if !response.status().is_success() {
                    errors += 1;
                }
            }
            Err(_) => errors += 1,
        }

        // Small delay to be fair to single-connection approach
        if i % 10 == 0 {
            sleep(Duration::from_millis(1)).await;
        }
    }

    let total_time = start_time.elapsed();
    let average_latency = latencies.iter().sum::<Duration>() / latencies.len().max(1) as u32;
    let requests_per_second = BENCHMARK_REQUESTS as f64 / total_time.as_secs_f64();
    let success_rate = (BENCHMARK_REQUESTS - errors) as f64 / BENCHMARK_REQUESTS as f64;

    Ok(BenchmarkResult {
        name: "Standard HTTP".to_string(),
        total_requests: BENCHMARK_REQUESTS,
        total_time,
        average_latency,
        requests_per_second,
        success_rate,
        errors,
    })
}

fn print_benchmark_results(
    fast: BenchmarkResult,
    conservative: BenchmarkResult,
    standard: BenchmarkResult,
) {
    info!("\n## BENCHMARK RESULTS");
    info!("═══════════════════════════════════════════════════════════════");

    let results = vec![&fast, &conservative, &standard];

    // Print header
    info!(
        "{:<25} {:>12} {:>15} {:>12} {:>10} {:>12}",
        "Transport", "Req/Sec", "Avg Latency", "Success %", "Errors", "Total Time"
    );
    info!("─────────────────────────────────────────────────────────────────────────");

    // Print results
    for result in &results {
        info!(
            "{:<25} {:>12.0} {:>13.2}ms {:>11.1}% {:>10} {:>10.2}s",
            result.name,
            result.requests_per_second,
            result.average_latency.as_secs_f64() * 1000.0,
            result.success_rate * 100.0,
            result.errors,
            result.total_time.as_secs_f64()
        );
    }

    info!("─────────────────────────────────────────────────────────────────────────");

    info!("\n📊 SUMMARY:");
    info!("Total requests per transport: {}", fast.total_requests);

    // Calculate improvements
    if standard.requests_per_second > 0.0 {
        let fast_improvement = ((fast.requests_per_second - standard.requests_per_second)
            / standard.requests_per_second)
            * 100.0;
        let conservative_improvement = ((conservative.requests_per_second
            - standard.requests_per_second)
            / standard.requests_per_second)
            * 100.0;

        info!("\n📈 PERFORMANCE COMPARISON:");
        info!(
            "HTTP (Fast):         {:+.1}% vs Standard HTTP",
            fast_improvement
        );
        info!(
            "HTTP (Conservative): {:+.1}% vs Standard HTTP",
            conservative_improvement
        );
    }

    info!("\nNote: RECOMMENDATIONS:");
    info!("• Use HTTP (Fast) for development and testing");
    info!("• Use HTTP (Conservative) for production reliability");
    info!("• HTTP transport provides reliable MCP communication");
    info!("• Configure timeouts based on your use case");
}

/// Demo server for benchmarking
async fn demo_benchmark_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use axum::{Router, response::Json, routing::post};
    use std::net::SocketAddr;

    let app = Router::new().route(
        "/",
        post(|| async {
            // Simulate some processing time
            sleep(Duration::from_micros(100)).await;

            Json(json!({
                "jsonrpc": "2.0",
                "result": {
                    "content": "Benchmark response",
                    "isError": false
                },
                "id": 1
            }))
        }),
    );

    let addr = SocketAddr::from(([127, 0, 0, 1], SERVER_PORT));
    info!("Benchmark server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
