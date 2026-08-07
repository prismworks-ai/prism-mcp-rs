# Benchmark Report

## Summary

Generated on: 2026-08-06

## Criterion Results

| Benchmark | Median estimate |
|-----------|-----------------|
| `plugin_config_creation` | 12.42 ns |
| `tool_registration` | 352.23 ns |
| `tool_lookup` | 14.77 ns |
| `plugin_metadata_creation` | 89.92 ns |
| `call_tool_result_generation` | 254.59 ns |
| `server_request_dispatch_ping` | 1.304 µs |
| `endpoint_failover_read` | 486.26 ns |

These are development-machine measurements with Criterion sample size 10, intended to verify that the benchmark paths execute. They are not production latency or throughput guarantees.

## System Information

- **OS:** Darwin 25.5.0
- **Arch:** arm64
- **CPU:** Apple M5 Pro
- **Rust:** rustc 1.94.0 (4a4ef493e 2026-03-02)
- **Cargo:** cargo 1.94.0 (85eff7c80 2026-01-15)

## Notes

- Benchmarks are run with `cargo bench --features bench,plugin,http --bench all_benchmarks`
- Results may vary based on system load and configuration
- For consistent results, run benchmarks on dedicated hardware
