# Performance Guide

This guide covers performance optimization techniques and best practices for the Prism MCP SDK.

## Overview

The Prism MCP SDK is designed for high performance with features like async I/O, connection pooling, and efficient serialization. This guide helps you optimize your MCP applications for maximum throughput and minimal latency.

## Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench protocol_bench

# Generate HTML report
cargo bench -- --output-format bencher | tee target/bench.txt
```

### Writing Custom Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use prism_mcp_rs::prelude::*;

fn bench_tool_execution(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let handler = MyToolHandler::new();
    
    c.bench_function("tool_execution", |b| {
        b.to_async(&runtime).iter(|| async {
            let args = HashMap::from([
                ("input".to_string(), json!("test data")),
            ]);
            let result = handler.call(black_box(args)).await;
            black_box(result)
        });
    });
}

criterion_group!(benches, bench_tool_execution);
criterion_main!(benches);
```

## Async Performance

### Efficient Async Operations

```rust
use futures::future::join_all;
use tokio::task;

#[async_trait]
impl ToolHandler for ParallelProcessor {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let items: Vec<String> = arguments.get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| McpError::InvalidParams {
                message: "Missing 'items' array".to_string(),
            })?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        
        // Process items in parallel
        let handles: Vec<_> = items
            .into_iter()
            .map(|item| {
                task::spawn(async move {
                    process_item(item).await
                })
            })
            .collect();
        
        let results = join_all(handles).await;
        
        // Combine results
        let combined = results
            .into_iter()
            .filter_map(|r| r.ok())
            .collect::<Vec<_>>()
            .join(", ");
        
        Ok(ToolResult::text(combined))
    }
}
```

### Connection Pooling

```rust
use deadpool::managed::{Pool, Manager, RecycleResult};
use async_trait::async_trait;

struct ConnectionManager {
    config: ConnectionConfig,
}

#[async_trait]
impl Manager for ConnectionManager {
    type Type = Connection;
    type Error = McpError;
    
    async fn create(&self) -> Result<Connection, McpError> {
        Connection::new(&self.config).await
    }
    
    async fn recycle(&self, conn: &mut Connection) -> RecycleResult<McpError> {
        conn.ping().await.map_err(|e| e.into())
    }
}

pub struct PooledHandler {
    pool: Pool<ConnectionManager>,
}

impl PooledHandler {
    pub async fn new(config: ConnectionConfig, size: usize) -> McpResult<Self> {
        let manager = ConnectionManager { config };
        let pool = Pool::builder(manager)
            .max_size(size)
            .build()?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl ToolHandler for PooledHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let conn = self.pool.get().await?;
        let result = conn.execute_query(&arguments).await?;
        Ok(ToolResult::text(result))
    }
}
```

## Memory Optimization

### Efficient Data Structures

```rust
use bytes::Bytes;
use std::sync::Arc;

// Share immutable data with Arc
pub struct SharedResource {
    data: Arc<Bytes>,
}

impl SharedResource {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: Arc::new(Bytes::from(data)),
        }
    }
    
    pub fn clone_data(&self) -> Arc<Bytes> {
        Arc::clone(&self.data)  // Cheap clone
    }
}

// Use SmallVec for small collections
use smallvec::SmallVec;

pub struct EfficientHandler {
    // Avoids heap allocation for up to 4 items
    cache: SmallVec<[String; 4]>,
}
```

### Stream Processing

Process large data sets without loading everything into memory:

```rust
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::{Stream, StreamExt};
use futures::stream;

pub struct StreamProcessor;

#[async_trait]
impl ResourceHandler for StreamProcessor {
    async fn read(
        &self,
        uri: &str,
        _params: &HashMap<String, String>,
    ) -> McpResult<Vec<ResourceContents>> {
        if let Some(path) = uri.strip_prefix("file://") {
            let file = tokio::fs::File::open(path).await?;
            let reader = BufReader::new(file);
            let mut lines = reader.lines();
            
            let mut processed = Vec::new();
            while let Some(line) = lines.next_line().await? {
                // Process line by line
                if let Some(result) = process_line(&line) {
                    processed.push(result);
                }
                
                // Yield periodically to avoid blocking
                if processed.len() % 1000 == 0 {
                    tokio::task::yield_now().await;
                }
            }
            
            Ok(vec![ResourceContents::Text {
                uri: uri.to_string(),
                mime_type: Some("text/plain".to_string()),
                text: processed.join("\n"),
                meta: None,
            }])
        } else {
            Err(McpError::ResourceNotFound(uri.to_string()))
        }
    }
}
```

## Serialization Performance

### Efficient JSON Handling

