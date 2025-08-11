# Utility Examples

This directory contains utility examples and tools for working with the MCP Protocol SDK.

# Examples

# Transport Benchmark
- **File**: `transport_benchmark.rs`
- **Features**: `http`, `tracing-subscriber`
- **Description**: Benchmarking tool to compare performance across different transport methods

**Key Code Pattern:**
```rust
use mcp_protocol_sdk::{
 client::{McpClient, ClientSession},
 transport::{stdio::StdioClientTransport, http::HttpClientTransport},
};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct BenchmarkResult {
 name: String,
 total_requests: usize,
 total_time: Duration,
 avg_latency_ms: f64,
 requests_per_second: f64,
}

async fn benchmark_transport<T>(
 name: &str,
 transport: T,
 num_requests: usize,
) -> Result<BenchmarkResult, Box<dyn std::error::Error>>
where
 T: Transport + Send + Sync + 'static,
{
 let client = McpClient::new("benchmark-client".to_string(), "1.0.0".to_string());
 let session = ClientSession::new(client);
 
 let init_result = session.connect(transport).await?;
 println!("Connected to {} for benchmarking", init_result.server_info.name);
 
 let start_time = Instant::now();
 
 for i in 0..num_requests {
 let client = session.client();
 let client_guard = client.lock().await;
 
 // Benchmark tool calls
 let mut args = HashMap::new();
 args.insert("message".to_string(), json!(format!("Request {}", i)));
 
 let _result = client_guard.call_tool("echo".to_string(), Some(args)).await?;
 }
 
 let total_time = start_time.elapsed();
 let avg_latency_ms = total_time.as_millis() as f64 / num_requests as f64;
 let requests_per_second = num_requests as f64 / total_time.as_secs_f64();
 
 Ok(BenchmarkResult {
 name: name.to_string(),
 total_requests: num_requests,
 total_time,
 avg_latency_ms,
 requests_per_second,
 })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
 // Benchmark different transports
 let num_requests = 100;
 
 println!("# MCP Transport Benchmark");
 println!("=".repeat(50));
 
 // Benchmark STDIO transport
 let stdio_transport = StdioClientTransport::new("./echo-server".to_string()).await?;
 let stdio_result = benchmark_transport("STDIO", stdio_transport, num_requests).await?;
 
 // Benchmark HTTP transport 
 let http_transport = HttpClientTransport::new("http://localhost:3000/mcp".to_string()).await?;
 let http_result = benchmark_transport("HTTP", http_transport, num_requests).await?;
 
 // Display results
 println!("\nBenchmark Results:");
 println!("-".repeat(50));
 
 for result in vec![stdio_result, http_result] {
 println!(
 "{}: {:.2} req/sec, {:.2}ms avg latency",
 result.name, result.requests_per_second, result.avg_latency_ms
 );
 }
 
 Ok(())
}
```

**Sample Output:**
```
# MCP Transport Benchmark
==================================================
Connected to echo-server for benchmarking
Connected to http-server for benchmarking

Benchmark Results:
--------------------------------------------------
STDIO: 45.23 req/sec, 22.11ms avg latency
HTTP: 67.89 req/sec, 14.73ms avg latency
```

# Running Examples

```bash
# Run transport benchmark
cargo run --example transport_benchmark --features "http,tracing-subscriber"
```

# Performance Testing Patterns

# Latency Measurement
```rust
use std::time::Instant;

let start = Instant::now();
// Perform operation
let latency = start.elapsed();
println!("Operation took: {:?}", latency);
```

# Throughput Testing
```rust
use tokio::time::{Duration, sleep};

let mut successful_requests = 0;
let mut failed_requests = 0;

for _ in 0..1000 {
 match client_guard.call_tool("test".to_string(), None).await {
 Ok(_) => successful_requests += 1,
 Err(_) => failed_requests += 1,
 }
 
 // Optional: Add delay between requests
 sleep(Duration::from_millis(10)).await;
}

let success_rate = (successful_requests as f64 / 1000.0) * 100.0;
println!("Success rate: {:.2}%", success_rate);
```

# Memory Usage Monitoring
```rust
// Simple memory usage tracking
fn get_memory_usage() -> usize {
 // This is a simplified example
 // In practice, you might use system crates like `sysinfo`
 std::alloc::System.usage().bytes_allocated
}

let initial_memory = get_memory_usage();
// Perform operations
let final_memory = get_memory_usage();
println!("Memory delta: {} bytes", final_memory - initial_memory);
```

# Connection Health Checks
```rust
async fn health_check(session: &ClientSession) -> bool {
 let client = session.client();
 match client.lock().await.list_tools().await {
 Ok(_) => true,
 Err(_) => false,
 }
}

// Use in benchmark
if!health_check(&session).await {
 eprintln!("Connection health check failed");
 return Err("Connection lost".into());
}
```

# Adding New Utilities

When adding new utility examples to this directory:

1. **Follow the naming convention**: `snake_case.rs`
2. **Add appropriate example entry** to `Cargo.toml`:
 ```toml
 [[example]]
 name = "my_utility"
 path = "examples/utilities/my_utility.rs"
 required-features = ["http", "tracing-subscriber"]
 ```
3. **Include proper documentation** and usage instructions
4. **Update this README** with the new utility and code snippets
5. **Add complete error handling** and logging
6. **Include performance metrics** where applicable

# Utility Template
```rust
//! My Utility Example
//!
//! Description of what this utility does.

use mcp_protocol_sdk::*;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
 // Initialize tracing
 tracing_subscriber::fmt::init();
 
 info!("Starting my utility...");
 
 // Your utility logic here
 
 info!("Utility completed successfully");
 Ok(())
}
```

For general examples documentation, see the [Examples Guide](../../docs/examples.md).
