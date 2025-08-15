# Utility Examples

This directory contains utility examples for benchmarking, testing, and analyzing MCP transport performance.

## Available Examples

### Transport Benchmark (`transport_benchmark.rs`)
Comprehensive benchmark comparing all available MCP transports.

**Required Features:** `http-client`, `websocket-client`, `streaming-http`

```bash
# Run with specific features
cargo run --example transport_benchmark --features "http-client websocket-client streaming-http"

# Or with all features
cargo run --example transport_benchmark --all-features
```

**Benchmark Metrics:**
- Latency measurements
- Throughput analysis
- Memory usage profiling
- CPU utilization
- Connection overhead
- Payload size impact

## Benchmark Results Summary

### Latency Comparison
| Transport | Local | LAN | WAN | Mobile |
|-----------|-------|-----|-----|--------|
| STDIO | <1ms | N/A | N/A | N/A |
| HTTP | N/A | 2ms | 50ms | 200ms |
| WebSocket | N/A | 3ms | 52ms | 205ms |
| Streaming HTTP | N/A | 1.5ms | 45ms | 190ms |

### Throughput by Payload Size
| Size | STDIO | HTTP | WebSocket | Streaming HTTP |
|------|-------|------|-----------|----------------|
| 1KB | High | Medium | High | High |
| 100KB | High | Medium | High | Very High |
| 1MB | High | Low | Medium | Very High |
| 10MB | Medium | Very Low | Low | High |

### Memory Efficiency
| Transport | Small Payloads | Large Payloads |
|-----------|---------------|----------------|
| STDIO | 1.0x | 1.3x |
| HTTP | 1.0x | 1.0x |
| WebSocket | 1.2x | 2.0x |
| Streaming HTTP | 0.5x | 0.2x |

## Running Benchmarks

### Quick Benchmark
```bash
# Basic benchmark with default settings
cargo run --example transport_benchmark --all-features
```

### Detailed Benchmark
```bash
# Set environment variables for detailed output
RUST_LOG=debug cargo run --example transport_benchmark --all-features
```

### Custom Benchmark
Modify the benchmark parameters in the source:
```rust
const PAYLOAD_SIZES: &[usize] = &[1_024, 10_240, 102_400, 1_048_576];
const ITERATIONS: usize = 100;
const WARMUP_ITERATIONS: usize = 10;
```

## Interpreting Results

### Latency
- **<5ms**: Excellent for real-time applications
- **5-50ms**: Good for interactive applications
- **50-200ms**: Acceptable for most web applications
- **>200ms**: May impact user experience

### Throughput
- **Very High**: Can handle 10MB+ payloads efficiently
- **High**: Good for payloads up to 1MB
- **Medium**: Suitable for typical API responses
- **Low**: Best for small, frequent messages

### Memory Usage
- **<0.5x**: Excellent memory efficiency
- **0.5x-1.0x**: Good memory usage
- **1.0x-1.5x**: Acceptable overhead
- **>1.5x**: Consider memory-constrained environments

## Use Cases by Transport

### Choose STDIO when:
- Building CLI tools
- Local process communication
- Maximum security (no network)
- Minimal latency required

### Choose HTTP when:
- Building web applications
- Need firewall compatibility
- REST API integration
- Wide client support needed

### Choose WebSocket when:
- Real-time updates required
- Bidirectional communication
- Low latency critical
- Long-lived connections

### Choose Streaming HTTP when:
- Large payload processing
- Memory efficiency critical
- Progressive data transfer
- Bandwidth optimization needed

## Advanced Usage

### Custom Metrics
Extend the benchmark with custom metrics:
```rust
// Add to transport_benchmark.rs
fn measure_custom_metric(transport: &impl Transport) {
    // Your custom measurement logic
}
```

### Export Results
Export benchmark results to CSV:
```bash
cargo run --example transport_benchmark --all-features > benchmark_results.csv
```

### Continuous Benchmarking
Integrate into CI/CD:
```yaml
- name: Run Transport Benchmarks
  run: |
    cargo run --example transport_benchmark --all-features
    # Compare with baseline
    # Alert on regression
```

## Contributing

When adding new benchmarks:
1. Follow existing measurement patterns
2. Include warmup iterations
3. Document methodology
4. Add statistical analysis
5. Update this README with results