```rust
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

// Use RawValue to avoid unnecessary parsing
#[derive(Serialize, Deserialize)]
pub struct LazyMessage {
    pub id: String,
    pub method: String,
    #[serde(borrow)]
    pub params: Box<RawValue>,  // Delays parsing
}

impl LazyMessage {
    pub fn parse_params<T: DeserializeOwned>(&self) -> McpResult<T> {
        serde_json::from_str(self.params.get())
            .map_err(|e| McpError::Json(e))
    }
}

// Use borrowed strings when possible
#[derive(Deserialize)]
pub struct BorrowedData<'a> {
    #[serde(borrow)]
    pub name: &'a str,
    #[serde(borrow)]
    pub value: &'a str,
}
```

### Binary Serialization

For internal communication, consider binary formats:

```rust
use bincode;

#[derive(Serialize, Deserialize)]
pub struct BinaryMessage {
    pub id: u64,
    pub payload: Vec<u8>,
}

impl BinaryMessage {
    pub fn to_bytes(&self) -> McpResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| McpError::Other(e.to_string()))
    }
    
    pub fn from_bytes(bytes: &[u8]) -> McpResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| McpError::Other(e.to_string()))
    }
}
```

## Caching Strategies

### In-Memory Cache

```rust
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct CachedHandler {
    cache: Arc<RwLock<LruCache<String, CacheEntry>>>,
    ttl: Duration,
}

struct CacheEntry {
    value: String,
    expires_at: Instant,
}

impl CachedHandler {
    pub fn new(capacity: usize, ttl_seconds: u64) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(capacity))),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }
    
    async fn get_cached(&self, key: &str) -> Option<String> {
        let mut cache = self.cache.write().await;
        if let Some(entry) = cache.get_mut(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            }
            cache.pop(key);  // Remove expired entry
        }
        None
    }
    
    async fn set_cached(&self, key: String, value: String) {
        let mut cache = self.cache.write().await;
        cache.put(key, CacheEntry {
            value,
            expires_at: Instant::now() + self.ttl,
        });
    }
}

#[async_trait]
impl ToolHandler for CachedHandler {
    async fn call(&self, arguments: HashMap<String, Value>) -> McpResult<ToolResult> {
        let key = serde_json::to_string(&arguments)?;
        
        // Check cache
        if let Some(cached) = self.get_cached(&key).await {
            return Ok(ToolResult::text(cached));
        }
        
        // Compute result
        let result = expensive_computation(&arguments).await?;
        
        // Cache result
        self.set_cached(key, result.clone()).await;
        
        Ok(ToolResult::text(result))
    }
}
```

## Transport Optimization

### Compression

```rust
use flate2::Compression;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use std::io::prelude::*;

pub struct CompressedTransport {
    inner: Box<dyn Transport>,
    compression_threshold: usize,
}

impl CompressedTransport {
    pub fn new(transport: Box<dyn Transport>) -> Self {
        Self {
            inner: transport,
            compression_threshold: 1024,  // Compress if > 1KB
        }
    }
    
    fn compress(&self, data: &[u8]) -> McpResult<Vec<u8>> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }
    
    fn decompress(&self, data: &[u8]) -> McpResult<Vec<u8>> {
        let mut decoder = GzDecoder::new(data);
        let mut result = Vec::new();
        decoder.read_to_end(&mut result)?;
        Ok(result)
    }
}
```

### Batching

```rust
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

pub struct BatchingHandler {
    sender: mpsc::Sender<Request>,
}

impl BatchingHandler {
    pub fn new(batch_size: usize, flush_interval: Duration) -> Self {
        let (sender, mut receiver) = mpsc::channel::<Request>(100);
        
        tokio::spawn(async move {
            let mut batch = Vec::with_capacity(batch_size);
            let mut interval = interval(flush_interval);
            
            loop {
                tokio::select! {
                    Some(request) = receiver.recv() => {
                        batch.push(request);
                        if batch.len() >= batch_size {
                            process_batch(&batch).await;
                            batch.clear();
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            process_batch(&batch).await;
                            batch.clear();
                        }
                    }
                }
            }
        });
        
        Self { sender }
    }
}
```

## Profiling

### CPU Profiling

```bash
# Install profiling tools
cargo install flamegraph

# Run with profiling
cargo flamegraph --bench protocol_bench

# View flamegraph.svg in browser
```

### Memory Profiling

```rust
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

fn main() {
    // Your application
}
```

## Performance Checklist

- [ ] Use async/await for I/O operations
- [ ] Implement connection pooling for external resources
- [ ] Cache frequently accessed data
- [ ] Use appropriate data structures (Arc, Bytes, SmallVec)
- [ ] Enable compression for large payloads
- [ ] Batch operations when possible
- [ ] Profile and benchmark critical paths
- [ ] Use streaming for large data sets
- [ ] Minimize allocations in hot paths
- [ ] Consider binary serialization for internal communication

## Further Reading

- [Benchmarks](../../benches/)
- [Async Programming in Rust](https://rust-lang.github.io/async-book/)
- [The Rust Performance Book](https://nnethercote.github.io/perf-book/)